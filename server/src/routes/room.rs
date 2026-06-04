use axum::{
    Json,
    extract::{Path, State},
};

use crate::{AppError, Context, event::PlayerData, room};

#[tracing::instrument(skip_all, fields(room_id = %*room_id))]
pub async fn room_handler(
    State(context): State<Context>,
    Path(room_id): Path<room::Id>,
) -> Result<Json<Vec<PlayerData>>, AppError> {
    let players = context.get_room_players(&room_id).await?;
    Ok(Json(players))
}
