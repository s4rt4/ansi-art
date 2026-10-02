use std::fmt::Write;

use crate::{
    cell::Grid,
    color::{quantize, ColorMode, Rgb},
};

const BACKGROUND: &str = "#0c0c0c";
const FOREGROUND: &str = "#cccccc";

/// Standalone page; colors are pre-quantized so it shows what an xterm-palette terminal would.
pub fn encode(grid: &Grid, mode: ColorMode) -> String {
    let hex = |c: Rgb| {
        let [r, g, b] = quantize(mode, c);
        format!("#{r:02x}{g:02x}{b:02x}")
    };
    let mono = mode == ColorMode::Mono;

    let mut body = String::new();
    for y in 0..grid.rows {
        let mut open: Option<String> = None;
        for cell in grid.trimmed_row(y) {
            let mut style = String::new();
            if !mono {
                if let (Some(fg), false) = (cell.fg, cell.ch == ' ') {
                    let _ = write!(style, "color:{}", hex(fg));
                }
                if let Some(bg) = cell.bg {
                    let sep = if style.is_empty() { "" } else { ";" };
                    let _ = write!(style, "{sep}background:{}", hex(bg));
                }
            }
            let style = (!style.is_empty()).then_some(style);
            if style != open {
                if open.is_some() {
                    body.push_str("</span>");
                }
                if let Some(s) = &style {
                    let _ = write!(body, "<span style=\"{s}\">");
                }
                open = style;
            }
            match cell.ch {
                '&' => body.push_str("&amp;"),
                '<' => body.push_str("&lt;"),
                '>' => body.push_str("&gt;"),
                ch => body.push(ch),
            }
        }
        if open.is_some() {
            body.push_str("</span>");
        }
        body.push('\n');
    }

    format!(
        r#"<!doctype html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>ANSI art</title>
<style>
  body {{ margin: 0; padding: 24px; background: {BACKGROUND}; }}
  pre {{
    margin: 0;
    color: {FOREGROUND};
    font: 14px/1 "Cascadia Mono", "JetBrains Mono", "DejaVu Sans Mono", Menlo, Consolas, monospace;
    letter-spacing: 0;
  }}
</style>
</head>
<body>
<pre>{body}</pre>
</body>
</html>
"#
    )
}
