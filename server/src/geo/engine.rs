use anyhow::bail;
use tracing::{debug, instrument};

use crate::geo::{Location, sampler::RandomLocationSampler, streetview::StreetViewClient};

const MAX_ATTEMPTS: usize = 100;

pub struct LocationEngine {
    streetview_client: StreetViewClient,
    location_sampler: RandomLocationSampler,
}

impl LocationEngine {
    pub fn new(
        streetview_client: StreetViewClient,
        location_sampler: RandomLocationSampler,
    ) -> Self {
        Self {
            streetview_client,
            location_sampler,
        }
    }

    #[instrument(skip(self))]
    pub async fn get_random_location(&self) -> anyhow::Result<Location> {
        for _ in 0..MAX_ATTEMPTS {
            let (lat, lng) = self.location_sampler.sample();
            debug!(lat, lng, "trying point");

            match self.streetview_client.find_panorama(lat, lng).await {
                Ok(location) => return Ok(location),
                _ => continue,
            }
        }
        bail!("Could not find location after {} attempts", MAX_ATTEMPTS)
    }
}
