use reqwest::Client;

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

    pub async fn find_panorama(&self, lat: f64, lng: f64) -> anyhow::Result<Option<(f64, f64)>> {
        let url = format!(
            "https://maps.googleapis.com/maps/api/streetview/metadata?location={},{}&radius={}&key={}",
            lat, lng, SEARCH_RADIUS, self.api_key
        );

        let resp: serde_json::Value = self.client.get(&url).send().await?.json().await?;

        let Some(location) = resp.get("location") else {
            return Ok(None);
        };

        let pano_lat = location.get("lat").and_then(|v| v.as_f64());
        let pano_lng = location.get("lng").and_then(|v| v.as_f64());

        match (pano_lat, pano_lng) {
            (Some(lat), Some(lng)) => Ok(Some((lat, lng))),
            _ => Ok(None),
        }
    }
}
