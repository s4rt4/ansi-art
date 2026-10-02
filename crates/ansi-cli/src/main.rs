use std::{io::Write, path::PathBuf, process::ExitCode};

use ansi_core::{export, render::ramps, Adjust, ColorMode, Dither, Format, GlyphOptions, Mode, RenderOptions};
use clap::Parser;

/// Turn images into ANSI art for the terminal.
#[derive(Parser)]
#[command(name = "ansi-art", version)]
struct Args {
    /// Input image (PNG, JPEG, WebP, GIF, BMP, TIFF, ...)
    input: PathBuf,

    #[arg(short, long, value_enum, default_value_t = Mode::HalfBlock)]
    mode: Mode,

    /// Width in columns [default: terminal width, max 80]
    #[arg(short, long)]
    width: Option<u32>,

    /// Height in rows [default: keep aspect ratio]
    #[arg(long)]
    height: Option<u32>,

    /// Color depth [default: detected from the terminal]
    #[arg(short, long, value_enum)]
    color: Option<ColorMode>,

    #[arg(short, long, value_enum, default_value_t = Dither::None)]
    dither: Dither,

    /// Ramp preset or literal characters, dark to bright (ascii mode)
    #[arg(long, default_value = "standard", long_help = ramp_help())]
    ramp: String,

    /// Luminance cut-off 0..1 for mono output and braille dots
    #[arg(long, default_value_t = 0.5)]
    threshold: f32,

    /// Character cell width / height of your terminal font
    #[arg(long, default_value_t = 0.5)]
    cell_aspect: f32,

    #[arg(long, default_value_t = 0.0, allow_hyphen_values = true)]
    brightness: f32,
    #[arg(long, default_value_t = 1.0)]
    contrast: f32,
    #[arg(long, default_value_t = 1.0)]
    gamma: f32,
    #[arg(long, default_value_t = 1.0)]
    saturation: f32,
    #[arg(long)]
    invert: bool,

    /// Glyph mode: charset preset (edges, letters, all, blocks) or literal characters
    #[arg(long, default_value = "edges")]
    charset: String,

    /// Glyph mode: character for solid areas ("" = let the matcher choose)
    #[arg(long, default_value = "M")]
    fill: String,

    /// Glyph mode: coverage 0..1 at which a cell counts as solid
    #[arg(long, default_value_t = 0.8)]
    fill_level: f32,

    /// Glyph mode: number of flat colors (1-8)
    #[arg(long, default_value_t = 3, value_parser = clap::value_parser!(u8).range(1..=8))]
    glyph_colors: u8,

    /// Output format [default: from --output extension, else ansi]
    #[arg(short, long, value_enum)]
    format: Option<Format>,

    /// Write to a file instead of stdout
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn ramp_help() -> String {
    let presets: Vec<_> = ramps::PRESETS.iter().map(|(name, chars)| format!("  {name:<9} {chars:?}")).collect();
    format!("Ramp preset or literal characters, dark to bright (ascii mode).\nPresets:\n{}", presets.join("\n"))
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ansi-art: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> ansi_core::Result<()> {
    let to_stdout = args.output.is_none();
    if to_stdout {
        // Turns on VT processing in legacy Windows consoles; no-op elsewhere.
        let _ = enable_ansi_support::enable_ansi_support();
    }

    let format = args
        .format
        .or_else(|| args.output.as_deref().and_then(Format::from_path))
        .unwrap_or(Format::Ansi);
    let width = args.width.unwrap_or_else(|| default_width(to_stdout));

    let opts = RenderOptions {
        mode: args.mode,
        width,
        height: args.height,
        cell_aspect: args.cell_aspect,
        color: args.color.unwrap_or_else(detect_color),
        dither: args.dither,
        ramp: args.ramp,
        threshold: args.threshold,
        adjust: Adjust {
            brightness: args.brightness,
            contrast: args.contrast,
            gamma: args.gamma,
            saturation: args.saturation,
            invert: args.invert,
        },
        glyph: GlyphOptions {
            charset: args.charset,
            fill: args.fill,
            fill_level: args.fill_level,
            colors: args.glyph_colors,
        },
        ..RenderOptions::default()
    };

    let img = ansi_core::load(&args.input)?;
    let grid = ansi_core::convert(&img, &opts);
    let out = export::export(&grid, format, opts.color)?;

    match &args.output {
        Some(path) => std::fs::write(path, &out.text)?,
        None => std::io::stdout().lock().write_all(out.text.as_bytes())?,
    }
    let file = args.output.as_ref().map_or("logo.txt".into(), |p| p.display().to_string());
    if let Some(hint) = out.hint_for(&file) {
        eprintln!("\nUse it with:\n  {hint}");
    }
    Ok(())
}

fn default_width(to_stdout: bool) -> u32 {
    let term = to_stdout.then(terminal_size::terminal_size).flatten();
    term.map_or(80, |(w, _)| (w.0 as u32).saturating_sub(1).clamp(8, 80))
}

/// Best-effort guess; `--color` always wins.
fn detect_color() -> ColorMode {
    let var = |k: &str| std::env::var(k).unwrap_or_default().to_ascii_lowercase();
    if std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()) {
        return ColorMode::Mono;
    }
    let colorterm = var("COLORTERM");
    if colorterm.contains("truecolor") || colorterm.contains("24bit") {
        return ColorMode::Truecolor;
    }
    // Windows 10+ consoles and Windows Terminal handle 24-bit color.
    if cfg!(windows) || std::env::var_os("WT_SESSION").is_some() {
        return ColorMode::Truecolor;
    }
    let term = var("TERM");
    match term.as_str() {
        "dumb" => ColorMode::Mono,
        t if t.contains("256color") || t.contains("kitty") || t.contains("alacritty") => ColorMode::Ansi256,
        "linux" => ColorMode::Ansi8,
        _ => ColorMode::Ansi16,
    }
}
