use serde::{Deserialize, Serialize};

use crate::{color::ColorMode, dither::Dither};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    /// Character ramp by brightness (` .:-=+*#%@`)
    Ascii,
    /// Shade blocks `░▒▓█`
    Shade,
    /// One full block per pixel
    Block,
    /// `▀` with fg/bg colors: 1×2 pixels per cell
    HalfBlock,
    /// Quadrant blocks `▘▚▟`: 2×2 pixels, two colors per cell
    Quadrant,
    /// Sextants (Unicode 13): 2×3 pixels, two colors per cell
    Sextant,
    /// Braille dots `⣿`: 2×4 pixels, one color per cell
    Braille,
    /// Shape-matched ASCII with a few flat colors, like neofetch distro logos
    Glyph,
}

impl Mode {
    pub const ALL: [Mode; 8] = [
        Mode::Ascii,
        Mode::Shade,
        Mode::Block,
        Mode::HalfBlock,
        Mode::Quadrant,
        Mode::Sextant,
        Mode::Braille,
        Mode::Glyph,
    ];

    /// Source pixels sampled per character cell (width, height).
    pub fn subcell(self) -> (usize, usize) {
        match self {
            Mode::Ascii | Mode::Shade | Mode::Block => (1, 1),
            Mode::HalfBlock => (1, 2),
            Mode::Quadrant => (2, 2),
            Mode::Sextant => (2, 3),
            Mode::Braille => (2, 4),
            Mode::Glyph => (crate::render::glyph::GW, crate::render::glyph::GH),
        }
    }
}

/// Settings for [`Mode::Glyph`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GlyphOptions {
    /// Preset name (see [`crate::render::glyph::CHARSETS`]) or literal characters to match against.
    pub charset: String,
    /// Character for fully covered cells; empty = let the matcher pick dense glyphs itself.
    pub fill: String,
    /// Coverage (0..1) at which a cell counts as full and gets `fill`.
    pub fill_level: f32,
    /// Number of flat colors the image is reduced to (1..=8).
    pub colors: u8,
}

impl Default for GlyphOptions {
    fn default() -> Self {
        Self { charset: "edges".into(), fill: "M".into(), fill_level: 0.8, colors: 3 }
    }
}

/// Tone adjustments applied before dithering.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Adjust {
    /// -1..1, added after contrast
    pub brightness: f32,
    /// 1.0 = unchanged
    pub contrast: f32,
    /// 1.0 = unchanged; >1 brightens midtones
    pub gamma: f32,
    /// 0 = grayscale, 1 = unchanged
    pub saturation: f32,
    pub invert: bool,
}

impl Default for Adjust {
    fn default() -> Self {
        Self { brightness: 0.0, contrast: 1.0, gamma: 1.0, saturation: 1.0, invert: false }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RenderOptions {
    pub mode: Mode,
    /// Output width in columns.
    pub width: u32,
    /// Output height in rows; derived from the aspect ratio when `None`.
    pub height: Option<u32>,
    /// Character cell width / height. Most terminal fonts are close to 0.5.
    pub cell_aspect: f32,
    pub color: ColorMode,
    pub dither: Dither,
    /// Preset name (see [`crate::render::ramps`]) or a literal dark→bright character list.
    pub ramp: String,
    /// Luminance cut-off (0..1) for monochrome output and braille dots.
    pub threshold: f32,
    /// Pixels with alpha below this are treated as transparent.
    pub alpha_threshold: f32,
    pub adjust: Adjust,
    pub glyph: GlyphOptions,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            mode: Mode::HalfBlock,
            width: 60,
            height: None,
            cell_aspect: 0.5,
            color: ColorMode::Truecolor,
            dither: Dither::None,
            ramp: "standard".into(),
            threshold: 0.5,
            alpha_threshold: 0.5,
            adjust: Adjust::default(),
            glyph: GlyphOptions::default(),
        }
    }
}
