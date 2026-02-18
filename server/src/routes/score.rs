use std::f64::consts::E;

use axum::{Json, extract::Query};
use serde::{Deserialize, Serialize};

use crate::routes::Location;

const WORLD_MAP_RANGE: f64 = 14_917.0;

#[derive(Deserialize)]
pub struct ScoreQuery {
    guess_lat: f64,
    guess_lng: f64,

    real_lat: f64,
    real_lng: f64,
}

#[derive(Serialize)]
pub struct ScoreResponse {
    distance_km: f64,
    score: u16,
}

pub async fn submit_location(Query(params): Query<ScoreQuery>) -> Json<ScoreResponse> {
    let loc1 = Location {
        lat: params.guess_lat,
        lng: params.guess_lng,
    };
    let loc2 = Location {
        lat: params.real_lat,
        lng: params.real_lng,
    };

    let distance_km = haversine_distance(&loc1, &loc2);
    let score = calculate_score(distance_km, WORLD_MAP_RANGE);

    Json(ScoreResponse { distance_km, score })
}

/// Calculate the Haversine distance between two locations in kilometers.
fn haversine_distance(loc1: &Location, loc2: &Location) -> f64 {
    let r = 6371.0; // Earth radius in kilometers
    let dlat = (loc2.lat - loc1.lat).to_radians();
    let dlng = (loc2.lng - loc1.lng).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + loc1.lat.to_radians().cos() * loc2.lat.to_radians().cos() * (dlng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}

/// Calculate the score based on distance using an exponential decay function.
fn calculate_score(dist: f64, map_range: f64) -> u16 {
    const MAX_SCORE: f64 = 5000.0;
    const DECAY_RATE: f64 = 10.0;

    let score = MAX_SCORE * E.powf(-DECAY_RATE * dist / map_range);
    let final_score = score.round();
    final_score.clamp(0.0, MAX_SCORE) as u16
}
