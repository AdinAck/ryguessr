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
    PlayerJoined(PlayerData),
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
    pub player_results: Vec<PlayerResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerResult {
    pub player: PlayerData,
    pub round_score: u32,
    pub distance: f64,
    pub guess_location: Coordinates,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerData {
    pub username: Username,
    pub color: Srgb8,
    pub score: u32,
}
