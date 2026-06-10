use anyhow::Context;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub google_maps_api_key: String,
    #[serde(default = "default_osm_data_dir")]
    pub osm_data_dir: String,
    #[serde(default = "default_web_dir")]
    pub web_dir: String,
    #[serde(default = "default_bind_addr")]
    pub bind_addr: String,
}

fn default_osm_data_dir() -> String {
    "osm".to_string()
}

fn default_web_dir() -> String {
    "../web/out".to_string()
}

fn default_bind_addr() -> String {
    "0.0.0.0:3000".to_string()
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        envy::from_env::<Self>().context("failed to load configuration from environment")
    }
}
