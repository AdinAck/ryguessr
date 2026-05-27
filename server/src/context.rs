use std::{collections::HashMap, sync::Arc, time::Duration};

use axum::http::StatusCode;
use colors::Srgb8;
use tokio::sync::RwLock;

use crate::{
    Handle, Room,
    colors::PlayerColor,
    geo::{Location, engine::LocationEngine},
    handle,
    name_gen::NameGenerator,
    room,
};

pub type SharedModel = Arc<RwLock<Model>>;

/// How long an idle room (no live SSE connections) is kept around before it
/// and any ghost members are removed.
const CLEANUP_DELAY: Duration = Duration::from_secs(60);

/// The service layer shared with every request handler. Owns the
/// [`LocationEngine`] and a shared handle to the in-memory [`Model`], and is
/// where any lifecycle operation that needs to touch time or spawn background work lives.
#[derive(Clone)]
pub struct Context {
    pub engine: Arc<LocationEngine>,
    pub model: SharedModel,
}

/// The in-memory state of the application. Methods on
/// [`Model`] are synchronous, anything async or time-aware lives on [`Context`].
#[derive(Default)]
pub struct Model {
    pub clients: HashMap<handle::Id, Handle>,
    pub rooms: HashMap<room::Id, Room>,
    pub name_generator: NameGenerator,
}

impl Context {
    /// Create an empty context.
    pub fn new(engine: LocationEngine) -> Self {
        Self {
            engine: Arc::new(engine),
            model: Arc::new(RwLock::new(Model::default())),
        }
    }

    /// Handle the end of an SSE stream: drop the room's connection count,
    /// remove the client if they're still bound to this room, and arm
    /// cleanup if the room is now idle.
    pub async fn on_sse_disconnect(&self, client_id: &handle::Id, room_id: &room::Id) {
        let mut model = self.model.write().await;

        if let Some(room) = model.rooms.get_mut(room_id) {
            room.disconnect();
        }

        let still_in_room = model
            .clients
            .get(client_id)
            .is_some_and(|h| &h.room == room_id);
        if still_in_room {
            model.remove_client(client_id);
            tracing::info!(client_id = %**client_id, "client disconnected, removed from room");
        }

        if model.rooms.get(room_id).is_some_and(Room::is_idle) {
            self.start_cleanup(&mut model, room_id.clone());
        }
    }

    /// Move a client into an existing room. Re-arms the new room's cleanup
    /// timer if it has no live SSE connections.
    pub async fn move_client_to_room(
        &self,
        client_id: &handle::Id,
        new_room_id: &room::Id,
    ) -> Result<(), StatusCode> {
        let mut model = self.model.write().await;
        model.move_client_to_room(client_id, new_room_id)?;

        if model.rooms.get(new_room_id).is_some_and(Room::is_idle) {
            self.start_cleanup(&mut model, new_room_id.clone());
        }

        Ok(())
    }

    /// Create a new room with the given user as the first member. Arms the
    /// cleanup timer immediately so a client that never establishes an SSE
    /// connection doesn't leak the room.
    pub async fn create_room(
        &self,
        location: Location,
        client_id: handle::Id,
        requested_username: Option<String>,
        color_override: Option<Srgb8>,
    ) -> (room::Id, String, Srgb8) {
        let mut model = self.model.write().await;
        let username = requested_username
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| model.generate_unique_name());

        model.remove_client(&client_id);
        let (room_id, color) =
            model.insert_new_room(location, client_id, username.clone(), color_override);

        self.start_cleanup(&mut model, room_id.clone());

        (room_id, username, color)
    }

    /// Spawn a cleanup task for the given room and store its handle on the room
    /// so [`Room::connect`] can abort it if the room becomes live again.
    fn start_cleanup(&self, model: &mut Model, room_id: room::Id) {
        let model_arc = Arc::clone(&self.model);
        let task_room_id = room_id.clone();

        let handle = tokio::spawn(async move {
            tokio::time::sleep(CLEANUP_DELAY).await;
            model_arc.write().await.drop_room_if_idle(&task_room_id);
        });

        let room = model
            .rooms
            .get_mut(&room_id)
            .expect("start_cleanup target room must exist while the write lock is held");

        room.arm_cleanup(handle);
    }
}

