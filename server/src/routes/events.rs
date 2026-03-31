use std::pin::Pin;

use axum::http::StatusCode;
use axum::{
    extract::State,
    response::{Sse, sse},
};
use futures_util::{Stream, StreamExt};
use tokio_stream::wrappers::BroadcastStream;

use crate::{Context, RoomEvent, context::SharedModel, event::RoundStartData, handle};

#[pin_project::pin_project(PinnedDrop)]
pub struct EventStream<S> {
    #[pin]
    stream: S,

    model: SharedModel,
    client_id: handle::Id,
}

impl<S: Stream> Stream for EventStream<S> {
    type Item = S::Item;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.project().stream.poll_next(cx)
    }
}

#[pin_project::pinned_drop]
impl<S> PinnedDrop for EventStream<S> {
    fn drop(self: Pin<&mut Self>) {
        let state = self.model.clone();
        let client_id = self.client_id.clone();

        tokio::spawn(async move {
            let mut state = state.write().await;
            state.remove_client(&client_id);
            tracing::info!(client_id = %*client_id, "client disconnected, removed from room");
        });
    }
}

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn sse_event_handler(
    State(context): State<Context>,
    client_id: handle::Id,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, axum::Error>>>, StatusCode> {
    let model = context.model.read().await;
    let handle = model
        .clients
        .get(&client_id)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let room_id = handle.room.clone();
    let room = model.rooms.get(&room_id).ok_or(StatusCode::NOT_FOUND)?;

    let round = room.round;
    let pano_id = room.location.pano_id.clone();
    let initial_event = RoomEvent::RoundStart(RoundStartData { pano_id, round });

    let rx = room.event_tx.subscribe();

    let broadcast_stream = BroadcastStream::new(rx).map(|result| match result {
        Ok(event) => Ok(event.try_into()?),
        Err(e) => {
            tracing::error!(%e, "failed to serialize event");

            Err(axum::Error::new(e))
        }
    });

    let event_stream = EventStream {
        stream: broadcast_stream,
        model: context.model.clone(),
        client_id: client_id.clone(),
    };

    room.event_tx.send(initial_event).unwrap();

    Ok(Sse::new(event_stream))
}
