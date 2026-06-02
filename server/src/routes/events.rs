use std::pin::Pin;

use axum::http::StatusCode;
use axum::{
    extract::State,
    response::{Sse, sse},
};
use futures_util::{Stream, StreamExt};
use tokio_stream::wrappers::BroadcastStream;
use tracing::debug;

use crate::{Context, RoomEvent, event::RoundStartData, handle, room};

#[pin_project::pin_project(PinnedDrop)]
pub struct EventStream<S> {
    #[pin]
    stream: S,

    context: Context,
    client_id: handle::Id,
    room_id: room::Id,
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
        let context = self.context.clone();
        let client_id = self.client_id.clone();
        let room_id = self.room_id.clone();

        tokio::spawn(async move {
            context.on_sse_disconnect(&client_id, &room_id).await;
        });
    }
}

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn sse_event_handler(
    State(context): State<Context>,
    client_id: handle::Id,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, axum::Error>>>, StatusCode> {
    let mut model = context.model.write().await;
    let handle = model
        .clients
        .get(&client_id)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let room_id = handle.room.clone();
    let room = model.rooms.get_mut(&room_id).ok_or(StatusCode::NOT_FOUND)?;
    room.connect();

    let round = room.round;
    let pano_id = room.location.pano_id.clone();
    let initial_event = RoomEvent::RoundStart(RoundStartData { pano_id, round });

    let rx = room.event_tx.subscribe();

    let broadcast_stream = BroadcastStream::new(rx).map(|result| match result {
        Ok(event) => {
            debug!("sending event: {:?}", event);

            Ok(event.try_into()?)
        }
        Err(e) => {
            tracing::error!(%e, "failed to serialize event");

            Err(axum::Error::new(e))
        }
    });

    let event_stream = EventStream {
        stream: broadcast_stream,
        context: context.clone(),
        client_id: client_id.clone(),
        room_id,
    };

    room.event_tx.send(initial_event).unwrap();

    Ok(Sse::new(event_stream))
}
