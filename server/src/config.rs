use std::env;

use anyhow::Context;

pub struct Config {
    pub google_maps_api_key: String,
    pub osm_data_dir: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let google_maps_api_key = env::var("GOOGLE_MAPS_API_KEY")
            .context("GOOGLE_MAPS_API_KEY must be set in environment or .env")?;
        let osm_data_dir = env::var("OSM_DATA_DIR").unwrap_or_else(|_| "osm".to_string());

        Ok(Self {
            google_maps_api_key,
            osm_data_dir,
        })
    }
}
