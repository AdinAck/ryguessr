use anyhow::{anyhow, bail};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub use macros::srgb;

/// OKLAB ranges that stay mostly within the sRGB gamut.
pub const L_RANGE: (f64, f64) = (0.35, 0.85);
pub const A_RANGE: (f64, f64) = (-0.15, 0.15);
pub const B_RANGE: (f64, f64) = (-0.15, 0.15);

#[derive(Debug, Clone, Copy)]
pub struct Oklab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

impl Oklab {
    pub fn dist_sq(self, other: Oklab) -> f64 {
        (self.l - other.l).powi(2) + (self.a - other.a).powi(2) + (self.b - other.b).powi(2)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LinearRgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Srgb8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<Srgb8> for Oklab {
    fn from(srgb: Srgb8) -> Self {
        Oklab::from(LinearRgb::from(srgb))
    }
}

impl TryFrom<&str> for Srgb8 {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> anyhow::Result<Self> {
        let hex = s.strip_prefix('#').unwrap_or(s);
        if hex.len() != 6 || !hex.is_ascii() {
            bail!("invalid hex color {s:?}, expected the form #RRGGBB");
        }
        let parse = |i: usize| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| anyhow!("invalid hex color {s:?}: {e}"))
        };
        Ok(Srgb8 {
            r: parse(0)?,
            g: parse(2)?,
            b: parse(4)?,
        })
    }
}

impl std::fmt::Display for Srgb8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { r, g, b } = *self;
        write!(f, "#{r:02x}{g:02x}{b:02x}")
    }
}

impl Serialize for Srgb8 {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Srgb8 {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = <&str>::deserialize(d)?;
        Srgb8::try_from(s).map_err(serde::de::Error::custom)
    }
}

impl From<Srgb8> for LinearRgb {
    fn from(Srgb8 { r, g, b }: Srgb8) -> Self {
        let decode = |c: u8| {
            let c = c as f64 / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        LinearRgb {
            r: decode(r),
            g: decode(g),
            b: decode(b),
        }
    }
}

impl From<LinearRgb> for Srgb8 {
    fn from(LinearRgb { r, g, b }: LinearRgb) -> Self {
        let encode = |c: f64| {
            let c = c.clamp(0.0, 1.0);
            let gamma = if c <= 0.0031308 {
                12.92 * c
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            };
            (gamma * 255.0 + 0.5) as u8
        };
        Srgb8 {
            r: encode(r),
            g: encode(g),
            b: encode(b),
        }
    }
}

impl From<LinearRgb> for Oklab {
    fn from(LinearRgb { r, g, b }: LinearRgb) -> Self {
        // linear sRGB -> LMS
        let l_ = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
        let m_ = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
        let s_ = 0.0883024619 * r + 0.2817188976 * g + 0.6299787005 * b;

        let l = l_.cbrt();
        let m = m_.cbrt();
        let s = s_.cbrt();

        Oklab {
            l: 0.2104542553 * l + 0.7936177850 * m - 0.0040720403 * s,
            a: 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
            b: 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
        }
    }
}

impl From<Oklab> for LinearRgb {
    fn from(Oklab { l, a, b }: Oklab) -> Self {
        // OKLAB -> LMS
        let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
        let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
        let s_ = l - 0.0894841775 * a - 1.2914855480 * b;

        let l = l_.powi(3);
        let m = m_.powi(3);
        let s = s_.powi(3);

        LinearRgb {
            r: 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
            g: -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
            b: -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
        }
    }
}
