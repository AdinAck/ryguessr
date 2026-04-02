use std::{collections::HashMap, sync::Arc};

use axum::http::StatusCode;
use tokio::sync::RwLock;

use crate::{
    Handle, Room, colors,
    geo::{Location, engine::LocationEngine},
    handle,
    name_gen::NameGenerator,
    room,
};

pub type SharedModel = Arc<RwLock<Model>>;

/// The context available to the API surface to facilitate the ryguessr services.
#[derive(Clone)]
pub struct Context {
    pub engine: Arc<LocationEngine>,
    pub model: SharedModel,
}

#[derive(Default)]
pub struct Model {
    pub clients: HashMap<handle::Id, Handle>,
    pub rooms: HashMap<room::Id, Room>,
    pub name_generator: NameGenerator,
}

impl Context {
    /// Create an empty context.
    pub fn new(engine: LocationEngine) -> Self {
        let model = Arc::new(RwLock::new(Model::default()));

        // Spawn a background task to periodically clean up stale rooms.
        let model_clone = model.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                let mut model = model_clone.write().await;
                model.cleanup_stale_rooms();
            }
        });

        Self {
            engine: Arc::new(engine),
            model,
        }
    }
}

impl Model {
    pub fn cleanup_stale_rooms(&mut self) {
        tracing::debug!(
            "running stale room cleanup ({} rooms active)",
            self.rooms.len()
        );
        let stale_room_ids: Vec<_> = self
            .rooms
            .iter()
            .filter(|(_, room)| room.is_stale())
            .map(|(id, _)| id.clone())
            .collect();

        for id in stale_room_ids {
            tracing::info!(room_id = %id.as_ref(), "cleaning up stale room");
            if let Some(_room) = self.rooms.remove(&id) {
                // Also remove all clients that were in this room
                let client_ids: Vec<_> = self
                    .clients
                    .iter()
                    .filter(|(_, handle)| handle.room == id)
                    .map(|(id, _)| id.clone())
                    .collect();
                for client_id in client_ids {
                    tracing::debug!(client_id = %*client_id, "removing client from stale room");
                    self.clients.remove(&client_id);
                }
            }
        }
    }

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
        new_color: String,
    ) -> Result<(), StatusCode> {
        let handle = self
            .clients
            .get_mut(client_id)
            .ok_or(StatusCode::UNAUTHORIZED)?;
        handle.color = new_color.clone();

        // Update the member's color in their current room
        let room = self
            .rooms
            .get_mut(&handle.room)
            .ok_or(StatusCode::NOT_FOUND)?;
        if let Some(member) = room.members.get_mut(client_id) {
            if let colors::MemberColor::Distinct { index, .. } = member.color {
                // If the member previously had a distinct color, return the index to the pool.
                room.colors.return_index(index);
            }

            member.color = colors::MemberColor::Custom(new_color);
        }

        Ok(())
    }

    pub fn move_client_to_room(
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
        let color = handle.color.clone();
        let old_room_id = handle.room.clone();

        // Remove client from old room
        self.remove_from_room(client_id, &old_room_id);

        // Add client to new room
        self.rooms
            .get_mut(new_room_id)
            .unwrap()
            .add_member(client_id, username, Some(color));

        // Update client's handle
        self.clients.get_mut(client_id).unwrap().room = new_room_id.clone();

        Ok(())
    }

    /// Remove a client from its room and from the client list, cleaning up empty rooms.
    pub fn remove_client(&mut self, client_id: &handle::Id) {
        let Some(handle) = self.clients.remove(client_id) else {
            return;
        };
        self.remove_from_room(client_id, &handle.room);
    }

    /// Create a new room with the given user as the first member, returning the room Id and assigned color.
    pub fn create_room(
        &mut self,
        location: Location,
        client_id: handle::Id,
        username: String,
        color_override: Option<String>,
    ) -> (room::Id, String) {
        // Ensure the client is removed from any existing rooms before creating a new one.
        self.remove_client(&client_id);

        // Generate room Id (ensure no collisions)
        let room_id = {
            let mut id = room::Id::random();
            while self.rooms.contains_key(&id) {
                id = room::Id::random();
            }
            id
        };

        tracing::info!(room_id = %room_id.as_ref(), username = %username, "created new room");

        // Create room + handle
        let room = Room::new(
            location,
            client_id.clone(),
            username.clone(),
            color_override,
        );
        let color = room.members.get(&client_id).unwrap().color.clone();

        self.rooms.insert(room_id.clone(), room);
        self.clients.insert(
            client_id.clone(),
            Handle {
                room: room_id.clone(),
                username,
                color: color.clone().into(),
            },
        );

        (room_id, color.into())
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

    fn remove_from_room(&mut self, client_id: &handle::Id, room_id: &room::Id) {
        if let Some(room) = self.rooms.get_mut(room_id) {
            room.remove_member(client_id);
            if room.members.is_empty() {
                tracing::info!(room_id = %room_id.as_ref(), "room empty, cleaning up...");
                self.rooms.remove(room_id);
            }
        }
    }
}
