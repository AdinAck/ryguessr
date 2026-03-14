/// R2 quasirandom sequence constants (1/φ₃, 1/φ₃², 1/φ₃³)
const ALPHA: [f64; 3] = [0.8191725134, 0.6710436067, 0.5497004779];

/// OKLAB ranges that stay mostly within sRGB gamut
const L_RANGE: (f64, f64) = (0.35, 0.85);
const A_RANGE: (f64, f64) = (-0.15, 0.15);
const B_RANGE: (f64, f64) = (-0.15, 0.15);

/// A color generator that produces maximally distinct hex colors.
pub struct DistinctColors {
    seed: [f64; 3],
    index: usize,
}

impl DistinctColors {
    /// Create a new generator with a random seed.
    pub fn new() -> Self {
        let (x, y, z) = rand::random();
        Self {
            seed: [x, y, z],
            index: 0,
        }
    }

    pub fn decrement_index(&mut self) {
        self.index -= 1;
    }

    fn nth_oklab(&self, n: usize) -> [f64; 3] {
        let n_f = n as f64 + 1.0;
        let l_unit = (self.seed[0] + n_f * ALPHA[0]) % 1.0;
        let a_unit = (self.seed[1] + n_f * ALPHA[1]) % 1.0;
        let b_unit = (self.seed[2] + n_f * ALPHA[2]) % 1.0;

        [
            lerp(L_RANGE.0, L_RANGE.1, l_unit),
            lerp(A_RANGE.0, A_RANGE.1, a_unit),
            lerp(B_RANGE.0, B_RANGE.1, b_unit),
        ]
    }

    fn nth_hex(&self, n: usize) -> String {
        let [r, g, b] = oklab_to_srgb8(self.nth_oklab(n));
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}

impl Iterator for DistinctColors {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let color = self.nth_hex(self.index);
        self.index += 1;
        Some(color)
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

/// OKLAB -> linear sRGB via LMS intermediate.
fn oklab_to_linear_srgb(lab: [f64; 3]) -> [f64; 3] {
    let [l, a, b] = lab;

    // OKLAB -> LMS (cube roots)
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    // LMS -> linear sRGB
    [
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    ]
}

/// Linear sRGB -> sRGB with gamma correction and clamping.
fn linear_to_srgb(c: f64) -> f64 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// OKLAB [L, a, b] -> sRGB [R, G, B] as u8.
fn oklab_to_srgb8(lab: [f64; 3]) -> [u8; 3] {
    let lin = oklab_to_linear_srgb(lab);
    [
        (linear_to_srgb(lin[0]) * 255.0 + 0.5) as u8,
        (linear_to_srgb(lin[1]) * 255.0 + 0.5) as u8,
        (linear_to_srgb(lin[2]) * 255.0 + 0.5) as u8,
    ]
}
