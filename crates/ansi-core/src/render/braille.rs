//! Braille patterns (U+2800): 2×4 dots per cell, foreground color only.

use super::{average, Ctx};
use crate::cell::Cell;

/// Dot bit for each (row, column) of the 2×4 grid, per the Unicode braille layout.
const DOTS: [[u32; 2]; 4] = [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];

pub(super) fn cell(ctx: &Ctx, px: &[[f32; 4]]) -> Cell {
    let mut bits = 0;
    for (i, p) in px.iter().enumerate() {
        if ctx.lit(p) {
            bits |= DOTS[i / 2][i % 2];
        }
    }
    if bits == 0 {
        return Cell::EMPTY;
    }
    let ch = char::from_u32(0x2800 + bits).unwrap_or(' ');
    let fg = average(px.iter().filter(|p| ctx.lit(p)));
    ctx.cell(ch, fg, None)
}
