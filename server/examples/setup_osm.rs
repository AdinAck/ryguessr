use std::{
    collections::{HashMap, HashSet, VecDeque},
    env,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex as StdMutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use anyhow::Context;
use futures_util::{StreamExt, stream};
use osmpbf::{Element, ElementReader};
use ratatui::{
    Terminal, TerminalOptions, Viewport,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Gauge, Paragraph, Widget},
};
use serde::Deserialize;
use tokio::sync::{Mutex, mpsc};
use tracing::info;

const DOWNLOAD_CONCURRENCY: usize = 5;

/// Strip HTML tags and collapse whitespace from Geofabrik region names,
/// which sometimes include things like "Województwo dolnośląskie<br />".
fn clean_label(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    let mut last_space = false;
    for c in s.chars() {
        if in_tag {
            if c == '>' {
                in_tag = false;
                if !last_space {
                    out.push(' ');
                    last_space = true;
                }
            }
            continue;
        }
        if c == '<' {
            in_tag = true;
            continue;
        }
        if c.is_whitespace() {
            if !last_space {
                out.push(' ');
                last_space = true;
            }
        } else {
            out.push(c);
            last_space = false;
        }
    }
    out.trim().to_string()
}

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

/// Per-download-slot shared state. The render thread reads it; download tasks
/// write to it. `pos` is atomic so the per-chunk update is lock-free.
struct SlotShared {
    label: StdMutex<String>,
    pos: AtomicU64,
    total: AtomicU64,
    active: AtomicBool,
}

impl SlotShared {
    fn new() -> Self {
        Self {
            label: StdMutex::new(String::new()),
            pos: AtomicU64::new(0),
            total: AtomicU64::new(0),
            active: AtomicBool::new(false),
        }
    }

    fn begin(&self, label: &str, total: u64) {
        *self.label.lock().unwrap() = label.to_string();
        self.total.store(total, Ordering::Relaxed);
        self.pos.store(0, Ordering::Relaxed);
        self.active.store(true, Ordering::Relaxed);
    }

    fn end(&self) {
        self.active.store(false, Ordering::Relaxed);
        self.pos.store(0, Ordering::Relaxed);
        self.total.store(0, Ordering::Relaxed);
    }
}

struct RenderState {
    slots: Vec<Arc<SlotShared>>,
    overall_done: AtomicU64,
    overall_total: u64,
    start: Instant,
}

enum LogMsg {
    Line(String),
    Shutdown,
}

fn format_bytes(n: u64) -> String {
    const K: f64 = 1024.0;
    let n = n as f64;
    if n >= K * K * K {
        format!("{:.2} GiB", n / (K * K * K))
    } else if n >= K * K {
        format!("{:.1} MiB", n / (K * K))
    } else if n >= K {
        format!("{:.1} KiB", n / K)
    } else {
        format!("{} B", n as u64)
    }
}

fn format_elapsed(d: Duration) -> String {
    let secs = d.as_secs();
    format!(
        "{:02}:{:02}:{:02}",
        secs / 3600,
        (secs / 60) % 60,
        secs % 60
    )
}

/// Render loop. Owns the terminal; runs on a dedicated OS thread so it doesn't
/// contend with the tokio runtime or get pre-empted mid-frame.
fn render_thread(
    state: Arc<RenderState>,
    mut log_rx: mpsc::Receiver<LogMsg>,
) -> anyhow::Result<()> {
    let backend = CrosstermBackend::new(std::io::stderr());
    let viewport_height = (state.slots.len() + 1) as u16;
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(viewport_height),
        },
    )?;

    'outer: loop {
        // Drain any pending log messages (these get inserted ABOVE the inline
        // viewport, so they scroll into history naturally).
        loop {
            match log_rx.try_recv() {
                Ok(LogMsg::Line(line)) => {
                    terminal.insert_before(1, |buf| {
                        Paragraph::new(line).render(buf.area, buf);
                    })?;
                }
                Ok(LogMsg::Shutdown) => break 'outer,
                Err(mpsc::error::TryRecvError::Empty) => break,
                Err(mpsc::error::TryRecvError::Disconnected) => break 'outer,
            }
        }

        terminal.draw(|frame| draw_ui(frame, &state))?;
        std::thread::sleep(Duration::from_millis(100));
    }

    terminal.clear()?;
    Ok(())
}

