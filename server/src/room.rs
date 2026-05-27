use std::collections::HashMap;

use colors::{Srgb8, srgb};
use derive_more::{AsRef, Deref};
use serde::{Deserialize, Serialize};
use tokio::{sync::broadcast, task::JoinHandle};

use crate::{
    Coordinates, RoomEvent,
    colors::{DistinctColors, PlayerColor},
    event::{PlayerData, PlayerResult, RoundEndData, RoundStartData},
    geo::Location,
    handle, score,
};

/// The length of a [`Room::Id`] identifier string.
const ROOM_ID_LENGTH: usize = 4;

/// Used when [`DistinctColors`] is saturated and can't produce a fresh
/// distinct color. (Rare)
const FALLBACK_COLOR: Srgb8 = srgb!("#808080");

/// A room exists in a particular [`Location`], housing multiple members who are all
/// at the room's location. A room's [`Config`] specifies the rules of how the room
/// behaves, or the constraints applied to / advantages given to certain members.
pub struct Room {
    /// A map corresponding the identifiers of each member to their local
    /// [attributes](MemberAttributes).
    pub members: HashMap<handle::Id, MemberAttributes>,
    /// The number of rounds played
    pub round: usize,
    /// The room's current [`Location`].
    pub location: Location,
    /// A generator for distinct colors to assign to members of the room for frontend display purposes. The same color will be assigned to the same member across rounds, but different members will have different colors.
    pub colors: DistinctColors,
    /// The room's current [configuration](Config).
    // Currently unused, but will be used to implement different game modes and constraints in the future.
    #[allow(dead_code)]
    config: Config,
    /// The event sender handle for the room, used to broadcast events to all members of the room.
    pub event_tx: broadcast::Sender<RoomEvent>,
    /// Number of live SSE connections subscribed to this room.
    active_connections: usize,
    /// Handle to a cleanup task scheduled while the room is idle.
    cleanup_handle: Option<JoinHandle<()>>,
    // TODO: location history for the round
}

impl Room {
    pub fn new(
        location: Location,
        client_id: handle::Id,
        username: String,
        color_override: Option<Srgb8>,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(16);
        let mut colors = DistinctColors::new();
        let color = match color_override {
            Some(srgb) => {
                colors.push_occupied(srgb);
                PlayerColor::custom(srgb)
            }
            None => colors.next().unwrap_or_else(|| {
                tracing::warn!("DistinctColors exhausted; using fallback");
                PlayerColor::distinct(FALLBACK_COLOR)
            }),
        };
        let members = HashMap::from([(client_id, MemberAttributes::new(username, color))]);
        Self {
            members,
            round: 0,
            location,
            colors,
            config: Config {},
            event_tx,
            active_connections: 0,
            cleanup_handle: None,
        }
    }

    /// Register a new live SSE connection. Aborts any pending cleanup since
    /// the room is no longer idle.
    pub fn connect(&mut self) {
        self.active_connections += 1;
        if let Some(prev) = self.cleanup_handle.take() {
            prev.abort();
        }
    }

    /// Unregister a live SSE connection. The caller is responsible for starting
    /// cleanup if the room is now idle.
    pub fn disconnect(&mut self) {
        self.active_connections = self.active_connections.saturating_sub(1);
    }

    /// A room is idle when no live SSE connection is subscribed to it.
    pub fn is_idle(&self) -> bool {
        self.active_connections == 0
    }

    /// Store the handle for a pending cleanup task, aborting any prior one so
    /// only the freshest timer is ever live.
    pub(crate) fn arm_cleanup(&mut self, handle: JoinHandle<()>) {
        if let Some(prev) = self.cleanup_handle.replace(handle) {
            prev.abort();
        }
    }

    /// Add a member to the room. If `color` is not provided, a new distinct color is generated.
    pub fn add_member(
        &mut self,
        client_id: &handle::Id,
        username: String,
        color_override: Option<Srgb8>,
    ) {
        let color = match color_override {
            Some(srgb) => {
                self.colors.push_occupied(srgb);
                PlayerColor::custom(srgb)
            }
            None => self.colors.next().unwrap_or_else(|| {
                tracing::warn!("DistinctColors exhausted; using fallback");
                PlayerColor::distinct(FALLBACK_COLOR)
            }),
        };

        let new_member = MemberAttributes::new(username.clone(), color);

        let event = RoomEvent::PlayerJoined(PlayerData::from(&new_member));

        self.members.insert(client_id.clone(), new_member);

        // Broadcast join event
        let _ = self.event_tx.send(event);
    }

