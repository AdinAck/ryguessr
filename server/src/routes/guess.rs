use axum::http::StatusCode;
use axum::{Json, extract::State};
use axum_extra::TypedHeader;

use crate::{Context, Coordinates, handle};

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn guess_handler(
    State(context): State<Context>,
    TypedHeader(client_id): TypedHeader<handle::Id>,
    Json(guess): Json<Coordinates>,
) -> Result<(), StatusCode> {
    let mut model = context.model.write().await;

    let room = model.client_room_mut(&client_id)?;
    room.submit_guess(&client_id, guess);

    Ok(())
}
