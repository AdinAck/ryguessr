use crate::geo::sampler::RandomLocationSampler;
use crate::streetview::StreetViewClient;

pub struct AppState {
    pub sampler: RandomLocationSampler,
    pub streetview: StreetViewClient,
}
