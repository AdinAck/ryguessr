use std::sync::Arc;

use axum::{
    extract::State,
    response::{Sse, sse},
};
use axum_extra::TypedHeader;
use futures_util::Stream;
use reqwest::StatusCode;
use tokio::sync::RwLock;

use crate::{Context, handle};

pub async fn sse_event_handler(
    State(context): State<Arc<RwLock<Context>>>,
    TypedHeader(client_id): TypedHeader<handle::Id>,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, axum::Error>>>, StatusCode> {
    let cx = context.read().await;
    let handle = cx.clients.get(&client_id).ok_or(StatusCode::UNAUTHORIZED)?;
    let room = cx.rooms.get(&handle.room).ok_or(StatusCode::NOT_FOUND)?;

    let mut rx = room.event_tx.subscribe();
    drop(cx);

    let stream = async_stream::stream! {
        while let Ok(event) = rx.recv().await {
            match sse::Event::default().event(event.name()).json_data(&event) {
                Ok(event_json) => yield Ok(event_json),
                Err(e) => log::error!("{} failed to serialize event", e),
            }
        }
    };

    Ok(Sse::new(stream))
}
