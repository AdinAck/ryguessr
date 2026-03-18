use axum::http::StatusCode;
use axum::{Json, extract::State};
use axum_extra::TypedHeader;

use crate::{Context, handle, room};

#[derive(serde::Deserialize, Default)]
pub struct InitRequest {
    username: Option<String>,
    color: Option<String>,
}

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
    Json(request): Json<InitRequest>,
) -> Result<Json<InitResponse>, StatusCode> {
    let location = context
        .engine
        .get_random_location()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut model = context.model.write().await;

    let username = request
        .username
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| model.generate_unique_name());

    let (room_id, color) =
        model.create_room(location, client_id.clone(), username.clone(), request.color);

    Ok(Json(InitResponse {
        room_id,
        username,
        color,
    }))
}
