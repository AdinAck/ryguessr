use std::{collections::HashMap, panic::Location};

use axum::response::sse;
use tokio::sync::mpsc;

use crate::{Handle, Room, geo::engine::LocationEngine, handle, room};

/// The context available to the API surface to facilitate the ryguessr services.
pub struct Context {
    location_engine: LocationEngine,

    pub clients: HashMap<handle::Id, Handle>,
    pub rooms: HashMap<room::Id, Room>,
}

impl Context {
    /// Create an empty context.
    pub fn empty(location_engine: LocationEngine) -> Self {
        Self {
            location_engine,
            clients: Default::default(),
            rooms: Default::default(),
        }
    }
}
