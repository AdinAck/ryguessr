mod config;
mod geo;
mod routes;
mod state;
mod streetview;

use std::path::Path;
use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use log::info;
use tower_http::services::ServeDir;

use crate::config::Config;
use crate::geo::regions::load_all_regions;
use crate::geo::sampler::RandomLocationSampler;
use crate::state::AppState;
use crate::streetview::StreetViewClient;

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

    let state = Arc::new(AppState {
        sampler,
        streetview,
    });

    let app = Router::new()
        .route("/api/random-location", get(routes::random::random_location))
        .with_state(state)
        .fallback_service(ServeDir::new("../web/out"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Listening on: {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
