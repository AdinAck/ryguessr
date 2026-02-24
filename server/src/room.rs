use std::collections::HashMap;

use derive_more::{AsRef, Deref};
use tokio::sync::broadcast;

use crate::{RoomEvent, geo::Location, handle};

/// A room exists in a particular [`Location`], housing multiple members who are all
/// at the room's location. A room's [`Config`] specifies the rules of how the room
/// behaves, or the constraints applied to / advantages given to certain members.
pub struct Room {
    /// A map corresponding the identifiers of each member to their local
    /// [attributes](MemberAttributes).
    members: HashMap<handle::Id, MemberAttributes>,
    /// The room's current [`Location`].
    location: Location,
    /// The room's current [configuration](Config).
    config: Config,
    /// The event sender handle for the room, used to broadcast events to all members of the room.
    pub event_tx: broadcast::Sender<RoomEvent>,
    // TODO: location history for the round
}

/// The attributes of a member of a [`Room`].
pub struct MemberAttributes {
    /// The current score of the member in the [`Room`].
    score: u16,
}

/// The unique identifier for a [`Room`].
#[derive(AsRef, Deref, Hash, PartialEq, Eq)]
pub struct Id(String);

/// The configuration of a [`Room`].
pub struct Config {}
