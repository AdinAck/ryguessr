use axum::{Json, extract::State, http::StatusCode};

use crate::{Context, handle};

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn color_handler(
    State(context): State<Context>,
    client_id: handle::Id,
    Json(color_hex): Json<String>,
) -> Result<(), StatusCode> {
    let mut model = context.model.write().await;

    model.set_color(&client_id, color_hex)
}
