use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::response::Sse;
use axum::response::sse::Event;
use axum::routing::get;
use futures_util::Stream;
use tokio::sync::{RwLock, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tower_http::services::ServeDir;
use tracing::{debug, info};

use ryguessr::config::Config;
use ryguessr::geo::Coordinates;
use ryguessr::geo::regions::load_all_regions;
use ryguessr::geo::sampler::RandomLocationSampler;
use ryguessr::geo::streetview::StreetViewClient;

pub struct AppState {
    pub sampler: RandomLocationSampler,
    pub streetview: StreetViewClient,
    pub location_senders: Vec<mpsc::Sender<Result<Event, axum::Error>>>,
}

const MAX_ATTEMPTS: usize = 100;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

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
                    let result: Result<Coordinates, String> = random_location(&state).await;

                    for sender in &state.location_senders {
                        sender
                            .send(result.clone().map_err(axum::Error::new).and_then(|coords| {
                                Event::default().event("location").json_data(coords)
                            }))
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

pub async fn random_location(state: &AppState) -> Result<Coordinates, String> {
    for _ in 0..MAX_ATTEMPTS {
        let (lat, lng) = state.sampler.sample();
        debug!("Trying point: {}, {}", lat, lng);

        match state.streetview.find_panorama(lat, lng).await {
            Ok(location) => return Ok(location.coordinates),
            Err(e) => {
                debug!("Street View API error: {}", e);
                continue;
            }
        }
    }

    Err(format!("No panorama found after {} attempts", MAX_ATTEMPTS))
}

async fn sse_handler(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    println!("Client connected.");

    let (tx, rx) = mpsc::channel(10);

    let stream = ReceiverStream::new(rx);

    state.write().await.location_senders.push(tx);

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
