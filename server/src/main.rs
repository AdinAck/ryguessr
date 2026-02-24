use std::{path::Path, sync::Arc};

use axum::{Router, routing::get};
use log::info;
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
use tower_http::services::ServeDir;

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

    let cx = Arc::new(RwLock::new(Context::empty(LocationEngine::new(
        StreetViewClient::new(config.google_maps_api_key),
        RandomLocationSampler::new(regions)?,
    ))));

    let app = Router::new()
        .route("/events", get(routes::events::sse_event_handler))
        .with_state(cx)
        .fallback_service(ServeDir::new("../web/out"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Listening on: {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
