use axum::{Json, extract::State};
use colors::Srgb8;

use crate::{AppError, Context, handle};

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn color_handler(
    State(context): State<Context>,
    client_id: handle::Id,
    Json(color): Json<Srgb8>,
) -> Result<(), AppError> {
    let mut model = context.model.write().await;

    model.set_color(&client_id, color)
}
