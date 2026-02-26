use std::collections::HashMap;

use crate::{Handle, Room, geo::Location, handle, room};

/// The context available to the API surface to facilitate the ryguessr services.
pub struct Context {
    pub clients: HashMap<handle::Id, Handle>,
    pub rooms: HashMap<room::Id, Room>,
}

impl Context {
    /// Create an empty context.
    pub fn empty() -> Self {
        Self {
            clients: Default::default(),
            rooms: Default::default(),
        }
    }

    /// Remove a client from its room and from the client list, cleaning up empty rooms.
    pub fn remove_client(&mut self, client_id: &handle::Id) {
        let Some(handle) = self.clients.remove(client_id) else {
            return;
        };
        let room_id = &handle.room;
        if let Some(room) = self.rooms.get_mut(room_id) {
            room.remove_member(client_id);
            if room.members.is_empty() {
                tracing::info!(room_id = %room_id.as_ref(), "room empty, cleaning up...");
                self.rooms.remove(room_id);
            }
        }
    }

    pub fn create_room_with_user(
        &mut self,
        location: Location,
        client_id: handle::Id,
        username: String,
    ) -> room::Id {
        // Generate room Id (ensure no collisions)
        let room_id = {
            let mut id = room::Id::random();
            while self.rooms.contains_key(&id) {
                id = room::Id::random();
            }
            id
        };

        // Create room + handle
        let room = Room::new(location, client_id.clone(), username.clone());
        self.rooms.insert(room_id.clone(), room);
        self.clients.insert(
            client_id,
            Handle {
                room: room_id.clone(),
                username,
            },
        );

        room_id
    }
}
