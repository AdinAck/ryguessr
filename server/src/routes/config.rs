use axum::{Json, extract::State, http::StatusCode};

use crate::{Context, handle, room};

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn config_handler(
    State(context): State<Context>,
    client_id: handle::Id,
    Json(config): Json<room::Config>,
) -> Result<(), StatusCode> {
    let mut model = context.model.write().await;

    model.client_room_mut(&client_id)?.set_config(config);

    Ok(())
}