fn draw_ui(frame: &mut ratatui::Frame, state: &RenderState) {
    let area = frame.area();
    let mut constraints = vec![Constraint::Length(1)]; // overall
    constraints.extend(std::iter::repeat_n(
        Constraint::Length(1),
        state.slots.len(),
    ));
    let chunks = Layout::vertical(constraints).split(area);

    // Overall bar
    let done = state.overall_done.load(Ordering::Relaxed);
    let total = state.overall_total;
    let ratio = if total > 0 {
        (done as f64 / total as f64).min(1.0)
    } else {
        0.0
    };
    let overall_label = format!(
        "Overall {}/{} regions  ({})",
        done,
        total,
        format_elapsed(state.start.elapsed())
    );
    let overall = Gauge::default()
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
        .ratio(ratio)
        .label(Span::styled(
            overall_label,
            Style::default().add_modifier(Modifier::BOLD),
        ));
    frame.render_widget(overall, chunks[0]);

    // Per-slot bars
    for (i, slot) in state.slots.iter().enumerate() {
        let area = chunks[i + 1];
        let active = slot.active.load(Ordering::Relaxed);
        if active {
            let pos = slot.pos.load(Ordering::Relaxed);
            let total = slot.total.load(Ordering::Relaxed);
            let ratio = if total > 0 {
                (pos as f64 / total as f64).min(1.0)
            } else {
                0.0
            };
            let label = slot.label.lock().unwrap().clone();
            let text = format!(
                "{} — {} / {}",
                label,
                format_bytes(pos),
                format_bytes(total)
            );
            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(Color::Cyan).bg(Color::Black))
                .ratio(ratio)
                .label(text);
            frame.render_widget(gauge, area);
        } else {
            let p = Paragraph::new(Line::from(Span::styled(
                "(idle)",
                Style::default().fg(Color::DarkGray),
            )));
            frame.render_widget(p, area);
        }
    }
}

