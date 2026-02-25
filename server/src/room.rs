use std::collections::HashMap;

use derive_more::{AsRef, Deref};
use tokio::sync::broadcast;

use crate::{
    Coordinates, RoomEvent,
    event::{PlayerResults, RoundData},
    geo::{Location, engine::LocationEngine},
    handle, score,
};

/// A room exists in a particular [`Location`], housing multiple members who are all
/// at the room's location. A room's [`Config`] specifies the rules of how the room
/// behaves, or the constraints applied to / advantages given to certain members.
pub struct Room {
    /// A map corresponding the identifiers of each member to their local
    /// [attributes](MemberAttributes).
    pub members: HashMap<handle::Id, MemberAttributes>,
    /// The room's current [`Location`].
    pub location: Location,
    /// The room's current [configuration](Config).
    // Currently unused, but will be used to implement different game modes and constraints in the future.
    #[allow(dead_code)]
    config: Config,
    /// The event sender handle for the room, used to broadcast events to all members of the room.
    pub event_tx: broadcast::Sender<RoomEvent>,
    // TODO: location history for the round
}

impl Room {
    pub fn new(location: Location, client_id: handle::Id, username: String) -> Self {
        let (event_tx, _) = broadcast::channel(16);
        let members = HashMap::from([(
            client_id,
            MemberAttributes {
                username,
                score: 0,
                guess: None,
                ready_next_round: false,
            },
        )]);
        Self {
            members,
            location,
            config: Config {},
            event_tx,
        }
    }

    /// Handle a ready submission from a member of the room. This will broadcast if everyone is ready to move on to the next round.
    pub async fn submit_ready(&mut self, client_id: &handle::Id, engine: &LocationEngine) {
        let Some(member) = self.members.get_mut(client_id) else {
            return;
        };

        member.ready_next_round = true;

        if !self.members.values().all(|m| m.ready_next_round) {
            return;
        };

        // Everyone is ready, start the next round
        let new_location = match engine.get_random_location().await {
            Ok(loc) => loc,
            Err(e) => {
                tracing::error!(%e, "failed to get new location for next round");
                return;
            }
        };
        let pano_id = new_location.pano_id.clone();
        self.location = new_location;

        for member in self.members.values_mut() {
            member.ready_next_round = false;
        }

        let _ = self.event_tx.send(RoomEvent::RoundStart(pano_id));
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
                    let last_score = score::calculate_score(distance) as u32;
                    (
                        m.username.clone(),
                        PlayerResults {
                            last_score,
                            cum_score: m.score,
                            distance,
                            guess_location,
                        },
                    )
                })
                .collect();

            let event = RoomEvent::RoundEnd(RoundData {
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
    /// Whether the member is ready to move on to the next round.
    pub ready_next_round: bool,
}

/// The unique identifier for a [`Room`].
#[derive(Clone, AsRef, Deref, Hash, PartialEq, Eq, serde::Serialize)]
pub struct Id(String);

impl Id {
    /// The characters that are allowed to be used in a random [`Id`].
    /// The characters 0, 1, O, and I are excluded to avoid confusion.
    const ALLOWED_CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

    /// Generate a random 4 character human readable [`Id`] for a new [`Room`].
    pub fn random() -> Self {
        (0..4)
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
