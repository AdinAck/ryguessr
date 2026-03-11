use axum::http::StatusCode;
use axum::{Json, extract::State};
use axum_extra::TypedHeader;

use crate::{Context, handle, room};

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn init_handler(
    State(context): State<Context>,
    TypedHeader(client_id): TypedHeader<handle::Id>,
    Json(username): Json<String>,
) -> Result<Json<room::Id>, StatusCode> {
    let location = context
        .engine
        .get_random_location()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut model = context.model.write().await;
    let room_id = model.create_room_with_user(location, client_id, username);

    Ok(Json(room_id))
}