/// Download a PBF, updating the given slot's shared state as bytes arrive.
async fn download_pbf(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    slot: &SlotShared,
    label: &str,
) -> anyhow::Result<()> {
    let resp = client
        .get(url)
        .send()
        .await?
        .error_for_status()
        .context("Failed to download PBF")?;

    let total_size = resp.content_length().unwrap_or(0);
    slot.begin(label, total_size);

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(dest)?;
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        slot.pos.fetch_add(chunk.len() as u64, Ordering::Relaxed);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

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
                && !if r.id.contains('-') {
                    {
                        // "us-midwest" -> check if "us/" sub-regions exist
                        let base = r.id.split('-').next().unwrap();
                        has_slash_children(base)
                    }
                } else {
                    false
                }
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

    // Split out already-processed regions (resumable).
    let total = filtered.len();
    let mut work: Vec<(String, String, String, PathBuf, PathBuf)> = Vec::new();
    let mut skipped = 0;
    for (idx, (region, path)) in filtered.iter().enumerate() {
        let roadpoints_path = output_dir.join(format!("{}.roadpoints", path));
        if roadpoints_path.exists() {
            skipped += 1;
            println!("[skip] {} (already exists)", path);
            continue;
        }
        let tmp_pbf = output_dir.join(format!(".tmp_download_{}.osm.pbf", idx));
        work.push((
            region.name.clone(),
            path.clone(),
            region.pbf_url.as_ref().unwrap().clone(),
            tmp_pbf,
            roadpoints_path,
        ));
    }

    if work.is_empty() {
        println!("Done! All {} regions already processed.", total);
        return Ok(());
    }

    println!(
        "Downloading {} region(s) with concurrency {} ({} already done)",
        work.len(),
        DOWNLOAD_CONCURRENCY,
        skipped
    );

    // Build shared render state and slot pool (just indices).
    let slots: Vec<Arc<SlotShared>> = (0..DOWNLOAD_CONCURRENCY)
        .map(|_| Arc::new(SlotShared::new()))
        .collect();
    let render_state = Arc::new(RenderState {
        slots: slots.clone(),
        overall_done: AtomicU64::new(skipped as u64),
        overall_total: total as u64,
        start: Instant::now(),
    });
    let slot_pool: Arc<Mutex<VecDeque<usize>>> =
        Arc::new(Mutex::new((0..DOWNLOAD_CONCURRENCY).collect()));

    let (log_tx, log_rx) = mpsc::channel::<LogMsg>(256);

    // Spawn the render thread (owns the terminal).
    let render_state_for_thread = render_state.clone();
    let render_handle = std::thread::spawn(move || render_thread(render_state_for_thread, log_rx));

    // Channel: download pool → preprocess worker.
    let (tx, mut rx) = mpsc::channel::<(String, String, PathBuf, PathBuf)>(DOWNLOAD_CONCURRENCY);

    let download_log = log_tx.clone();
    let download_client = client.clone();
    let download_pool = slot_pool.clone();
    let download_slots = slots.clone();
    let download_handle = tokio::spawn(async move {
        stream::iter(work)
            .for_each_concurrent(
                DOWNLOAD_CONCURRENCY,
                |(name, path, url, tmp_pbf, roadpoints)| {
                    let tx = tx.clone();
                    let client = download_client.clone();
                    let log_tx = download_log.clone();
                    let pool = download_pool.clone();
                    let slots = download_slots.clone();
                    async move {
                        let label = clean_label(&name);
                        let slot_idx = {
                            let mut p = pool.lock().await;
                            p.pop_front().expect("slot pool exhausted")
                        };
                        let slot = &slots[slot_idx];
                        let result = download_pbf(&client, &url, &tmp_pbf, slot, &label).await;
                        slot.end();
                        pool.lock().await.push_back(slot_idx);

                        match result {
                            Ok(()) => {
                                if tx.send((name, path, tmp_pbf, roadpoints)).await.is_err() {
                                    // Receiver dropped; nothing more we can do.
                                }
                            }
                            Err(e) => {
                                let _ = log_tx
                                    .send(LogMsg::Line(format!(
                                        "Error downloading {}: {} — skipping",
                                        path, e
                                    )))
                                    .await;
                                let _ = fs::remove_file(&tmp_pbf);
                            }
                        }
                    }
                },
            )
            .await;
    });

    // Preprocess worker (serial — preprocess_pbf already saturates CPU via rayon).
    let mut done = skipped;
    while let Some((name, path, tmp_pbf, roadpoints)) = rx.recv().await {
        done += 1;
        let pbf_size = fs::metadata(&tmp_pbf).map(|m| m.len()).unwrap_or(0);
        let label = clean_label(&name);
        let _ = log_tx
            .send(LogMsg::Line(format!(
                "[{}/{}] Preprocessing {} ({:.1} MB)...",
                done,
                total,
                label,
                pbf_size as f64 / 1_048_576.0
            )))
            .await;

        match preprocess_pbf(&tmp_pbf, &roadpoints) {
            Ok(points) => {
                let rp_size = fs::metadata(&roadpoints).map(|m| m.len()).unwrap_or(0);
                let _ = log_tx
                    .send(LogMsg::Line(format!(
                        "  Wrote {} ({} points, {:.1} MB)",
                        roadpoints.display(),
                        points,
                        rp_size as f64 / 1_048_576.0
                    )))
                    .await;
            }
            Err(e) => {
                let _ = log_tx
                    .send(LogMsg::Line(format!(
                        "Error preprocessing {}: {} — skipping",
                        path, e
                    )))
                    .await;
                let _ = fs::remove_file(&roadpoints);
            }
        }

        let _ = fs::remove_file(&tmp_pbf);
        render_state.overall_done.fetch_add(1, Ordering::Relaxed);
    }

    download_handle.await?;
    let _ = log_tx.send(LogMsg::Shutdown).await;
    let _ = render_handle.join();

    println!("Done! All regions processed.");
    Ok(())
}
