use std::{
    collections::{HashMap, HashSet},
    env,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use osmpbf::{Element, ElementReader};
use serde::Deserialize;

/// Highway tag values that represent drivable roads (where Street View cars go).
const ROAD_HIGHWAY_TYPES: &[&str] = &[
    "motorway",
    "trunk",
    "primary",
    "secondary",
    "tertiary",
    "unclassified",
    "residential",
    "motorway_link",
    "trunk_link",
    "primary_link",
    "secondary_link",
    "tertiary_link",
    "living_street",
];

const GRID_RESOLUTION: f64 = 0.001;

fn lat_lng_to_cell(lat: f64, lng: f64) -> (i32, i32) {
    (
        (lat / GRID_RESOLUTION).floor() as i32,
        (lng / GRID_RESOLUTION).floor() as i32,
    )
}

#[derive(Deserialize)]
struct GeofabrikIndex {
    features: Vec<GeofabrikFeature>,
}

#[derive(Deserialize)]
struct GeofabrikFeature {
    properties: GeofabrikProperties,
}

#[derive(Deserialize)]
struct GeofabrikProperties {
    id: String,
    parent: Option<String>,
    name: String,
    urls: Option<GeofabrikUrls>,
}

#[derive(Deserialize)]
struct GeofabrikUrls {
    pbf: Option<String>,
}

struct Region {
    id: String,
    name: String,
    pbf_url: Option<String>,
}

/// Preprocess a PBF file into a .roadpoints file. Returns number of points written.
fn preprocess_pbf(pbf_path: &Path, output_path: &Path) -> anyhow::Result<usize> {
    // Pass 1: collect node IDs referenced by road ways
    let reader = ElementReader::from_path(pbf_path)?;
    let road_node_ids: HashSet<i64> = reader.par_map_reduce(
        |element| {
            if let Element::Way(way) = element {
                let is_road = way
                    .tags()
                    .any(|(k, v)| k == "highway" && ROAD_HIGHWAY_TYPES.contains(&v));
                if is_road {
                    return way.refs().collect::<HashSet<_>>();
                }
            }
            HashSet::new()
        },
        HashSet::new,
        |mut a, b| {
            a.extend(b);
            a
        },
    )?;

    // Pass 2: resolve node coordinates, grid-deduplicate
    let reader = ElementReader::from_path(pbf_path)?;
    let coords: Vec<(f64, f64)> = reader.par_map_reduce(
        |element| {
            let (id, lat, lon) = match element {
                Element::Node(node) => (node.id(), node.lat(), node.lon()),
                Element::DenseNode(node) => (node.id, node.lat(), node.lon()),
                _ => return vec![],
            };
            if road_node_ids.contains(&id) {
                vec![(lat, lon)]
            } else {
                vec![]
            }
        },
        Vec::new,
        |mut a, b| {
            a.extend(b);
            a
        },
    )?;

    // Grid deduplication
    let mut seen_cells: HashSet<(i32, i32)> = HashSet::new();
    let mut deduped: Vec<(f32, f32)> = Vec::new();

    for (lat, lng) in &coords {
        let cell = lat_lng_to_cell(*lat, *lng);
        if seen_cells.insert(cell) {
            deduped.push((*lat as f32, *lng as f32));
        }
    }

    // Write binary output
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    for (lat, lng) in &deduped {
        writer.write_all(&lat.to_le_bytes())?;
        writer.write_all(&lng.to_le_bytes())?;
    }
    writer.flush()?;

    Ok(deduped.len())
}

async fn download_pbf(client: &reqwest::Client, url: &str, dest: &Path) -> anyhow::Result<()> {
    let resp = client
        .get(url)
        .send()
        .await?
        .error_for_status()
        .context("Failed to download PBF")?;

    let total_size = resp.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("##-"),
    );

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(dest)?;
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        pb.inc(chunk.len() as u64);
    }

    pb.finish_and_clear();
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let region_filter = args
        .iter()
        .position(|a| a == "--region")
        .map(|i| args.get(i + 1).expect("--region requires a value").clone());

    let output_dir = PathBuf::from(
        args.iter()
            .position(|a| a == "--output")
            .and_then(|i| args.get(i + 1))
            .map(|s| s.as_str())
            .unwrap_or("osm"),
    );

    // Fetch the Geofabrik index
    info!("Fetching Geofabrik region index...");
    let client = reqwest::Client::new();
    let index_url = "https://download.geofabrik.de/index-v1.json";
    let index: GeofabrikIndex = client
        .get(index_url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    info!("Found {} regions in index", index.features.len());

    // Build region tree: collect all IDs and identify parents
    let mut regions: HashMap<String, Region> = HashMap::new();
    let mut parent_ids: HashSet<String> = HashSet::new();

    for feature in &index.features {
        let props = &feature.properties;
        if let Some(parent) = &props.parent {
            parent_ids.insert(parent.clone());
        }
        regions.insert(
            props.id.clone(),
            Region {
                id: props.id.clone(),
                name: props.name.clone(),
                pbf_url: props.urls.as_ref().and_then(|u| u.pbf.clone()),
            },
        );
    }

    // Build parent lookup for path construction
    let parent_map: HashMap<String, String> = index
        .features
        .iter()
        .filter_map(|f| {
            f.properties
                .parent
                .as_ref()
                .map(|p| (f.properties.id.clone(), p.clone()))
        })
        .collect();

    // Leaf regions = most granular extracts we want to download.
    // Exclude a region if:
    //  - it's listed as another region's parent, OR
    //  - another region's ID starts with this ID + "/" (e.g. "us" when "us/california" exists), OR
    //  - it's an aggregate region like "us-midwest" where "us/" sub-regions exist
    let all_ids: Vec<&str> = regions.keys().map(|s| s.as_str()).collect();
    let has_slash_children = |id: &str| -> bool {
        let prefix = format!("{}/", id);
        all_ids.iter().any(|other| other.starts_with(&prefix))
    };
    let mut leaves: Vec<&Region> = regions
        .values()
        .filter(|r| {
            r.pbf_url.is_some()
                && !parent_ids.contains(&r.id)
                && !has_slash_children(&r.id)
                && !r
                    .id
                    .contains('-')
                    .then(|| {
                        // "us-midwest" -> check if "us/" sub-regions exist
                        let base = r.id.split('-').next().unwrap();
                        has_slash_children(base)
                    })
                    .unwrap_or(false)
        })
        .collect();
    leaves.sort_by(|a, b| a.id.cmp(&b.id));

    // Build full path for a region ID (e.g. "africa/south-africa")
    let build_path = |id: &str| -> String {
        let mut parts = vec![id.to_string()];
        let mut current = id.to_string();
        while let Some(parent) = parent_map.get(&current) {
            parts.push(parent.clone());
            current = parent.clone();
        }
        parts.reverse();
        parts.join("/")
    };

    // Apply region filter
    let filtered: Vec<(&Region, String)> = leaves
        .iter()
        .map(|r| (*r, build_path(&r.id)))
        .filter(|(_, path)| {
            if let Some(filter) = &region_filter {
                path.starts_with(filter.as_str()) || *path == *filter
            } else {
                true
            }
        })
        .collect();

    if filtered.is_empty() {
        if let Some(filter) = &region_filter {
            anyhow::bail!("No leaf regions match filter '{}'", filter);
        } else {
            anyhow::bail!("No leaf regions found");
        }
    }

    println!(
        "Processing {} leaf region(s){}",
        filtered.len(),
        region_filter
            .as_ref()
            .map(|f| format!(" (filter: {})", f))
            .unwrap_or_default()
    );

    // Process each leaf region
    let tmp_pbf = output_dir.join(".tmp_download.osm.pbf");
    let mut completed = 0;
    let total = filtered.len();

    for (region, path) in &filtered {
        let roadpoints_path = output_dir.join(format!("{}.roadpoints", path));

        // Skip if already processed (resumable)
        if roadpoints_path.exists() {
            completed += 1;
            println!(
                "[{}/{}] Skipping {} (already exists)",
                completed, total, path
            );
            continue;
        }

        completed += 1;
        let url = region.pbf_url.as_ref().unwrap();
        println!("[{}/{}] Downloading {}...", completed, total, region.name);
        info!("URL: {}", url);

        download_pbf(&client, url, &tmp_pbf).await?;

        let pbf_size = fs::metadata(&tmp_pbf)?.len();
        println!(
            "  Downloaded {:.1} MB, preprocessing...",
            pbf_size as f64 / 1_048_576.0
        );

        let points = preprocess_pbf(&tmp_pbf, &roadpoints_path)?;
        let rp_size = fs::metadata(&roadpoints_path)?.len();
        println!(
            "  Wrote {} ({} points, {:.1} MB)",
            roadpoints_path.display(),
            points,
            rp_size as f64 / 1_048_576.0
        );

        // Clean up the PBF
        fs::remove_file(&tmp_pbf)?;
    }

    println!("Done! All regions processed.");
    Ok(())
}
