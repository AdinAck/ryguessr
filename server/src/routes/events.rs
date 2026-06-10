use std::pin::Pin;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use axum::response::sse::KeepAlive;
use axum::{extract::State, response::Sse};
use futures_util::{Stream, StreamExt};
use reqwest::header;
use tokio_stream::wrappers::BroadcastStream;
use tracing::{debug, error};
use uuid::Uuid;

use crate::event::{EventEnvelope, Recipients};
use crate::{Context, RoomEvent, event::RoundStartData, handle, room};

#[pin_project::pin_project(PinnedDrop)]
pub struct EventStream<S> {
    #[pin]
    stream: S,

    context: Context,
    client_id: handle::Id,
    room_id: room::Id,
    session: Uuid,
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
        let session = self.session;

        tokio::spawn(async move {
            context
                .on_sse_disconnect(&client_id, &room_id, session)
                .await;
        });
    }
}

#[tracing::instrument(skip_all, fields(client_id = %*client_id))]
pub async fn sse_event_handler(
    State(context): State<Context>,
    client_id: handle::Id,
) -> Result<Response<Body>, StatusCode> {
    let session = Uuid::new_v4();

    let mut model = context.model.write().await;

    let handle = model
        .clients
        .get_mut(&client_id)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    handle.session = Some(session);

    let room_id = handle.room.clone();
    let room = model.rooms.get_mut(&room_id).ok_or(StatusCode::NOT_FOUND)?;
    room.connect();

    let round = room.round;
    let pano_id = room.location.pano_id.clone();
    let initial_event = RoomEvent::RoundStart(RoundStartData { pano_id, round });

    let rx = room.event_tx.subscribe();

    let filter_id = client_id.clone();
    let broadcast_stream = BroadcastStream::new(rx).filter_map(move |result| {
        std::future::ready(match result {
            Ok(envelope) => {
                let include = match &envelope.recipients {
                    Recipients::All => true,
                    Recipients::Only(set) => set.contains(&filter_id),
                    Recipients::Except(set) => !set.contains(&filter_id),
                };
                if include {
                    debug!("sending event: {:?}", envelope.event);
                    Some(envelope.event.try_into())
                } else {
                    None
                }
            }
            Err(e) => {
                error!(%e, "sse subscriber lagged");
                Some(Err(axum::Error::new(e)))
            }
        })
    });

    let event_stream = EventStream {
        stream: broadcast_stream,
        context: context.clone(),
        client_id: client_id.clone(),
        room_id,
        session,
    };

    room.event_tx
        .send(EventEnvelope {
            event: initial_event,
            recipients: Recipients::only_one(client_id),
        })
        .unwrap();

    let sse = Sse::new(event_stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    );
    let mut response = sse.into_response();
    // headers for cloudflare or reverse proxies that buffer responses by default
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        "no-cache, no-transform".parse().unwrap(),
    );
    response
        .headers_mut()
        .insert("X-Accel-Buffering", "no".parse().unwrap());

    Ok(response)
}
