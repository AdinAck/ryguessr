use anyhow::anyhow;
use reqwest::Client;

use crate::geo::Location;

/// Search radius in meters for finding nearby panoramas.
/// This is 111 meters because points are seperated by a grid of 0.001 degrees, which is
/// approximately 111 meters at the equator.
const SEARCH_RADIUS: u32 = 111;

pub struct StreetViewClient {
    client: Client,
    api_key: String,
}

impl StreetViewClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn find_panorama(&self, lat: f64, lng: f64) -> anyhow::Result<Location> {
        let url = format!(
            "https://maps.googleapis.com/maps/api/streetview/metadata?location={},{}&radius={}&key={}",
            lat, lng, SEARCH_RADIUS, self.api_key
        );

        let resp: serde_json::Value = self.client.get(&url).send().await?.json().await?;

        let location = resp
            .get("location")
            .ok_or(anyhow!("expected location field in response"))?;

        let lat = location.get("lat").and_then(|v| v.as_f64()).ok_or(anyhow!(
            "expected latitude in location response as float 64"
        ))?;
        let lng = location.get("lng").and_then(|v| v.as_f64()).ok_or(anyhow!(
            "expected longitude in location response as float 64"
        ))?;

        let pano_id = resp
            .get("pano_id")
            .ok_or(anyhow!("expected pano id"))?
            .as_str()
            .ok_or(anyhow!("expected pano id to be a string"))?
            .to_string();

        Ok(Location { lat, lng, pano_id })
    }
}
