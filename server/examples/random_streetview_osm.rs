use std::{env, path::Path};

use anyhow::Context;
use dotenvy::dotenv;
use tracing::{debug, info, trace};
use memmap2::Mmap;
use rand::distr::{Distribution, weighted::WeightedIndex};
use reqwest::Client;

/// Size of a single point: two f32s (lat, lng), 8 bytes.
const POINT_SIZE: usize = 8;

struct RegionData {
    name: String,
    mmap: Mmap,
    count: usize,
}

/// Recursively load all .roadpoints files from a directory tree.
fn load_all_regions(dir: &Path) -> anyhow::Result<Vec<RegionData>> {
    let mut regions = Vec::new();
    load_regions_recursive(dir, dir, &mut regions)?;
    regions.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(regions)
}

fn load_regions_recursive(
    base: &Path,
    dir: &Path,
    regions: &mut Vec<RegionData>,
) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            load_regions_recursive(base, &path, regions)?;
        } else if path.extension().is_some_and(|e| e == "roadpoints") {
            let file = std::fs::File::open(&path)?;
            let mmap = unsafe { Mmap::map(&file)? };
            let count = mmap.len() / POINT_SIZE;
            let name = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .with_extension("")
                .to_string_lossy()
                .to_string();
            trace!(
                "  {} — {} points ({:.1} MB)",
                name,
                count,
                mmap.len() as f64 / 1_048_576.0
            );
            regions.push(RegionData { name, mmap, count });
        }
    }
    Ok(())
}

fn get_point(mmap: &Mmap, index: usize) -> (f64, f64) {
    let offset = index * POINT_SIZE;
    let lat = f32::from_le_bytes(mmap[offset..offset + 4].try_into().unwrap());
    let lng = f32::from_le_bytes(mmap[offset + 4..offset + 8].try_into().unwrap());
    (lat as f64, lng as f64)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    dotenv().ok();

    let api_key =
        env::var("GOOGLE_MAPS_API_KEY").context("GOOGLE_MAPS_API_KEY must be set in .env")?;

    let osm_dir = env::args().nth(1).unwrap_or_else(|| "osm".to_string());

    let regions = load_all_regions(Path::new(&osm_dir))?;
    if regions.is_empty() {
        anyhow::bail!("No .roadpoints files found in {}", osm_dir);
    }

    let total: usize = regions.iter().map(|r| r.count).sum();
    info!("Loaded {} regions, {} total points", regions.len(), total);

    // Weight region selection by point count so sampling is uniform across all points
    let region_dist = WeightedIndex::new(regions.iter().map(|r| r.count))?;

    let client = Client::new();
    let mut rng = rand::rng();

    for _ in 0..100 {
        let region = &regions[region_dist.sample(&mut rng)];
        let idx = rand::random_range(0..region.count);
        let (lat, lng) = get_point(&region.mmap, idx);

        debug!("Trying point in {}: {}, {}", region.name, lat, lng);

        let url = format!(
            "https://maps.googleapis.com/maps/api/streetview/metadata?location={},{}&radius=111&key={}",
            lat, lng, api_key
        );

        let resp = serde_json::from_str::<serde_json::Value>(
            &client.get(url).send().await?.text().await?,
        )?;

        let pano_id = match resp.get("pano_id") {
            Some(id) => id.as_str().unwrap_or(""),
            None => continue,
        };

        info!("Found pano id {pano_id}");
        info!("https://www.google.com/maps/@?api=1&map_action=pano&pano={pano_id}");

        return Ok(());
    }

    info!("No panorama found after 100 attempts");
    Ok(())
}
