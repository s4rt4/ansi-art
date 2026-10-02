use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

pub type Rgb = [u8; 3];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "kebab-case")]
pub enum ColorMode {
    /// 24-bit `38;2;r;g;b` (Windows Terminal, iTerm2, kitty, GNOME, ...)
    Truecolor,
    /// xterm 256-color cube + grayscale ramp
    Ansi256,
    /// 16 colors incl. bright variants (actual hues depend on the terminal theme)
    Ansi16,
    /// 8 basic colors, for the most limited terminals
    Ansi8,
    /// No color codes at all
    Mono,
}

impl ColorMode {
    pub const ALL: [ColorMode; 5] =
        [ColorMode::Truecolor, ColorMode::Ansi256, ColorMode::Ansi16, ColorMode::Ansi8, ColorMode::Mono];
}

/// xterm's default 16-color palette; used as the reference when quantizing.
pub const XTERM16: [Rgb; 16] = [
    [0x00, 0x00, 0x00], [0xcd, 0x00, 0x00], [0x00, 0xcd, 0x00], [0xcd, 0xcd, 0x00],
    [0x00, 0x00, 0xee], [0xcd, 0x00, 0xcd], [0x00, 0xcd, 0xcd], [0xe5, 0xe5, 0xe5],
    [0x7f, 0x7f, 0x7f], [0xff, 0x00, 0x00], [0x00, 0xff, 0x00], [0xff, 0xff, 0x00],
    [0x5c, 0x5c, 0xff], [0xff, 0x00, 0xff], [0x00, 0xff, 0xff], [0xff, 0xff, 0xff],
];

/// Colors 16..=255 only: they are fixed, unlike 0..15 which every theme redefines.
pub static XTERM256: LazyLock<Vec<Rgb>> = LazyLock::new(|| {
    const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
    let mut pal = Vec::with_capacity(240);
    for r in LEVELS {
        for g in LEVELS {
            for b in LEVELS {
                pal.push([r, g, b]);
            }
        }
    }
    pal.extend((0..24u8).map(|i| [8 + 10 * i; 3]));
    pal
});

static PAL256_F: LazyLock<Vec<[f32; 3]>> = LazyLock::new(|| XTERM256.iter().map(|&c| to_f(c)).collect());
static PAL16_F: LazyLock<Vec<[f32; 3]>> = LazyLock::new(|| XTERM16.iter().map(|&c| to_f(c)).collect());
static PAL2_F: [[f32; 3]; 2] = [[0.0; 3], [1.0; 3]];

/// Palette a mode quantizes to, or `None` for truecolor.
pub fn palette(mode: ColorMode) -> Option<&'static [[f32; 3]]> {
    match mode {
        ColorMode::Truecolor => None,
        ColorMode::Ansi256 => Some(&PAL256_F),
        ColorMode::Ansi16 => Some(&PAL16_F),
        ColorMode::Ansi8 => Some(&PAL16_F[..8]),
        ColorMode::Mono => Some(&PAL2_F),
    }
}

#[inline]
pub fn to_f(c: Rgb) -> [f32; 3] {
    c.map(|v| v as f32 / 255.0)
}

/// Nearest palette entry using the "redmean" weighted distance (cheap, perceptually decent).
pub fn nearest(pal: &[[f32; 3]], c: [f32; 3]) -> usize {
    let mut best = 0;
    let mut best_d = f32::MAX;
    for (i, p) in pal.iter().enumerate() {
        let rm = (c[0] + p[0]) * 0.5;
        let (dr, dg, db) = (c[0] - p[0], c[1] - p[1], c[2] - p[2]);
        let d = (2.0 + rm) * dr * dr + 4.0 * dg * dg + (3.0 - rm) * db * db;
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    best
}

/// Weighted k-means in RGB. Returns up to `k` centers, most heavily weighted first.
/// Seeded farthest-first so small but distinct accents (a blue letter on green) survive.
pub fn kmeans(points: &[([f32; 3], f32)], k: usize) -> Vec<[f32; 3]> {
    let d2 = |a: &[f32; 3], b: &[f32; 3]| (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f32>();
    if points.is_empty() || k == 0 {
        return Vec::new();
    }
    let total: f32 = points.iter().map(|p| p.1).sum::<f32>().max(1e-6);
    let mean = [0, 1, 2].map(|i| points.iter().map(|(c, w)| c[i] * w).sum::<f32>() / total);

    let mut centers = vec![mean];
    while centers.len() < k {
        let far = points
            .iter()
            .map(|(c, _)| (c, centers.iter().map(|m| d2(c, m)).fold(f32::MAX, f32::min)))
            .max_by(|a, b| a.1.total_cmp(&b.1));
        match far {
            Some((c, d)) if d > 1e-4 => centers.push(*c),
            _ => break,
        }
    }

    let mut weights = vec![0.0; centers.len()];
    for _ in 0..12 {
        let mut sums = vec![[0.0f32; 3]; centers.len()];
        weights.iter_mut().for_each(|w| *w = 0.0);
        for (c, w) in points {
            let j = (0..centers.len()).min_by(|&a, &b| d2(c, &centers[a]).total_cmp(&d2(c, &centers[b]))).unwrap();
            for i in 0..3 {
                sums[j][i] += c[i] * w;
            }
            weights[j] += w;
        }
        for (j, center) in centers.iter_mut().enumerate() {
            if weights[j] > 0.0 {
                *center = sums[j].map(|s| s / weights[j]);
            }
        }
    }

    let mut ranked: Vec<_> = centers.into_iter().zip(weights).filter(|(_, w)| *w > 0.0).collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    ranked.into_iter().map(|(c, _)| c).collect()
}

/// SGR parameters selecting `c` as foreground (or background) in `mode`.
pub fn sgr(mode: ColorMode, c: Rgb, bg: bool) -> Option<String> {
    let off = if bg { 10 } else { 0 };
    Some(match mode {
        ColorMode::Mono => return None,
        ColorMode::Truecolor => format!("{};2;{};{};{}", 38 + off, c[0], c[1], c[2]),
        ColorMode::Ansi256 => format!("{};5;{}", 38 + off, 16 + nearest(&PAL256_F, to_f(c))),
        ColorMode::Ansi16 => match nearest(&PAL16_F, to_f(c)) {
            i @ 0..=7 => (30 + off + i).to_string(),
            i => (90 + off + i - 8).to_string(),
        },
        ColorMode::Ansi8 => (30 + off + nearest(&PAL16_F[..8], to_f(c))).to_string(),
    })
}

/// Palette index (0..=255) for modes that have one; truecolor maps into the 256-color cube.
pub fn ansi_index(mode: ColorMode, c: Rgb) -> Option<u8> {
    let f = to_f(c);
    Some(match mode {
        ColorMode::Truecolor | ColorMode::Ansi256 => 16 + nearest(&PAL256_F, f) as u8,
        ColorMode::Ansi16 => nearest(&PAL16_F, f) as u8,
        ColorMode::Ansi8 => nearest(&PAL16_F[..8], f) as u8,
        ColorMode::Mono => return None,
    })
}

/// The color `c` becomes after quantizing to `mode` (as xterm would display it).
pub fn quantize(mode: ColorMode, c: Rgb) -> Rgb {
    match mode {
        ColorMode::Truecolor => c,
        ColorMode::Ansi256 => XTERM256[nearest(&PAL256_F, to_f(c))],
        ColorMode::Ansi16 => XTERM16[nearest(&PAL16_F, to_f(c))],
        ColorMode::Ansi8 => XTERM16[nearest(&PAL16_F[..8], to_f(c))],
        ColorMode::Mono => XTERM16[7],
    }
}
