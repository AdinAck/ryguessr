use axum::{Json, extract::State, http::StatusCode};

use crate::{Context, handle};

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn username_handler(
    State(context): State<Context>,
    client_id: handle::Id,
    Json(username): Json<String>,
) -> Result<(), StatusCode> {
    let mut model = context.model.write().await;

    model.set_name(&client_id, username)
}
