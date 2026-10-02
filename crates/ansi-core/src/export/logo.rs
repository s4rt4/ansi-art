//! Custom logos for neofetch (`${c1}`) and fastfetch (`$1`): plain text with color
//! placeholders, and the actual colors passed on the command line (see the hint).
//!
//! Only foreground colors survive; background colors (half-block etc.) are dropped,
//! so this suits the glyph, ascii and braille modes best.

use std::collections::HashMap;

use super::Output;
use crate::{
    cell::Grid,
    color::{ansi_index, kmeans, nearest, sgr, to_f, ColorMode, Rgb},
    pixels::to_rgb,
};

/// neofetch knows `${c1}`..`${c6}`.
const NEOFETCH_MAX: usize = 6;
/// fastfetch knows `$1`..`$9`.
const FASTFETCH_MAX: usize = 9;

pub fn neofetch(grid: &Grid, mode: ColorMode) -> Output {
    // neofetch prints the logo with `printf '%b'`, so backslashes must be doubled.
    let (text, colors) = build(grid, mode, NEOFETCH_MAX, |i| format!("${{c{i}}}"), |ch, out| match ch {
        '\\' => out.push_str("\\\\"),
        ch => out.push(ch),
    });
    let mut hint = String::from("neofetch --ascii {file}");
    if !colors.is_empty() {
        let indices: Vec<String> =
            colors.iter().filter_map(|&c| ansi_index(mode, c)).map(|i| i.to_string()).collect();
        hint.push_str(" --ascii_colors ");
        hint.push_str(&indices.join(" "));
    }
    Output { text, hint: Some(hint) }
}

pub fn fastfetch(grid: &Grid, mode: ColorMode) -> Output {
    let (text, colors) = build(grid, mode, FASTFETCH_MAX, |i| format!("${i}"), |ch, out| match ch {
        '$' => out.push_str("$$"),
        ch => out.push(ch),
    });
    let mut hint = String::from("fastfetch --logo {file} --logo-type file");
    for (i, &c) in colors.iter().enumerate() {
        if let Some(code) = sgr(mode, c, false) {
            hint.push_str(&format!(" --logo-color-{} '{code}'", i + 1));
        }
    }
    Output { text, hint: Some(hint) }
}

/// Returns the placeholder text and the colors behind placeholders 1..=n.
fn build(
    grid: &Grid,
    mode: ColorMode,
    max_colors: usize,
    placeholder: impl Fn(usize) -> String,
    escape: impl Fn(char, &mut String),
) -> (String, Vec<Rgb>) {
    let palette = if mode == ColorMode::Mono { Vec::new() } else { palette(grid, max_colors) };

    let mut out = String::new();
    for y in 0..grid.rows {
        // Re-state the color on every line: not every tool carries it across lines.
        let mut current = None;
        for cell in grid.trimmed_row(y) {
            if let (false, Some(fg), false) = (cell.ch == ' ', cell.fg, palette.is_empty()) {
                let idx = nearest(&palette, to_f(fg));
                if current != Some(idx) {
                    out.push_str(&placeholder(idx + 1));
                    current = Some(idx);
                }
            }
            escape(cell.ch, &mut out);
        }
        out.push('\n');
    }
    let colors = palette.iter().map(|c| to_rgb(&[c[0], c[1], c[2], 1.0])).collect();
    (out, colors)
}

/// The grid's foreground colors, merged down to `max` with k-means when there are more.
fn palette(grid: &Grid, max: usize) -> Vec<[f32; 3]> {
    let mut counts: HashMap<Rgb, f32> = HashMap::new();
    for cell in grid.cells.iter().filter(|c| c.ch != ' ') {
        if let Some(fg) = cell.fg {
            *counts.entry(fg).or_default() += 1.0;
        }
    }
    let points: Vec<([f32; 3], f32)> = counts.into_iter().map(|(c, n)| (to_f(c), n)).collect();
    if points.len() <= max {
        let mut points = points;
        // Most used first; ties broken by color so output is deterministic.
        points.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.partial_cmp(&b.0).unwrap()));
        return points.into_iter().map(|(c, _)| c).collect();
    }
    kmeans(&points, max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Cell;

    fn grid() -> Grid {
        let green = Some([0, 200, 0]);
        let blue = Some([0, 0, 255]);
        let cells = vec![
            Cell { ch: 'M', fg: green, bg: None },
            Cell { ch: 'M', fg: green, bg: None },
            Cell { ch: 'M', fg: green, bg: None },
            Cell { ch: '\\', fg: blue, bg: None },
            Cell { ch: '$', fg: blue, bg: None },
        ];
        Grid { cols: 5, rows: 1, cells }
    }

    #[test]
    fn neofetch_placeholders_and_escapes() {
        let out = neofetch(&grid(), ColorMode::Ansi256);
        assert_eq!(out.text, "${c1}MMM${c2}\\\\$\n");
        assert_eq!(out.hint.unwrap(), "neofetch --ascii {file} --ascii_colors 40 21");
    }

    #[test]
    fn fastfetch_placeholders_and_escapes() {
        let out = fastfetch(&grid(), ColorMode::Truecolor);
        assert_eq!(out.text, "$1MMM$2\\$$\n");
        assert!(out.hint.unwrap().ends_with("--logo-color-1 '38;2;0;200;0' --logo-color-2 '38;2;0;0;255'"));
    }
}