impl Model {
    pub fn generate_unique_name(&self) -> String {
        let mut name = self.name_generator.generate();

        // Check for collisions across all existing clients.
        while self.clients.values().any(|c| c.username == name) {
            name = self.name_generator.generate();
        }

        name
    }

    pub fn set_name(&mut self, client_id: &handle::Id, new_name: String) -> Result<(), StatusCode> {
        // Check for name collision across all existing clients.
        if self.clients.values().any(|c| c.username == new_name) {
            return Err(StatusCode::CONFLICT);
        }

        // Update the client's name in the model.
        let handle = self
            .clients
            .get_mut(client_id)
            .ok_or(StatusCode::UNAUTHORIZED)?;
        handle.username = new_name.clone();

        // Update the client's name in their current room.
        let room = self
            .rooms
            .get_mut(&handle.room)
            .ok_or(StatusCode::NOT_FOUND)?;
        if let Some(member) = room.members.get_mut(client_id) {
            member.username = new_name;
        }

        Ok(())
    }

    pub fn set_color(
        &mut self,
        client_id: &handle::Id,
        new_color: Srgb8,
    ) -> Result<(), StatusCode> {
        let handle = self
            .clients
            .get_mut(client_id)
            .ok_or(StatusCode::UNAUTHORIZED)?;
        handle.color = new_color;

        // Update the member's color in their current room
        let room = self
            .rooms
            .get_mut(&handle.room)
            .ok_or(StatusCode::NOT_FOUND)?;
        if let Some(member) = room.members.get_mut(client_id) {
            // Free the prior color so a future distinct pick can reuse it,
            // then register the new custom pick so picks stay clear of it.
            room.colors.remove_occupied(member.color.srgb);
            room.colors.push_occupied(new_color);
            member.color = PlayerColor::custom(new_color);
        }

        Ok(())
    }

    pub fn client_room_mut(&mut self, client_id: &handle::Id) -> Result<&mut Room, StatusCode> {
        let room_id = self
            .clients
            .get(client_id)
            .ok_or(StatusCode::UNAUTHORIZED)?
            .room
            .clone();

        self.rooms.get_mut(&room_id).ok_or(StatusCode::NOT_FOUND)
    }

    pub(crate) fn move_client_to_room(
        &mut self,
        client_id: &handle::Id,
        new_room_id: &room::Id,
    ) -> Result<(), StatusCode> {
        let handle = self
            .clients
            .get(client_id)
            .ok_or(StatusCode::UNAUTHORIZED)?;

        if &handle.room == new_room_id {
            return Ok(());
        }

        // Validate new room exists before making any changes
        if !self.rooms.contains_key(new_room_id) {
            return Err(StatusCode::NOT_FOUND);
        }

        let username = handle.username.clone();
        let color = handle.color;
        let old_room_id = handle.room.clone();

        self.remove_from_room(client_id, &old_room_id);

        self.rooms
            .get_mut(new_room_id)
            .unwrap()
            .add_member(client_id, username, Some(color));

        self.clients.get_mut(client_id).unwrap().room = new_room_id.clone();

        Ok(())
    }

    pub(crate) fn remove_client(&mut self, client_id: &handle::Id) {
        let Some(handle) = self.clients.remove(client_id) else {
            return;
        };
        self.remove_from_room(client_id, &handle.room);
    }

