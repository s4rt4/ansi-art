//! Shape-matched ASCII, the hand-drawn distro-logo look.
//!
//! 1. Work out which pixels are "ink" (alpha, or distance from a detected background).
//! 2. Reduce ink to a few flat colors with k-means.
//! 3. Per cell, take the dominant color's coverage mask and pick the glyph whose
//!    rasterized shape (from the bundled font) puts its ink in the same place:
//!    `.` `,` for bottom edges, `'` `` ` `` for top edges, `:` `|` for sides, the
//!    fill character for solid areas.

use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use ab_glyph::{point, Font, FontRef, PxScale, ScaleFont};

use super::Ctx;
use crate::{
    cell::Grid,
    color::{kmeans, ColorMode},
    options::RenderOptions,
    pixels::{to_rgb, Pixels},
};

/// Sub-cell sampling grid; also the resolution glyph shapes are compared at.
pub const GW: usize = 4;
pub const GH: usize = 8;
const N: usize = GW * GH;

type Mask = [f32; N];

pub const CHARSETS: &[(&str, &str)] = &[
    ("edges", ".,:;'`-_~^\"!|/\\()<>+=*"),
    ("letters", ".,:;'`-_~^\"!|/\\()<>+=*0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"),
    ("all", "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~"),
    ("blocks", "▀▄▌▐▖▗▘▝▚▞▙▛▜▟░▒▓"),
];

/// Cells with less coverage than this stay blank.
const EMPTY_LEVEL: f32 = 0.06;
/// How much a glyph's ink density must agree with the cell's coverage, vs. ink position.
const DENSITY_WEIGHT: f32 = 1.2;

static FONT: &[u8] = include_bytes!("../../assets/JetBrainsMono-Regular.ttf");
static MASKS: LazyLock<Mutex<HashMap<char, Option<Mask>>>> = LazyLock::new(Default::default);

pub fn resolve_charset(name_or_chars: &str) -> &str {
    CHARSETS.iter().find(|(name, _)| *name == name_or_chars).map_or(name_or_chars, |(_, chars)| chars)
}

struct Candidate {
    ch: char,
    /// Ink distribution, summing to 1.
    shape: Mask,
    /// Ink amount relative to the densest candidate, 0..1.
    density: f32,
}

pub(super) fn render(ctx: &Ctx, px: &Pixels, cols: usize, rows: usize, opts: &RenderOptions) -> Grid {
    let g = &opts.glyph;
    let coverage = coverage(px, opts.alpha_threshold);

    let k = if opts.color == ColorMode::Mono { 1 } else { g.colors.clamp(1, 8) as usize };
    let ink: Vec<_> = px
        .data
        .iter()
        .zip(&coverage)
        .filter(|(_, &c)| c >= 0.5)
        .map(|(p, &c)| ([p[0], p[1], p[2]], c))
        .collect();
    let palette = kmeans(&ink, k);
    let label: Vec<usize> = px
        .data
        .iter()
        .map(|p| crate::color::nearest(&palette, [p[0], p[1], p[2]]))
        .collect();

    let candidates = candidates(resolve_charset(&g.charset), g.fill.chars().next());
    let fill = g.fill.chars().next().filter(|c| !c.is_whitespace());

    let mut grid = Grid::new(cols, rows);
    if palette.is_empty() || candidates.is_empty() {
        return grid;
    }
    let mut weight = vec![0.0f32; palette.len()];
    for y in 0..rows {
        for x in 0..cols {
            let at = |i: usize| (y * GH + i / GW) * px.width + x * GW + i % GW;

            weight.iter_mut().for_each(|w| *w = 0.0);
            for i in 0..N {
                weight[label[at(i)]] += coverage[at(i)];
            }
            let (color, _) = weight.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).unwrap();

            let mut target = [0.0f32; N];
            for (i, t) in target.iter_mut().enumerate() {
                if label[at(i)] == color {
                    *t = coverage[at(i)];
                }
            }
            let level = target.iter().sum::<f32>() / N as f32;
            if level < EMPTY_LEVEL {
                continue;
            }
            let ch = match fill {
                Some(f) if level >= g.fill_level => f,
                _ => best_match(&target, level, &candidates),
            };
            let c = palette[color];
            grid.set(x, y, ctx.cell(ch, to_rgb(&[c[0], c[1], c[2], 1.0]), None));
        }
    }
    grid
}

