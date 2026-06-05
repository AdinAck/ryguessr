use std::collections::HashSet;

use crate::{Coordinates, geo::PanoId, handle};
use axum::response::sse;
use colors::Srgb8;
use serde::Serialize;
use strum::AsRefStr;

type Username = String;

#[derive(Debug, Clone)]
pub enum Recipients {
    All,
    Only(HashSet<handle::Id>),
    Except(HashSet<handle::Id>),
}

impl Recipients {
    pub fn except_one(id: handle::Id) -> Self {
        Self::Except(HashSet::from([id]))
    }

    pub fn only_one(id: handle::Id) -> Self {
        Self::Only(HashSet::from([id]))
    }
}

#[derive(Debug, Clone)]
pub struct EventEnvelope {
    pub event: RoomEvent,
    pub recipients: Recipients,
}

impl From<RoomEvent> for EventEnvelope {
    fn from(event: RoomEvent) -> Self {
        let recipients = event.recipients();
        Self { event, recipients }
    }
}

#[derive(Debug, Clone, Serialize, AsRefStr)]
#[serde(untagged)]
#[strum(serialize_all = "kebab-case")]
pub enum RoomEvent {
    RoundStart(RoundStartData),
    RoundEnd(RoundEndData),
    PlayerJoined {
        #[serde(skip)]
        client_id: handle::Id,
        #[serde(flatten)]
        data: PlayerData,
    },
    PlayerLeft {
        #[serde(skip)]
        client_id: handle::Id,
        username: Username,
    },
}

impl RoomEvent {
    /// The delivery policy is a property of the event variant
    pub fn recipients(&self) -> Recipients {
        match self {
            Self::PlayerJoined { client_id, .. } | Self::PlayerLeft { client_id, .. } => {
                Recipients::except_one(client_id.clone())
            }
            Self::RoundStart(_) | Self::RoundEnd(_) => Recipients::All,
        }
    }
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
