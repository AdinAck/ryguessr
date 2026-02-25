use std::sync::Arc;

use axum::extract::State;
use axum_extra::TypedHeader;
use reqwest::StatusCode;
use tokio::sync::RwLock;

use crate::{Context, handle::Id};

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn guess_handler(
    State(context): State<Arc<RwLock<Context>>>,
    TypedHeader(client_id): TypedHeader<Id>,
) -> StatusCode {
    StatusCode::OK
}
