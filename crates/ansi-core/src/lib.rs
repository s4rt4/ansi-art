//! ansi-core: turns raster images into terminal art.
//!
//! Pipeline: load → resample to sub-cell pixels → adjust → dither → render cells → export.

pub mod cell;
pub mod color;
pub mod dither;
pub mod export;
pub mod options;
pub mod pixels;
pub mod render;

use std::path::Path;

pub use cell::{Cell, Grid};
pub use color::{ColorMode, Rgb};
pub use dither::Dither;
pub use export::Format;
pub use image::DynamicImage;
pub use options::{Adjust, GlyphOptions, Mode, RenderOptions};

use image::{imageops::FilterType, GenericImageView};
use pixels::Pixels;

/// Larger sources are downscaled on load; no render mode needs more detail than this.
pub const MAX_SOURCE_SIDE: u32 = 2048;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("image: {0}")]
    Image(#[from] image::ImageError),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Decodes any format the `image` crate supports (PNG, JPEG, WebP, GIF, BMP, TIFF, ...).
pub fn load(path: impl AsRef<Path>) -> Result<DynamicImage> {
    let img = image::ImageReader::open(path)?.with_guessed_format()?.decode()?;
    let (w, h) = img.dimensions();
    Ok(if w.max(h) > MAX_SOURCE_SIDE {
        img.resize(MAX_SOURCE_SIDE, MAX_SOURCE_SIDE, FilterType::Triangle)
    } else {
        img
    })
}

/// Output size in character cells, preserving the image aspect ratio.
pub fn grid_size(img_w: u32, img_h: u32, opts: &RenderOptions) -> (usize, usize) {
    let cols = opts.width.max(1) as usize;
    let rows = match opts.height {
        Some(h) => h.max(1) as usize,
        None => {
            let ratio = img_h as f32 / img_w.max(1) as f32;
            (cols as f32 * ratio * opts.cell_aspect).round().max(1.0) as usize
        }
    };
    (cols, rows)
}

pub fn convert(img: &DynamicImage, opts: &RenderOptions) -> Grid {
    let (w, h) = img.dimensions();
    let (cols, rows) = grid_size(w, h, opts);
    let (sx, sy) = opts.mode.subcell();
    let mut px = Pixels::from_image(img, (cols * sx) as u32, (rows * sy) as u32);
    px.adjust(&opts.adjust);
    // Glyph mode quantizes to its own flat palette; dither noise would wreck the shapes.
    if opts.mode != Mode::Glyph {
        dither::apply(&mut px, opts);
    }
    render::render(&px, cols, rows, opts)
}
