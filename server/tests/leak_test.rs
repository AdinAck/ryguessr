use ryguessr::{
    Model,
    geo::{Coordinates, Location},
    handle,
};
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_room_leak_on_reinit() {
    let mut model = Model::default();
    let client_id = handle::Id::generate();
    let location = Location {
        pano_id: "pano".to_string(),
        coordinates: Coordinates { lat: 0.0, lng: 0.0 },
    };

    model.create_room(
        location.clone(),
        client_id.clone(),
        "test".to_string(),
        None,
    );
    assert_eq!(model.rooms.len(), 1, "Should have 1 room initially");

    // Re-init with same client_id
    model.create_room(location, client_id.clone(), "test".to_string(), None);

    // If it leaks, this will be 2
    assert_eq!(
        model.rooms.len(),
        1,
        "Should still have only 1 room, old one should be cleaned up"
    );
}

#[tokio::test]
async fn test_stale_room_cleanup() {
    let mut model = Model::default();
    let client_id = handle::Id::generate();
    let location = Location {
        pano_id: "pano".to_string(),
        coordinates: Coordinates { lat: 0.0, lng: 0.0 },
    };

    let (room_id, _) = model.create_room(location, client_id.clone(), "test".to_string(), None);
    assert_eq!(model.rooms.len(), 1);
    assert_eq!(model.clients.len(), 1);

    // Manually set last_activity to 61 seconds ago
    if let Some(room) = model.rooms.get_mut(&room_id) {
        room.last_activity = Instant::now() - Duration::from_secs(61);
    }

    // Cleanup should remove it
    model.cleanup_stale_rooms();

    assert_eq!(model.rooms.len(), 0, "Stale room should be cleaned up");
    assert_eq!(
        model.clients.len(),
        0,
        "Client associated with stale room should be cleaned up"
    );
}

#[tokio::test]
async fn test_active_room_not_cleaned_up() {
    let mut model = Model::default();
    let client_id = handle::Id::generate();
    let location = Location {
        pano_id: "pano".to_string(),
        coordinates: Coordinates { lat: 0.0, lng: 0.0 },
    };

    let (room_id, _) = model.create_room(location, client_id.clone(), "test".to_string(), None);

    // Set active connection and old last_activity
    if let Some(room) = model.rooms.get_mut(&room_id) {
        room.increment_connection();
        room.last_activity = Instant::now() - Duration::from_secs(61);
    }

    // Cleanup should NOT remove it
    model.cleanup_stale_rooms();

    assert_eq!(model.rooms.len(), 1, "Active room should NOT be cleaned up");
}
