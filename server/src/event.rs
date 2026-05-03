use std::collections::HashMap;

use crate::{Coordinates, geo::PanoId};
use axum::response::sse;
use colors::Srgb8;
use serde::Serialize;
use strum::AsRefStr;

type Username = String;

#[derive(Debug, Clone, Serialize, AsRefStr)]
#[serde(untagged)]
#[strum(serialize_all = "kebab-case")]
pub enum RoomEvent {
    RoundStart(RoundStartData),
    RoundEnd(RoundEndData),
    PlayerJoined(JoinData),
    PlayerLeft { username: Username },
}

impl TryFrom<RoomEvent> for sse::Event {
    type Error = axum::Error;

    fn try_from(event: RoomEvent) -> Result<Self, Self::Error> {
        sse::Event::default().event(event.as_ref()).json_data(event)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RoundStartData {
    pub pano_id: PanoId,
    pub round: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoundEndData {
    pub real_location: Coordinates,
    pub player_results: HashMap<Username, PlayerResults>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerResults {
    pub last_score: u32,
    pub cum_score: u32,
    pub distance: f64,
    pub guess_location: Coordinates,
    pub color: Srgb8,
}

#[derive(Debug, Clone, Serialize)]
pub struct JoinData {
    pub username: String,
    pub color: Srgb8,
}
