use std::path::Path;

use memmap2::Mmap;
use tracing::{instrument, trace};

const POINT_SIZE: usize = 8;

pub struct RegionData {
    pub name: String,
    pub mmap: Mmap,
    pub count: usize,
}

impl RegionData {
    pub fn get_point(&self, index: usize) -> (f64, f64) {
        let offset = index * POINT_SIZE;
        let lat = f32::from_le_bytes(self.mmap[offset..offset + 4].try_into().unwrap());
        let lng = f32::from_le_bytes(self.mmap[offset + 4..offset + 8].try_into().unwrap());
        (lat as f64, lng as f64)
    }
}

#[instrument(fields(dir = %dir.display()))]
pub fn load_all_regions(dir: &Path) -> anyhow::Result<Vec<RegionData>> {
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
