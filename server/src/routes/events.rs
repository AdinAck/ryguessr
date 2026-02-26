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

/// Guard that removes a member from their room when the SSE stream is dropped (client disconnects).
struct DisconnectGuard {
    context: Arc<RwLock<Context>>,
    client_id: handle::Id,
}

impl Drop for DisconnectGuard {
    fn drop(&mut self) {
        let context = self.context.clone();
        let client_id = self.client_id.clone();
        tokio::spawn(async move {
            let mut cx = context.write().await;
            cx.remove_client(&client_id);
            tracing::info!(client_id = %*client_id, "client disconnected, removed from room");
        });
    }
}

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn sse_event_handler(
    State((context, _)): State<(Arc<RwLock<Context>>, Arc<LocationEngine>)>,
    TypedHeader(client_id): TypedHeader<handle::Id>,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, axum::Error>>>, StatusCode> {
    let cx = context.read().await;
    let handle = cx.clients.get(&client_id).ok_or(StatusCode::UNAUTHORIZED)?;
    let room_id = handle.room.clone();
    let room = cx.rooms.get(&room_id).ok_or(StatusCode::NOT_FOUND)?;

    let pano_id = room.location.pano_id.clone();
    let initial_event = stream::once(async {
        let event = RoomEvent::RoundStart(pano_id);
        event.try_into()
    });

    let mut rx = room.event_tx.subscribe();
    drop(cx);

    let guard = DisconnectGuard { context, client_id };

    let stream = async_stream::stream! {
        // Hold the guard alive for the lifetime of the stream.
        let _guard = guard;
        while let Ok(event) = rx.recv().await {
            match event.try_into() {
                Ok(event_json) => yield Ok(event_json),
                Err(err) => tracing::error!(%err, "failed to serialize event"),
            }
        }
    };

    Ok(Sse::new(initial_event.chain(stream)))
}
