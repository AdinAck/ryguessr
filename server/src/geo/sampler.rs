use rand::distr::{Distribution, weighted::WeightedIndex};

use super::regions::RegionData;

pub struct RandomLocationSampler {
    regions: Vec<RegionData>,
    region_dist: WeightedIndex<usize>,
}

impl RandomLocationSampler {
    pub fn new(regions: Vec<RegionData>) -> anyhow::Result<Self> {
        let region_dist = WeightedIndex::new(regions.iter().map(|r| r.count))?;
        Ok(Self {
            regions,
            region_dist,
        })
    }

    pub fn sample(&self) -> (f64, f64) {
        let mut rng = rand::rng();
        let region = &self.regions[self.region_dist.sample(&mut rng)];
        let idx = rand::random_range(0..region.count);
        region.get_point(idx)
    }
}
