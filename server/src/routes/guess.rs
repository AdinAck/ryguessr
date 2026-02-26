use std::sync::Arc;

use axum::{Json, extract::State};
use axum_extra::TypedHeader;
use reqwest::StatusCode;
use tokio::sync::RwLock;

use crate::{Context, Coordinates, geo::engine::LocationEngine, handle};

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn guess_handler(
    State((context, _)): State<(Arc<RwLock<Context>>, Arc<LocationEngine>)>,
    TypedHeader(client_id): TypedHeader<handle::Id>,
    Json(guess): Json<Coordinates>,
) -> StatusCode {
    let mut cx = context.write().await;
    let room_id = match cx.clients.get(&client_id) {
        Some(h) => h.room.clone(),
        None => return StatusCode::UNAUTHORIZED,
    };
    let room = match cx.rooms.get_mut(&room_id) {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND,
    };

    room.submit_guess(&client_id, guess);

    StatusCode::OK
}
