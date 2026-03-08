use serde::{Deserialize, Serialize};

pub mod engine;
pub mod regions;
pub mod sampler;
pub mod streetview;

pub type PanoId = String;

pub struct Location {
    pub coordinates: Coordinates,
    pub pano_id: PanoId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub lat: f64,
    pub lng: f64,
}
