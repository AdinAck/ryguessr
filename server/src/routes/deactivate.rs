use axum::extract::State;

use crate::{AppError, Context, handle};

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn deactivate_handler(
    State(context): State<Context>,
    client_id: handle::Id,
) -> Result<(), AppError> {
    context
        .model
        .write()
        .await
        .client_room_mut(&client_id)?
        .deactivate();

    Ok(())
}
