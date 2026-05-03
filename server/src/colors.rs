use colors::{A_RANGE, B_RANGE, L_RANGE, LinearRgb, Oklab, Srgb8, srgb};

/// R2 quasirandom sequence constants (1/φ₃, 1/φ₃², 1/φ₃³). Walking these in 3D
/// produces a low-discrepancy sequence, so successive indices map to evenly
/// spread points in OKLAB space.
const ALPHA: [f64; 3] = [0.8191725134, 0.6710436067, 0.5497004779];

/// Squared Delta-E threshold below which two player colors are considered too similar.
/// 0.04 in OKLAB is a small but visible perceptual difference.
const DISTINCT_THRESHOLD_SQ: f64 = 0.04 * 0.04;

/// Larger threshold used against map background colors. Player markers cover a
/// small area, so they need more separation from large background regions to
/// stay visible.
const MAP_DISTINCT_THRESHOLD_SQ: f64 = 0.12 * 0.12;

/// Maximum R2 indices [`DistinctColors::next`] probes before giving up. Sized
/// well above the realistic rejection rate; a return of `None` means the
/// reserved region is genuinely saturated.
const MAX_DISTINCT_TRIES: usize = 1024;

/// Colors sampled from the Google Maps base layer (terrain, parks, water, road
/// fill, etc.). Generated player colors must stay perceptually distinct from
/// these so markers don't blend into the map.
const MAP_RESERVED_COLORS: &[Srgb8] = &[
    srgb!("#f6f5f5"),
    srgb!("#f5f0e5"),
    srgb!("#bbf0d0"),
    srgb!("#cdf1dc"),
    srgb!("#ccd0cb"),
    srgb!("#70d3e7"),
];

/// A color assigned to a member of a room.
#[derive(Debug, Clone, Copy)]
pub struct PlayerColor {
    pub srgb: Srgb8,
    /// `true` if the client picked it explicitly, `false` if the server
    /// generated it via [`DistinctColors`]. Custom colors are preserved across
    /// regenerations.
    pub custom: bool,
}

impl PlayerColor {
    pub fn custom(srgb: Srgb8) -> Self {
        Self { srgb, custom: true }
    }

    pub fn distinct(srgb: Srgb8) -> Self {
        Self {
            srgb,
            custom: false,
        }
    }
}

/// Generates colors that are perceptually distinct from one another.
///
/// Each instance walks an R2 quasirandom sequence in OKLAB space starting from
/// a random seed, so two `DistinctColors` will produce different palettes even
/// at the same index.
pub struct DistinctColors {
    seed: [f64; 3],
    next_index: usize,
    occupied: Vec<Srgb8>,
}

impl DistinctColors {
    pub fn new() -> Self {
        let (x, y, z) = rand::random();
        Self {
            seed: [x, y, z],
            next_index: 0,
            occupied: Vec::new(),
        }
    }

    /// Register a color as occupied so future iterations avoid it.
    pub fn push_occupied(&mut self, srgb: Srgb8) {
        self.occupied.push(srgb);
    }

    /// Free a previously-occupied color so it becomes eligible again.
    pub fn remove_occupied(&mut self, srgb: Srgb8) {
        if let Some(pos) = self.occupied.iter().position(|c| *c == srgb) {
            self.occupied.remove(pos);
        }
    }

    fn nth_oklab(&self, n: usize) -> Oklab {
        // Offset by 1 so index 0 produces a color distinct from the seed itself.
        let step = (n + 1) as f64;
        Oklab {
            l: lerp(L_RANGE.0, L_RANGE.1, (self.seed[0] + step * ALPHA[0]) % 1.0),
            a: lerp(A_RANGE.0, A_RANGE.1, (self.seed[1] + step * ALPHA[1]) % 1.0),
            b: lerp(B_RANGE.0, B_RANGE.1, (self.seed[2] + step * ALPHA[2]) % 1.0),
        }
    }

    fn nth_srgb(&self, n: usize) -> Srgb8 {
        LinearRgb::from(self.nth_oklab(n)).into()
    }
}

/// Yields colors distinct from the map background and from anything already
/// passed to [`DistinctColors::push_occupied`] or returned by a prior call.
/// Returns `None` only if [`MAX_DISTINCT_TRIES`] consecutive samples all clash.
impl Iterator for DistinctColors {
    type Item = PlayerColor;

    fn next(&mut self) -> Option<Self::Item> {
        let member_oklabs: Vec<Oklab> = self.occupied.iter().copied().map(Oklab::from).collect();
        let map_oklabs: Vec<Oklab> = MAP_RESERVED_COLORS
            .iter()
            .copied()
            .map(Oklab::from)
            .collect();

        let is_distinct = |oklab: Oklab| {
            member_oklabs
                .iter()
                .all(|&other| oklab.dist_sq(other) >= DISTINCT_THRESHOLD_SQ)
                && map_oklabs
                    .iter()
                    .all(|&other| oklab.dist_sq(other) >= MAP_DISTINCT_THRESHOLD_SQ)
        };

        for _ in 0..MAX_DISTINCT_TRIES {
            let idx = self.next_index;
            self.next_index += 1;
            if is_distinct(self.nth_oklab(idx)) {
                let srgb = self.nth_srgb(idx);
                self.occupied.push(srgb);
                return Some(PlayerColor::distinct(srgb));
            }
        }
        None
    }
}

impl Default for DistinctColors {
    fn default() -> Self {
        Self::new()
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
