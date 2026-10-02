//! Cell renderers. Each one looks at the sub-cell pixels behind a character cell
//! and picks a glyph plus foreground/background colors.

mod blocks;
mod braille;
pub mod glyph;
mod pattern;
pub mod ramps;

use crate::{
    cell::{Cell, Grid},
    color::{ColorMode, Rgb},
    options::{Mode, RenderOptions},
    pixels::{luma, to_rgb, Pixels},
};

pub fn render(px: &Pixels, cols: usize, rows: usize, opts: &RenderOptions) -> Grid {
    let ctx = Ctx {
        colored: opts.color != ColorMode::Mono,
        threshold: opts.threshold,
        alpha_threshold: opts.alpha_threshold,
    };
    if opts.mode == Mode::Glyph {
        // Needs whole-image analysis (background, palette) before looking at cells.
        return glyph::render(&ctx, px, cols, rows, opts);
    }
    let ramp = match opts.mode {
        Mode::Shade => ramps::resolve("blocks"),
        _ => ramps::resolve(&opts.ramp),
    };
    let (sw, sh) = opts.mode.subcell();
    let mut grid = Grid::new(cols, rows);
    let mut buf = Vec::with_capacity(sw * sh);
    for y in 0..rows {
        for x in 0..cols {
            buf.clear();
            for dy in 0..sh {
                for dx in 0..sw {
                    buf.push(px.get(x * sw + dx, y * sh + dy));
                }
            }
            let cell = match opts.mode {
                Mode::Ascii | Mode::Shade => blocks::ramp(&ctx, buf[0], &ramp),
                Mode::Block => blocks::full(&ctx, buf[0]),
                Mode::HalfBlock => blocks::half(&ctx, buf[0], buf[1]),
                Mode::Quadrant => pattern::quadrant(&ctx, &buf),
                Mode::Sextant => pattern::sextant(&ctx, &buf),
                Mode::Braille => braille::cell(&ctx, &buf),
                Mode::Glyph => unreachable!("handled above"),
            };
            grid.set(x, y, cell);
        }
    }
    grid
}

struct Ctx {
    colored: bool,
    threshold: f32,
    alpha_threshold: f32,
}

impl Ctx {
    fn opaque(&self, p: &[f32; 4]) -> bool {
        p[3] >= self.alpha_threshold
    }

    /// "Ink" test for monochrome output: visible and bright enough.
    fn lit(&self, p: &[f32; 4]) -> bool {
        self.opaque(p) && luma(p) >= self.threshold
    }

    fn color(&self, c: Rgb) -> Option<Rgb> {
        self.colored.then_some(c)
    }

    fn cell(&self, ch: char, fg: Rgb, bg: Option<Rgb>) -> Cell {
        Cell { ch, fg: self.color(fg), bg: bg.and_then(|c| self.color(c)) }
    }
}

fn average<'a>(pixels: impl IntoIterator<Item = &'a [f32; 4]>) -> Rgb {
    let mut sum = [0.0f32; 4];
    for p in pixels {
        for c in 0..3 {
            sum[c] += p[c];
        }
        sum[3] += 1.0;
    }
    let n = sum[3].max(1.0);
    to_rgb(&[sum[0] / n, sum[1] / n, sum[2] / n, 1.0])
}