fn best_match(target: &Mask, level: f32, candidates: &[Candidate]) -> char {
    let total: f32 = target.iter().sum();
    let score = |c: &Candidate| {
        let shape: f32 = target.iter().zip(&c.shape).map(|(t, g)| (t / total - g).abs()).sum();
        shape + DENSITY_WEIGHT * (level - c.density).abs()
    };
    candidates.iter().min_by(|a, b| score(a).total_cmp(&score(b))).map_or(' ', |c| c.ch)
}

/// Ink per pixel, 0..1. Transparent images use alpha; opaque ones measure the
/// distance from the background color, estimated from the image border.
fn coverage(px: &Pixels, alpha_threshold: f32) -> Vec<f32> {
    let transparent = px.data.iter().any(|p| p[3] < alpha_threshold);
    if transparent {
        return px.data.iter().map(|p| p[3]).collect();
    }
    let (w, h) = (px.width, px.height);
    let border: Vec<[f32; 4]> = (0..w)
        .flat_map(|x| [px.get(x, 0), px.get(x, h - 1)])
        .chain((0..h).flat_map(|y| [px.get(0, y), px.get(w - 1, y)]))
        .collect();
    let bg = [0, 1, 2].map(|i| border.iter().map(|p| p[i]).sum::<f32>() / border.len() as f32);
    px.data
        .iter()
        .map(|p| {
            let d = (0..3).map(|i| (p[i] - bg[i]).powi(2)).sum::<f32>().sqrt();
            ((d - 0.06) / 0.14).clamp(0.0, 1.0)
        })
        .collect()
}

fn candidates(charset: &str, fill: Option<char>) -> Vec<Candidate> {
    let mut chars: Vec<char> = charset.chars().filter(|c| !c.is_whitespace() && !c.is_control()).collect();
    chars.extend(fill);
    chars.sort_unstable();
    chars.dedup();

    let mut cache = MASKS.lock().unwrap();
    let masks: Vec<(char, Mask)> =
        chars.into_iter().filter_map(|ch| cache.entry(ch).or_insert_with(|| rasterize(ch)).map(|m| (ch, m))).collect();

    let mean = |m: &Mask| m.iter().sum::<f32>() / N as f32;
    let densest = masks.iter().map(|(_, m)| mean(m)).fold(0.0, f32::max).max(1e-6);
    masks
        .into_iter()
        .map(|(ch, m)| {
            let sum: f32 = m.iter().sum();
            Candidate { ch, shape: m.map(|v| v / sum), density: mean(&m) / densest }
        })
        .collect()
}

/// Renders `ch` into one terminal cell (advance width × ascent-to-descent) and
/// averages it down to the GW×GH grid. `None` if the font lacks the glyph or it has no ink.
fn rasterize(ch: char) -> Option<Mask> {
    const HEIGHT: f32 = 96.0;
    let font = FontRef::try_from_slice(FONT).ok()?;
    let id = font.glyph_id(ch);
    if id.0 == 0 {
        return None;
    }
    let scale = PxScale::from(HEIGHT);
    let sf = font.as_scaled(scale);
    let (cw, chh) = (sf.h_advance(id).round().max(1.0) as usize, (sf.ascent() - sf.descent()).round() as usize);

    let mut buf = vec![0.0f32; cw * chh];
    let outline = font.outline_glyph(id.with_scale_and_position(scale, point(0.0, sf.ascent())))?;
    let b = outline.px_bounds();
    outline.draw(|x, y, c| {
        let (px, py) = (b.min.x as i32 + x as i32, b.min.y as i32 + y as i32);
        if px >= 0 && py >= 0 && (px as usize) < cw && (py as usize) < chh {
            buf[py as usize * cw + px as usize] = c;
        }
    });

    let mut mask = [0.0f32; N];
    for gy in 0..GH {
        let (y0, y1) = (gy * chh / GH, ((gy + 1) * chh / GH).max(gy * chh / GH + 1));
        for gx in 0..GW {
            let (x0, x1) = (gx * cw / GW, ((gx + 1) * cw / GW).max(gx * cw / GW + 1));
            let sum: f32 = (y0..y1).flat_map(|y| (x0..x1).map(move |x| (x, y))).map(|(x, y)| buf[y * cw + x]).sum();
            mask[gy * GW + gx] = sum / ((y1 - y0) * (x1 - x0)) as f32;
        }
    }
    (mask.iter().sum::<f32>() > 1e-3).then_some(mask)
}
