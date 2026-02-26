use std::{env, fs::File, io::BufReader};

use anyhow::Context;
use dotenvy::dotenv;
use geo::{Area, BoundingRect, Contains, MultiPolygon, Polygon};
use geojson::{FeatureCollection, GeoJson, Value};
use rand::{
    self,
    distr::{Distribution, weighted::WeightedIndex},
    rngs::ThreadRng,
    seq::IndexedRandom,
};
use reqwest::Client;
use tracing::{debug, info};

struct Country {
    name: String,
    region: String,
    geometry: MultiPolygon<f64>,
    population: f64,
}

impl Country {
    pub fn get_random_point(&self, rng: &mut ThreadRng) -> Option<(f64, f64)> {
        // Choose random territory weighted
        let chosen_poly = self
            .geometry
            .0
            .choose_weighted(rng, |poly| poly.unsigned_area())
            .ok()?;

        let rect = chosen_poly.bounding_rect()?;
        let (min_x, max_x) = (rect.min().x, rect.max().x);
        let (min_y, max_y) = (rect.min().y, rect.max().y);

        for _ in 0..100 {
            let x = rand::random_range(min_x..max_x);
            let y = rand::random_range(min_y..max_y);
            if chosen_poly.contains(&geo::Point::new(x, y)) {
                return Some((y, x)); // lat, lng
            }
        }

        None
    }
}

fn load_countries(path: &str) -> anyhow::Result<Vec<Country>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let geojson: GeoJson = serde_json::from_reader(reader)?;

    let collection: FeatureCollection = geojson.try_into()?;
    let mut countries = vec![];

    for feature in collection.features {
        let properties = &feature.properties.context("Missing properties")?;

        let name = properties
            .get("NAME")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        let region = properties
            .get("CONTINENT")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let geometry_value = feature
            .geometry
            .as_ref()
            .map(|g| &g.value)
            .context("Missing geometry")?;

        let multipolygon = match geometry_value {
            Value::Polygon(_) => {
                let poly: Polygon<f64> = geometry_value.clone().try_into()?;
                MultiPolygon::new(vec![poly])
            }
            Value::MultiPolygon(_) => geometry_value.clone().try_into()?,
            _ => continue,
        };

        let population = properties
            .get("POP_EST")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        countries.push(Country {
            name,
            region,
            geometry: multipolygon,
            population,
        });
    }

    Ok(countries)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    dotenv().ok();

    let api_key =
        env::var("GOOGLE_MAPS_API_KEY").context("GOOGLE_MAPS_API_KEY must be set in .env")?;

    let client = Client::new();

    let mut rng = rand::rng();

    // Load country geometries
    let countries = load_countries("countries_geo.json")?;
    info!("Loaded {} countries", countries.len());

    // Pick random country
    let weighted_dist = WeightedIndex::new(countries.iter().map(|c| c.population)).unwrap();
    let country = &countries[weighted_dist.sample(&mut rng)];
    info!("Selected country: {} ({})", country.name, country.region);

    for _ in 0..1000 {
        let (lat, lng) = country
            .get_random_point(&mut rng)
            .context("Couldn't find point")?;

        debug!("Selected point: {}, {}", lat, lng);

        let url = format!(
            "https://maps.googleapis.com/maps/api/streetview/metadata?location={},{}&radius=5000&key={}",
            lat, lng, api_key
        );

        let resp = serde_json::from_str::<serde_json::Value>(
            &client.get(url).send().await?.text().await?,
        )?;

        let pano_id = match resp.get("pano_id") {
            Some(id) => id.as_str().unwrap_or(""),
            None => {
                // debug!("No pano_id found for point {}, {}", lat, lng);
                continue;
            }
        };

        info!("found pano id {pano_id}");

        return Ok(());
    }

    Ok(())
}
