use std::collections::HashMap;

use crate::{Coordinates, geo::PanoId};
use axum::response::sse;
use serde::Serialize;

type PlayerName = String;

#[derive(Clone, Serialize)]
pub enum RoomEvent {
    RoundStart(PanoId),
    RoundEnd(RoundData),
}

impl RoomEvent {
    pub fn name(&self) -> &'static str {
        match self {
            RoomEvent::RoundStart(_) => "round_start",
            RoomEvent::RoundEnd(_) => "round_end",
        }
    }
}

impl TryFrom<RoomEvent> for sse::Event {
    type Error = axum::Error;

    fn try_from(event: RoomEvent) -> Result<Self, Self::Error> {
        sse::Event::default().event(event.name()).json_data(event)
    }
}

#[derive(Clone, Serialize)]
pub struct RoundData {
    pub real_location: Coordinates,
    pub player_results: HashMap<PlayerName, PlayerResults>,
}

#[derive(Clone, Serialize)]
pub struct PlayerResults {
    pub last_score: u32,
    pub cum_score: u32,
    pub distance: f64,
    pub guess_location: Coordinates,
}
