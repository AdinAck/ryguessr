use std::sync::Arc;

use axum::{
    extract::State,
    response::{Sse, sse},
};
use axum_extra::TypedHeader;
use futures_util::{Stream, StreamExt, stream};
use reqwest::StatusCode;
use tokio::sync::RwLock;

use crate::{Context, RoomEvent, geo::engine::LocationEngine, handle};

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn sse_event_handler(
    State((context, _)): State<(Arc<RwLock<Context>>, Arc<LocationEngine>)>,
    TypedHeader(client_id): TypedHeader<handle::Id>,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, axum::Error>>>, StatusCode> {
    let cx = context.read().await;
    let handle = cx.clients.get(&client_id).ok_or(StatusCode::UNAUTHORIZED)?;
    let room = cx.rooms.get(&handle.room).ok_or(StatusCode::NOT_FOUND)?;

    let pano_id = room.location.pano_id.clone();
    let initial_event = stream::once(async {
        let event = RoomEvent::RoundStart(pano_id);
        event.try_into()
    });

    let mut rx = room.event_tx.subscribe();
    drop(cx);

    let stream = async_stream::stream! {
        while let Ok(event) = rx.recv().await {
            match event.try_into() {
                Ok(event_json) => yield Ok(event_json),
                Err(err) => tracing::error!(%err, "failed to serialize event"),
            }
        }
    };

    Ok(Sse::new(initial_event.chain(stream)))
}
