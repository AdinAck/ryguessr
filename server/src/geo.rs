use serde::Serialize;

pub mod engine;
pub mod regions;
pub mod sampler;
pub mod streetview;

pub type PanoId = String;

pub struct Location {
    coordinates: Coordinates,
    pano_id: PanoId,
}

#[derive(Clone, Serialize)]
pub struct Coordinates {
    lat: f64,
    lng: f64,
}
