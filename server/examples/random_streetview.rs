use std::env;

use anyhow::{Context, anyhow};
use dotenvy::dotenv;
use log::info;
use reqwest::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    dotenv().ok();

    let api_key =
        env::var("GOOGLE_MAPS_API_KEY").context("GOOGLE_MAPS_API_KEY must be set in .env")?;

    let client = Client::new();

    let lat = rand::random_range(-90.0..90.0);
    let lng = rand::random_range(-180.0..180.0);

    let url = format!(
        "https://maps.googleapis.com/maps/api/streetview/metadata?location={},{}&radius=1000000&key={}",
        lat, lng, api_key
    );

    let resp =
        serde_json::from_str::<serde_json::Value>(&client.get(url).send().await?.text().await?)?;

    let pano_id = resp
        .get("pano_id")
        .ok_or(anyhow!("expected pano id"))?
        .as_str()
        .ok_or(anyhow!("expected pano id to be a string"))?;

    info!("found pano id {pano_id}");

    Ok(())
}
