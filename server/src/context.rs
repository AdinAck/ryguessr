use std::{sync::Arc, time::Duration};

use axum::http::StatusCode;
use colors::Srgb8;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    Model, Room,
    colors::PlayerColor,
    event::PlayerData,
    geo::{Location, engine::LocationEngine},
    handle, room,
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
    pub api_key: Arc<str>,
}

impl Context {
    /// Create an empty context.
    pub fn new(engine: LocationEngine, google_maps_api_key: String) -> Self {
        Self {
            engine: Arc::new(engine),
            model: Arc::new(RwLock::new(Model::default())),
            api_key: Arc::from(google_maps_api_key),
        }
    }

    /// Handle the end of an SSE stream: drop the room's connection count,
    /// remove the client if they're still bound to this room, and arm
    /// cleanup if the room is now idle.
    pub async fn on_sse_disconnect(
        &self,
        client_id: &handle::Id,
        room_id: &room::Id,
        session: Uuid,
    ) {
        let mut model = self.model.write().await;

        if let Some(room) = model.rooms.get_mut(room_id) {
            room.disconnect();
        }

        let is_active_session = model
            .clients
            .get(client_id)
            .is_some_and(|h| &h.room == room_id && h.session == Some(session));

        if is_active_session {
            model.remove_client(client_id);
            tracing::info!(client_id = %**client_id, "client disconnected, removed from room");
        } else {
            tracing::debug!(client_id = %**client_id, "ignoring stale sse disconnect (already another session)");
        }

        if model.rooms.get(room_id).is_some_and(Room::is_idle) {
            self.start_cleanup(&mut model, room_id.clone());
        }
    }

    /// Move a client into an existing room. Re-arms the new room's cleanup
    /// timer if it has no live SSE connections. Returns the new client.
    pub async fn move_client_to_room(
        &self,
        client_id: &handle::Id,
        new_room_id: &room::Id,
    ) -> Result<PlayerData, StatusCode> {
        let mut model = self.model.write().await;
        model.move_client_to_room(client_id, new_room_id)?;

        if model.rooms.get(new_room_id).is_some_and(Room::is_idle) {
            self.start_cleanup(&mut model, new_room_id.clone());
        }

        let member = model
            .rooms
            .get(new_room_id)
            .and_then(|r| r.members.get(client_id))
            .expect("client is in the room after a successful move");

        Ok(member.clone().into())
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
    ) -> (room::Id, String, PlayerColor) {
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

    pub async fn get_room_players(
        &self,
        room_id: &room::Id,
    ) -> Result<Vec<PlayerData>, StatusCode> {
        let model = self.model.read().await;
        Ok(model.room(room_id)?.players())
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
