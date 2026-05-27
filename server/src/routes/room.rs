use anyhow::Result;
use axum::{
    Json,
    extract::{Path, State},
};
use reqwest::StatusCode;

use crate::{Context, event::PlayerData, room};

pub async fn room_handler(
    State(context): State<Context>,
    Path(room_id): Path<room::Id>,
) -> Result<Json<Vec<PlayerData>>, StatusCode> {
    let players = context.get_room_players(&room_id).await?;
    Ok(Json(players))
}
