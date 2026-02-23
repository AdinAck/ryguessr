use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::response::Sse;
use axum::response::sse::Event;
use axum::routing::get;
use axum::{Json, Router};
use futures_util::Stream;
use log::{debug, info};
use ryguessr::routes::random::{ErrorResponse, LocationResponse};
use tokio::sync::{RwLock, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tower_http::services::ServeDir;

use ryguessr::config::Config;
use ryguessr::geo::regions::load_all_regions;
use ryguessr::geo::sampler::RandomLocationSampler;
use ryguessr::streetview::StreetViewClient;

pub struct AppState {
    pub sampler: RandomLocationSampler,
    pub streetview: StreetViewClient,
    pub location_senders: Vec<mpsc::Sender<Result<Event, axum::Error>>>,
}

const MAX_ATTEMPTS: usize = 100;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = Config::from_env()?;

    let regions = load_all_regions(Path::new(&config.osm_data_dir))?;
    anyhow::ensure!(
        !regions.is_empty(),
        "No .roadpoints files found in {}",
        config.osm_data_dir
    );

    let total: usize = regions.iter().map(|r| r.count).sum();
    info!("Loaded {} regions, {} total points", regions.len(), total);

    let sampler = RandomLocationSampler::new(regions)?;
    let streetview = StreetViewClient::new(config.google_maps_api_key);

    let state = Arc::new(RwLock::new(AppState {
        sampler,
        streetview,
        location_senders: Vec::new(),
    }));

    tokio::spawn({
        let state = Arc::clone(&state);

        async move {
            loop {
                let state = state.read().await;

                if !state.location_senders.is_empty() {
                    let coords = random_location(&state).await;

                    for sender in &state.location_senders {
                        sender
                            .send(
                                coords
                                    .clone()
                                    .map_err(|e| axum::Error::new(e.error))
                                    .and_then(|location_response| {
                                        Event::default()
                                            .event("location")
                                            .json_data(location_response)
                                    }),
                            )
                            .await
                            .unwrap();
                    }
                }

                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        }
    });

    let app = Router::new()
        .route("/sse", get(sse_handler))
        .with_state(state)
        .fallback_service(ServeDir::new("../web/out"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Listening on: {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}

pub async fn random_location(state: &AppState) -> Result<LocationResponse, ErrorResponse> {
    for _ in 0..MAX_ATTEMPTS {
        let (lat, lng) = state.sampler.sample();
        debug!("Trying point: {}, {}", lat, lng);

        let result = state.streetview.find_panorama(lat, lng).await;

        match result {
            Ok(Some((pano_lat, pano_lng))) => {
                return Ok(LocationResponse {
                    lat: pano_lat,
                    lng: pano_lng,
                });
            }
            Ok(None) => continue,
            Err(e) => {
                debug!("Street View API error: {}", e);
                continue;
            }
        }
    }

    Err(ErrorResponse {
        error: format!("No panorama found after {} attempts", MAX_ATTEMPTS),
    })
}

async fn sse_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(data): Json<serde_json::Value>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    println!("Client connected with testimony: {data:?}.");

    let (tx, rx) = mpsc::channel(10);

    let stream = ReceiverStream::new(rx);

    state.write().await.location_senders.push(tx);

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
