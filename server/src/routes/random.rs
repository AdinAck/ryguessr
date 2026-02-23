use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use log::debug;
use serde::Serialize;

use crate::state::AppState;

const MAX_ATTEMPTS: usize = 100;

#[derive(Clone, Serialize)]
pub struct LocationResponse {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Clone, Serialize, Debug)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn random_location(
    State(state): State<Arc<AppState>>,
) -> Result<Json<LocationResponse>, (StatusCode, Json<ErrorResponse>)> {
    for _ in 0..MAX_ATTEMPTS {
        let (lat, lng) = state.sampler.sample();
        debug!("Trying point: {}, {}", lat, lng);

        let result = state.streetview.find_panorama(lat, lng).await;

        match result {
            Ok(Some((pano_lat, pano_lng))) => {
                return Ok(Json(LocationResponse {
                    lat: pano_lat,
                    lng: pano_lng,
                }));
            }
            Ok(None) => continue,
            Err(e) => {
                debug!("Street View API error: {}", e);
                continue;
            }
        }
    }

    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorResponse {
            error: format!("No panorama found after {} attempts", MAX_ATTEMPTS),
        }),
    ))
}