    /// Insert a freshly created room with the given user as the first member.
    /// Returns the new room id and the color assigned to the member.
    pub(crate) fn insert_new_room(
        &mut self,
        location: Location,
        client_id: handle::Id,
        username: String,
        color_override: Option<Srgb8>,
    ) -> (room::Id, Srgb8) {
        let room_id = {
            let mut id = room::Id::random();
            while self.rooms.contains_key(&id) {
                id = room::Id::random();
            }
            id
        };

        tracing::info!(room_id = %room_id.as_ref(), username = %username, "created new room");

        let room = Room::new(
            location,
            client_id.clone(),
            username.clone(),
            color_override,
        );
        let color = room.members.get(&client_id).unwrap().color.srgb;

        self.rooms.insert(room_id.clone(), room);
        self.clients.insert(
            client_id,
            Handle {
                room: room_id.clone(),
                username,
                color,
            },
        );

        (room_id, color)
    }

    fn remove_from_room(&mut self, client_id: &handle::Id, room_id: &room::Id) {
        if let Some(room) = self.rooms.get_mut(room_id) {
            room.remove_member(client_id);
        }
    }

    /// If the room exists and is idle, drop it and remove any ghost clients
    /// still pointing at it.
    pub(crate) fn drop_room_if_idle(&mut self, room_id: &room::Id) {
        if !self.rooms.get(room_id).is_some_and(Room::is_idle) {
            return;
        }
        self.clients.retain(|_, h| &h.room != room_id);
        self.rooms.remove(room_id);
        tracing::info!(room_id = %room_id.as_ref(), "removed idle room");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::Coordinates;

    fn dummy_location() -> Location {
        Location {
            pano_id: "pano".to_string(),
            coordinates: Coordinates { lat: 0.0, lng: 0.0 },
        }
    }

    fn insert(model: &mut Model, name: &str) -> (handle::Id, room::Id) {
        let client_id = handle::Id::generate();
        let (room_id, _) =
            model.insert_new_room(dummy_location(), client_id.clone(), name.into(), None);
        (client_id, room_id)
    }

    #[test]
    fn new_room_is_idle() {
        let mut model = Model::default();
        let (_, room_id) = insert(&mut model, "canyon");
        assert!(model.rooms[&room_id].is_idle());
    }

    #[test]
    fn connect_then_disconnect_toggles_idle() {
        let mut model = Model::default();
        let (_, room_id) = insert(&mut model, "canyon");
        let room = model.rooms.get_mut(&room_id).unwrap();
        room.connect();
        assert!(!room.is_idle());
        room.disconnect();
        assert!(room.is_idle());
    }

    #[test]
    fn drop_if_idle_evicts_ghost_clients() {
        let mut model = Model::default();
        insert(&mut model, "canyon");
        let (_, room_id) = insert(&mut model, "adin");
        // Bob's room is idle (no SSE), holds Bob as a ghost member.
        model.drop_room_if_idle(&room_id);
        assert!(!model.rooms.contains_key(&room_id));
        // Alice's ghost still exists in her own (still-idle) room; only Bob
        // got cleaned up.
        assert_eq!(model.clients.len(), 1);
    }

    #[test]
    fn drop_if_idle_spares_live_room() {
        let mut model = Model::default();
        let (_, room_id) = insert(&mut model, "canyon");
        model.rooms.get_mut(&room_id).unwrap().connect();
        model.drop_room_if_idle(&room_id);
        assert!(model.rooms.contains_key(&room_id));
    }

    #[test]
    fn reinit_leaves_prior_room_idle_and_empty() {
        let mut model = Model::default();
        let client_id = handle::Id::generate();
        let (first, _) =
            model.insert_new_room(dummy_location(), client_id.clone(), "canyon".into(), None);

        model.remove_client(&client_id);
        let (second, _) = model.insert_new_room(dummy_location(), client_id, "canyon".into(), None);

        assert_ne!(first, second);
        let prior = &model.rooms[&first];
        assert!(prior.is_idle());
        assert!(prior.members.is_empty());
        assert_eq!(model.rooms[&second].members.len(), 1);
    }
}
