use axum::{Json, extract::State};

use crate::event::PlayerData;
use crate::{AppError, Context, handle, room};

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn join_handler(
    State(context): State<Context>,
    client_id: handle::Id,
    Json(room_id): Json<room::Id>,
) -> Result<Json<Vec<PlayerData>>, AppError> {
    let players = context.move_client_to_room(&client_id, &room_id).await?;

    Ok(Json(players))
}
