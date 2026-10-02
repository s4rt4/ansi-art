//! One-pixel-per-cell modes and the half-block mode.

use super::{Ctx, luma, to_rgb};
use crate::cell::Cell;

pub(super) fn ramp(ctx: &Ctx, p: [f32; 4], ramp: &[char]) -> Cell {
    if !ctx.opaque(&p) {
        return Cell::EMPTY;
    }
    let idx = (luma(&p).clamp(0.0, 1.0) * (ramp.len() - 1) as f32).round() as usize;
    match ramp[idx] {
        ' ' => Cell::EMPTY,
        ch => ctx.cell(ch, to_rgb(&p), None),
    }
}

pub(super) fn full(ctx: &Ctx, p: [f32; 4]) -> Cell {
    let on = if ctx.colored { ctx.opaque(&p) } else { ctx.lit(&p) };
    if on { ctx.cell('█', to_rgb(&p), None) } else { Cell::EMPTY }
}

pub(super) fn half(ctx: &Ctx, top: [f32; 4], bottom: [f32; 4]) -> Cell {
    if !ctx.colored {
        return match (ctx.lit(&top), ctx.lit(&bottom)) {
            (false, false) => Cell::EMPTY,
            (true, false) => Cell { ch: '▀', fg: None, bg: None },
            (false, true) => Cell { ch: '▄', fg: None, bg: None },
            (true, true) => Cell { ch: '█', fg: None, bg: None },
        };
    }
    let (t, b) = (to_rgb(&top), to_rgb(&bottom));
    match (ctx.opaque(&top), ctx.opaque(&bottom)) {
        (false, false) => Cell::EMPTY,
        (true, false) => ctx.cell('▀', t, None),
        (false, true) => ctx.cell('▄', b, None),
        (true, true) if t == b => ctx.cell('█', t, None),
        (true, true) => ctx.cell('▀', t, Some(b)),
    }
}
