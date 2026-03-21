use axum::http::StatusCode;
use axum::{Json, extract::State};
use axum_extra::TypedHeader;

use crate::{Context, handle, room};

#[derive(serde::Serialize)]
pub struct InitResponse {
    room_id: room::Id,
    username: String,
    color: String,
}

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn init_handler(
    State(context): State<Context>,
    TypedHeader(client_id): TypedHeader<handle::Id>,
) -> Result<Json<InitResponse>, StatusCode> {
    let location = context
        .engine
        .get_random_location()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut model = context.model.write().await;
    let username = model.generate_unique_name();
    let (room_id, color) = model.create_room(location, client_id.clone(), username.clone());

    Ok(Json(InitResponse {
        room_id,
        username,
        color,
    }))
}
