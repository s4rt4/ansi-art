use std::collections::HashMap;

use crate::{
    cell::Grid,
    color::{sgr, ColorMode, Rgb},
};

/// Encodes the grid as SGR escape sequences, emitting codes only when they change.
/// Every line ends reset so the art can be printed, cut or concatenated safely.
pub fn encode(grid: &Grid, mode: ColorMode) -> String {
    let mut cache: HashMap<(Rgb, bool), Option<String>> = HashMap::new();
    let mut code = |c: Option<Rgb>, bg: bool| -> Option<String> {
        c.and_then(|c| cache.entry((c, bg)).or_insert_with(|| sgr(mode, c, bg)).clone())
    };

    let mut out = String::with_capacity(grid.cols * grid.rows * 8);
    for y in 0..grid.rows {
        let (mut fg, mut bg): (Option<String>, Option<String>) = (None, None);
        for cell in grid.trimmed_row(y) {
            let want_bg = code(cell.bg, true);
            // A space never shows its foreground, so keep whatever is active.
            let want_fg = if cell.ch == ' ' { fg.clone() } else { code(cell.fg, false) };

            if want_fg != fg || want_bg != bg {
                let mut params: Vec<&str> = Vec::new();
                let dropped = (want_fg.is_none() && fg.is_some()) || (want_bg.is_none() && bg.is_some());
                if dropped {
                    params.push("0");
                }
                if want_fg.is_some() && (dropped || want_fg != fg) {
                    params.push(want_fg.as_deref().unwrap());
                }
                if want_bg.is_some() && (dropped || want_bg != bg) {
                    params.push(want_bg.as_deref().unwrap());
                }
                out.push_str("\x1b[");
                out.push_str(&params.join(";"));
                out.push('m');
                fg = want_fg;
                bg = want_bg;
            }
            out.push(cell.ch);
        }
        if fg.is_some() || bg.is_some() {
            out.push_str("\x1b[0m");
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Cell;

    fn grid(cells: Vec<Cell>) -> Grid {
        Grid { cols: cells.len(), rows: 1, cells }
    }

    #[test]
    fn emits_codes_only_on_change_and_resets_line() {
        let red = Some([255, 0, 0]);
        let g = grid(vec![
            Cell { ch: '█', fg: red, bg: None },
            Cell { ch: '█', fg: red, bg: None },
            Cell::EMPTY,
        ]);
        assert_eq!(encode(&g, ColorMode::Truecolor), "\x1b[38;2;255;0;0m██\x1b[0m\n");
    }

    #[test]
    fn dropping_a_color_resets_then_restores_the_other() {
        let g = grid(vec![
            Cell { ch: '▀', fg: Some([255, 255, 255]), bg: Some([0, 0, 255]) },
            Cell { ch: '▀', fg: Some([255, 255, 255]), bg: None },
        ]);
        assert_eq!(
            encode(&g, ColorMode::Ansi16),
            "\x1b[97;44m▀\x1b[0;97m▀\x1b[0m\n"
        );
    }

    #[test]
    fn mono_has_no_escapes() {
        let g = grid(vec![Cell { ch: '#', fg: Some([1, 2, 3]), bg: Some([4, 5, 6]) }]);
        assert_eq!(encode(&g, ColorMode::Mono), "#\n");
    }
}
