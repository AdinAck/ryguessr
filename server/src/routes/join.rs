use axum::http::StatusCode;
use axum::{Json, extract::State};
use serde::Serialize;

use crate::event::{PlayerData, RoundStartData};
use crate::{Context, handle, room};

#[derive(Serialize)]
pub struct JoinResponse {
    player_data: Vec<PlayerData>,
    round_data: RoundStartData,
}

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn join_handler(
    State(context): State<Context>,
    client_id: handle::Id,
    Json(room_id): Json<room::Id>,
) -> Result<Json<JoinResponse>, StatusCode> {
    let (players, round_data) = context.move_client_to_room(&client_id, &room_id).await?;

    Ok(Json(JoinResponse {
        player_data: players,
        round_data,
    }))
}
