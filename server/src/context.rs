use std::collections::HashMap;

use crate::{Handle, Room, handle, room};

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
}
