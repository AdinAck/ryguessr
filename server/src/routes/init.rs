use std::sync::Arc;

use axum::{Json, extract::State};
use axum_extra::TypedHeader;
use reqwest::StatusCode;
use tokio::sync::RwLock;

use crate::{Context, geo::engine::LocationEngine, handle, room};

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn init_handler(
    State((context, engine)): State<(Arc<RwLock<Context>>, Arc<LocationEngine>)>,
    TypedHeader(client_id): TypedHeader<handle::Id>,
    Json(username): Json<String>,
) -> Result<Json<room::Id>, StatusCode> {
    let location = engine
        .get_random_location()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut cx = context.write().await;
    let room_id = cx.create_room_with_user(location, client_id, username);

    Ok(Json(room_id))
}
