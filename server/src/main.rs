use std::path::Path;

use axum::{
    Router,
    routing::{get, post},
};
use ryguessr::{
    config::Config,
    context::Context,
    geo::{
        engine::LocationEngine, regions::load_all_regions, sampler::RandomLocationSampler,
        streetview::StreetViewClient,
    },
    routes,
};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;

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

    let engine = LocationEngine::new(
        StreetViewClient::new(config.google_maps_api_key.clone()),
        RandomLocationSampler::new(regions)?,
    );
    let cx = Context::new(engine, config.google_maps_api_key);

    let app = Router::new()
        .route("/api/init", post(routes::init::init_handler))
        .route("/api/events", get(routes::events::sse_event_handler))
        .route("/api/guess", post(routes::guess::guess_handler))
        .route("/api/next", post(routes::next::next_handler))
        .route("/api/join", post(routes::join::join_handler))
        .route("/api/username", post(routes::username::username_handler))
        .route("/api/color", post(routes::color::color_handler))
        .route("/api/room/{code}", get(routes::room::room_handler))
        .with_state(cx)
        .layer(TraceLayer::new_for_http())
        .fallback_service(ServeDir::new(&config.web_dir));

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    info!("Listening on: {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
