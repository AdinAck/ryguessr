use std::collections::HashMap;

use crate::geo::{Coordinates, PanoId};
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

#[derive(Clone, Serialize)]
struct RoundData {
    real_location: Coordinates,
    player_results: HashMap<PlayerName, PlayerResults>,
}

#[derive(Clone, Serialize)]
struct PlayerResults {
    last_score: u32,
    cum_score: u32,
    distance: f64,
    guess_location: Coordinates,
}
