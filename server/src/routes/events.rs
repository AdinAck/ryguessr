use std::{pin::Pin, sync::Arc};

use axum::{
    extract::State,
    response::{Sse, sse},
};
use axum_extra::TypedHeader;
use derive_more::{AsRef, Deref};
use futures_util::{Stream, StreamExt};
use reqwest::StatusCode;
use tokio::sync::RwLock;
use tokio_stream::wrappers::{BroadcastStream, ReceiverStream};

use crate::{
    Context, RoomEvent, SharedContext, event::RoundStartData, geo::engine::LocationEngine, handle,
};

#[derive(AsRef, Deref)]
pub struct EventStream {
    #[deref]
    stream: ReceiverStream<RoomEvent>,

    context: SharedContext,
    client_id: handle::Id,
}

impl Stream for EventStream {
    type Item = RoomEvent;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().stream).poll_next(cx)
    }
}

impl Drop for EventStream {
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

    let round = room.round;
    let pano_id = room.location.pano_id.clone();
    let initial_event = RoomEvent::RoundStart(RoundStartData { pano_id, round });

    let rx = room.event_tx.subscribe();

    let stream = BroadcastStream::new(rx).map(|result| match result {
        Ok(event) => Ok(event.try_into()?),
        Err(e) => {
            tracing::error!(%e, "failed to serialize event");

            Err(axum::Error::new(e))
        }
    });

    room.event_tx.send(initial_event).unwrap();

    Ok(Sse::new(stream))
}
