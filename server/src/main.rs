use std::{path::Path, sync::Arc};

use axum::{Router, routing::get};
use ryguessr::{
    config::Config,
    context::Context,
    geo::{
        engine::LocationEngine, regions::load_all_regions, sampler::RandomLocationSampler,
        streetview::StreetViewClient,
    },
    routes,
};
use tokio::sync::RwLock;
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

    let cx = Arc::new(RwLock::new(Context::empty(LocationEngine::new(
        StreetViewClient::new(config.google_maps_api_key),
        RandomLocationSampler::new(regions)?,
    ))));

    let app = Router::new()
        .route("/events", get(routes::events::sse_event_handler))
        .with_state(cx)
        .layer(TraceLayer::new_for_http())
        .fallback_service(ServeDir::new("../web/out"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Listening on: {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
