use std::sync::Arc;

use axum::{Json, extract::State};
use axum_extra::TypedHeader;
use reqwest::StatusCode;
use tokio::sync::RwLock;

use crate::{Context, geo::engine::LocationEngine, handle, room};

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn join_handler(
    State((context, _)): State<(Arc<RwLock<Context>>, Arc<LocationEngine>)>,
    TypedHeader(client_id): TypedHeader<handle::Id>,
    Json(room_id): Json<room::Id>,
) -> StatusCode {
    let mut cx = context.write().await;

    // Remove client from old room
    let (username, old_room_id) = match cx.clients.get(&client_id) {
        Some(h) => (h.username.clone(), h.room.clone()),
        None => return StatusCode::UNAUTHORIZED,
    };
    let old_room = match cx.rooms.get_mut(&old_room_id) {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND,
    };
    old_room.remove_member(&client_id);

    // Add client to new room
    let new_room = match cx.rooms.get_mut(&room_id) {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND,
    };
    new_room.add_member(&client_id, username);

    StatusCode::OK
}
