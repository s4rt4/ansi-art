use serde::{Deserialize, Serialize};

use crate::{
    color::{nearest, palette, ColorMode},
    options::RenderOptions,
    pixels::{luma, Pixels},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "kebab-case")]
pub enum Dither {
    None,
    /// Error diffusion, smooth gradients
    FloydSteinberg,
    /// Error diffusion with higher contrast (classic Mac look)
    Atkinson,
    /// Ordered 4×4 pattern, retro crosshatch
    Bayer4,
    /// Ordered 8×8 pattern, finer
    Bayer8,
}

impl Dither {
    pub const ALL: [Dither; 5] =
        [Dither::None, Dither::FloydSteinberg, Dither::Atkinson, Dither::Bayer4, Dither::Bayer8];
}

type Kernel = (&'static [(isize, usize, f32)], f32);

const FLOYD_STEINBERG: Kernel = (&[(1, 0, 7.0), (-1, 1, 3.0), (0, 1, 5.0), (1, 1, 1.0)], 16.0);
const ATKINSON: Kernel =
    (&[(1, 0, 1.0), (2, 0, 1.0), (-1, 1, 1.0), (0, 1, 1.0), (1, 1, 1.0), (0, 2, 1.0)], 8.0);

/// Quantizes pixels in place to the color mode's palette. Truecolor is left untouched.
pub fn apply(px: &mut Pixels, opts: &RenderOptions) {
    let Some(pal) = palette(opts.color) else { return };
    if opts.dither == Dither::None {
        return;
    }
    if opts.color == ColorMode::Mono {
        for p in &mut px.data {
            let l = luma(p);
            p[..3].fill(l);
        }
    }
    let at = opts.alpha_threshold;
    match opts.dither {
        Dither::None => {}
        Dither::FloydSteinberg => diffuse(px, pal, FLOYD_STEINBERG, at),
        Dither::Atkinson => diffuse(px, pal, ATKINSON, at),
        Dither::Bayer4 => ordered(px, pal, 4, spread(opts.color), at),
        Dither::Bayer8 => ordered(px, pal, 8, spread(opts.color), at),
    }
}

/// How far ordered dithering may push a value: roughly the palette's step size.
fn spread(mode: ColorMode) -> f32 {
    match mode {
        ColorMode::Mono => 1.0,
        ColorMode::Ansi8 => 0.5,
        ColorMode::Ansi16 => 0.35,
        ColorMode::Ansi256 | ColorMode::Truecolor => 0.15,
    }
}

fn diffuse(px: &mut Pixels, pal: &[[f32; 3]], (kernel, div): Kernel, at: f32) {
    let (w, h) = (px.width, px.height);
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if px.data[i][3] < at {
                continue;
            }
            let old = [0, 1, 2].map(|c| px.data[i][c].clamp(0.0, 1.0));
            let new = pal[nearest(pal, old)];
            px.data[i][..3].copy_from_slice(&new);
            let err = [0, 1, 2].map(|c| old[c] - new[c]);
            for &(dx, dy, wt) in kernel {
                let (nx, ny) = (x as isize + dx, y + dy);
                if nx < 0 || nx as usize >= w || ny >= h {
                    continue;
                }
                let n = &mut px.data[ny * w + nx as usize];
                if n[3] >= at {
                    for c in 0..3 {
                        n[c] += err[c] * wt / div;
                    }
                }
            }
        }
    }
}

fn ordered(px: &mut Pixels, pal: &[[f32; 3]], size: usize, spread: f32, at: f32) {
    let m = bayer(size);
    for y in 0..px.height {
        for x in 0..px.width {
            let p = &mut px.data[y * px.width + x];
            if p[3] < at {
                continue;
            }
            let t = m[(y % size) * size + x % size] * spread;
            let c = [0, 1, 2].map(|i| (p[i] + t).clamp(0.0, 1.0));
            p[..3].copy_from_slice(&pal[nearest(pal, c)]);
        }
    }
}

/// Bayer threshold matrix (size = power of two), normalized to -0.5..0.5.
fn bayer(size: usize) -> Vec<f32> {
    let mut m = vec![0u32];
    let mut n = 1;
    while n < size {
        let n2 = n * 2;
        let mut next = vec![0; n2 * n2];
        for y in 0..n {
            for x in 0..n {
                let v = 4 * m[y * n + x];
                next[y * n2 + x] = v;
                next[y * n2 + x + n] = v + 2;
                next[(y + n) * n2 + x] = v + 3;
                next[(y + n) * n2 + x + n] = v + 1;
            }
        }
        m = next;
        n = n2;
    }
    let cells = (n * n) as f32;
    m.into_iter().map(|v| (v as f32 + 0.5) / cells - 0.5).collect()
}
