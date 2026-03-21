use axum::extract::State;
use axum::http::StatusCode;

use crate::{Context, handle};

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn next_handler(
    State(context): State<Context>,
    client_id: handle::Id,
) -> Result<(), StatusCode> {
    let start_new_round = {
        let mut model = context.model.write().await;
        let room = model.client_room_mut(&client_id)?;
        room.submit_ready(&client_id)
    };

    if start_new_round {
        let new_location = context.engine.get_random_location().await.map_err(|e| {
            tracing::error!(%e, "failed to get new location for next round");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let mut model = context.model.write().await;
        if let Ok(room) = model.client_room_mut(&client_id) {
            room.start_next_round(new_location);
        }
    }

    Ok(())
}
