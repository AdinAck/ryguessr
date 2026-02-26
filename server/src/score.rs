use std::f64;

use crate::Coordinates;

const EARTH_RADIUS: f64 = 6371.0;
const WORLD_MAP_RANGE: f64 = 14_917.0;

/// Calculate the Haversine distance between two locations in kilometers.
pub fn haversine_distance(loc1: &Coordinates, loc2: &Coordinates) -> f64 {
    let dlat = (loc2.lat - loc1.lat).to_radians();
    let dlng = (loc2.lng - loc1.lng).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + loc1.lat.to_radians().cos() * loc2.lat.to_radians().cos() * (dlng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    EARTH_RADIUS * c
}

/// Calculate the score based on distance using an exponential decay function.
pub fn calculate_score(dist: f64) -> u16 {
    const MAX_SCORE: f64 = 5000.0;
    const DECAY_RATE: f64 = 10.0;

    // In the future, we may want to adjust WORLD_MAP_RANGE depending on the size of the playable
    // area by adding it as a parameter.
    let score = MAX_SCORE * f64::consts::E.powf(-DECAY_RATE * dist / WORLD_MAP_RANGE);
    let final_score = score.round();
    final_score.clamp(0.0, MAX_SCORE) as u16
}
