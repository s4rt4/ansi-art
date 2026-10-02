//! Two-color block patterns: quadrants (2×2) and sextants (2×3).
//!
//! Each cell's pixels are split into a bright and a dark cluster; the bright
//! cluster becomes the glyph's "on" bits drawn in fg, the dark one fills bg.

use super::{average, Ctx};
use crate::{cell::Cell, color::Rgb};

/// Indexed by bits: 1 = top-left, 2 = top-right, 4 = bottom-left, 8 = bottom-right.
const QUADRANTS: [char; 16] =
    [' ', '▘', '▝', '▀', '▖', '▌', '▞', '▛', '▗', '▚', '▐', '▜', '▄', '▙', '▟', '█'];

pub(super) fn quadrant(ctx: &Ctx, px: &[[f32; 4]]) -> Cell {
    build(ctx, px, |bits| QUADRANTS[bits as usize])
}

pub(super) fn sextant(ctx: &Ctx, px: &[[f32; 4]]) -> Cell {
    build(ctx, px, sextant_char)
}

/// Bits run row by row, left then right (Unicode's "BLOCK SEXTANT-1..6" numbering).
/// The four patterns that already exist as older block characters are skipped by
/// the U+1FB00 range, so the code point is shifted past them.
fn sextant_char(bits: u32) -> char {
    match bits {
        0 => ' ',
        21 => '▌',
        42 => '▐',
        63 => '█',
        n => {
            let skipped = (n > 21) as u32 + (n > 42) as u32;
            char::from_u32(0x1FB00 + n - 1 - skipped).unwrap_or('?')
        }
    }
}

fn build(ctx: &Ctx, px: &[[f32; 4]], glyph: impl Fn(u32) -> char) -> Cell {
    let full = (1u32 << px.len()) - 1;
    if !ctx.colored {
        let bits = mask(px, |p| ctx.lit(p));
        return Cell { ch: glyph(bits), fg: None, bg: None };
    }
    let (bits, fg, bg) = split(ctx, px, full);
    match (bits, fg) {
        (0, _) | (_, None) => Cell::EMPTY,
        (b, Some(fg)) if b == full => ctx.cell('█', fg, None),
        (b, Some(fg)) => ctx.cell(glyph(b), fg, bg),
    }
}

fn mask(px: &[[f32; 4]], on: impl Fn(&[f32; 4]) -> bool) -> u32 {
    px.iter().enumerate().fold(0, |m, (i, p)| if on(p) { m | 1 << i } else { m })
}

/// Returns (fg bits, fg color, bg color).
fn split(ctx: &Ctx, px: &[[f32; 4]], full: u32) -> (u32, Option<Rgb>, Option<Rgb>) {
    let opaque = mask(px, |p| ctx.opaque(p));
    if opaque == 0 {
        return (0, None, None);
    }
    let pick = |m: u32| px.iter().enumerate().filter(move |(i, _)| m & (1 << i) != 0).map(|(_, p)| p);
    if opaque != full {
        // Transparent pixels show the terminal background, so only one color is left.
        return (opaque, Some(average(pick(opaque))), None);
    }

    // 2-means in RGB, seeded with the brightest and darkest pixel.
    let lum = |p: &[f32; 4]| super::luma(p);
    let mut hi = *px.iter().max_by(|a, b| lum(a).total_cmp(&lum(b))).unwrap();
    let mut lo = *px.iter().min_by(|a, b| lum(a).total_cmp(&lum(b))).unwrap();
    if dist(&hi, &lo) < 1e-4 {
        return (full, Some(average(px)), None);
    }
    let mut bits = 0;
    for _ in 0..3 {
        bits = mask(px, |p| dist(p, &hi) <= dist(p, &lo));
        if bits == 0 || bits == full {
            return (full, Some(average(px)), None);
        }
        hi = mean(pick(bits));
        lo = mean(pick(!bits & full));
    }
    (bits, Some(average(pick(bits))), Some(average(pick(!bits & full))))
}

fn dist(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    (0..3).map(|c| (a[c] - b[c]).powi(2)).sum()
}

fn mean<'a>(pixels: impl Iterator<Item = &'a [f32; 4]>) -> [f32; 4] {
    let (mut sum, mut n) = ([0.0f32; 4], 0.0);
    for p in pixels {
        for c in 0..3 {
            sum[c] += p[c];
        }
        n += 1.0;
    }
    [sum[0] / n, sum[1] / n, sum[2] / n, 1.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sextant_mapping_matches_unicode_names() {
        assert_eq!(sextant_char(0b000001), '\u{1FB00}'); // SEXTANT-1
        assert_eq!(sextant_char(0b000011), '\u{1FB02}'); // SEXTANT-12
        assert_eq!(sextant_char(0b010110), '\u{1FB14}'); // SEXTANT-235 (just past the 21 gap)
        assert_eq!(sextant_char(0b111110), '\u{1FB3B}'); // SEXTANT-23456, last in the block
        assert_eq!(sextant_char(21), '▌');
        assert_eq!(sextant_char(42), '▐');
    }

    #[test]
    fn quadrant_table_is_consistent() {
        assert_eq!(QUADRANTS[0b0011], '▀');
        assert_eq!(QUADRANTS[0b0101], '▌');
        assert_eq!(QUADRANTS[0b1001], '▚');
    }
}
