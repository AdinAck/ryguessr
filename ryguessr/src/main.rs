use std::env;

use anyhow::Context;
use dotenvy::dotenv;
use google_maps::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let api_key =
        env::var("GOOGLE_MAPS_API_KEY").context("GOOGLE_MAPS_API_KEY must be set in .env")?;

    let _client = Client::try_new(api_key)?;

    // more to come...

    Ok(())
}
