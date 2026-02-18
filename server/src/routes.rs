use serde::{Deserialize, Serialize};

pub mod random;
pub mod score;

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize, Deserialize)]
pub struct Location {
    pub lat: f64,
    pub lng: f64,
}