    /// Snapshot of every member as `PlayerData`, suitable for sending to a
    /// client that needs to render the current roster.
    pub fn players(&self) -> Vec<PlayerData> {
        self.members.values().map(PlayerData::from).collect()
    }

    pub fn remove_member(&mut self, client_id: &handle::Id) {
        let Some(member) = self.members.remove(client_id) else {
            return;
        };
        self.colors.remove_occupied(member.color.srgb);

        // Broadcast leave event
        let _ = self.event_tx.send(RoomEvent::PlayerLeft {
            username: member.username,
        });
    }

    /// Handle a ready submission from a member of the room.
    /// Returns true if everyone is ready.
    pub fn submit_ready(&mut self, client_id: &handle::Id) -> bool {
        let Some(member) = self.members.get_mut(client_id) else {
            return false;
        };

        member.ready_next_round = true;

        self.members.values().all(|m| m.ready_next_round)
    }

    /// Transition to next round
    pub fn start_next_round(&mut self, new_location: Location) {
        let pano_id = new_location.pano_id.clone();
        self.location = new_location;

        // Reset ready status for next round
        for member in self.members.values_mut() {
            member.ready_next_round = false;
        }

        self.round += 1;
        let round = self.round;
        let _ = self
            .event_tx
            .send(RoomEvent::RoundStart(RoundStartData { pano_id, round }));
    }

    /// Handle a guess from a member of the room. This will update the member's score and broadcast
    /// if everyone has submitted a guess for the current round.
    pub fn submit_guess(&mut self, client_id: &handle::Id, guess: Coordinates) {
        let Some(member) = self.members.get_mut(client_id) else {
            return;
        };

        let distance = score::haversine_distance(&guess, &self.location.coordinates);
        let points = score::calculate_score(distance);

        member.score += points as u32;
        member.guess = Some(guess);

        // Check if everyone has guessed
        let all_guessed = self.members.values().all(|m| m.guess.is_some());
        if all_guessed {
            let player_results = self
                .members
                .values()
                .map(|m| {
                    let guess_location = m.guess.clone().unwrap(); // Safe unwrap()
                    let distance =
                        score::haversine_distance(&guess_location, &self.location.coordinates);
                    let round_score = score::calculate_score(distance) as u32;
                    PlayerResult {
                        player: PlayerData::from(m),
                        round_score,
                        distance,
                        guess_location,
                    }
                })
                .collect();

            let event = RoomEvent::RoundEnd(RoundEndData {
                real_location: self.location.coordinates.clone(),
                player_results,
            });
            // Broadcast round end event to all members of the room.
            let _ = self.event_tx.send(event);

            // Reset guesses for next round
            for member in self.members.values_mut() {
                member.guess = None;
            }
        }
    }
}

/// The attributes of a member of a [`Room`].
pub struct MemberAttributes {
    /// The display name of the member.
    pub username: String,
    /// The current score of the member in the [`Room`].
    pub score: u32,
    /// The member's most recent guess for the current round, if they have submitted one.
    pub guess: Option<Coordinates>,
    /// The color assigned to the member for frontend display purposes.
    pub color: PlayerColor,
    /// Whether the member is ready to move on to the next round.
    pub ready_next_round: bool,
}

impl MemberAttributes {
    pub fn new(username: String, color: PlayerColor) -> Self {
        Self {
            username,
            score: 0,
            guess: None,
            color,
            ready_next_round: false,
        }
    }
}

impl From<&MemberAttributes> for PlayerData {
    fn from(m: &MemberAttributes) -> Self {
        Self {
            username: m.username.clone(),
            color: m.color.srgb,
            score: m.score,
        }
    }
}

/// The unique identifier for a [`Room`].
#[derive(Clone, AsRef, Debug, Deref, Hash, PartialEq, Eq, Serialize)]
pub struct Id(String);

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let len = s.chars().count();
        if len != ROOM_ID_LENGTH {
            return Err(serde::de::Error::invalid_length(
                s.chars().count(),
                &"a string of exactly 5 characters",
            ));
        }
        Ok(Self(s))
    }
}

impl Id {
    /// The characters that are allowed to be used in a random [`Id`].
    /// The characters 0, 1, O, and I are excluded to avoid confusion.
    const ALLOWED_CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

    /// Generate a random 4 character human readable [`Id`] for a new [`Room`].
    pub fn random() -> Self {
        (0..ROOM_ID_LENGTH)
            .map(|_| Self::ALLOWED_CHARS[rand::random_range(0..Self::ALLOWED_CHARS.len())] as char)
            .collect()
    }
}

impl FromIterator<char> for Id {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

/// The configuration of a [`Room`].
pub struct Config {}
