use image::{imageops, imageops::FilterType, DynamicImage};

use crate::{color::Rgb, options::Adjust};

/// RGBA buffer in 0..1, sized to the render grid's sub-cell resolution.
#[derive(Clone, Debug)]
pub struct Pixels {
    pub width: usize,
    pub height: usize,
    pub data: Vec<[f32; 4]>,
}

impl Pixels {
    /// Resamples with premultiplied alpha so transparent edges don't bleed dark fringes.
    pub fn from_image(img: &DynamicImage, width: u32, height: u32) -> Self {
        let mut src = img.to_rgba32f();
        for p in src.pixels_mut() {
            let a = p.0[3];
            p.0[0] *= a;
            p.0[1] *= a;
            p.0[2] *= a;
        }
        let resized = imageops::resize(&src, width.max(1), height.max(1), FilterType::Lanczos3);
        let data = resized
            .pixels()
            .map(|p| {
                let [r, g, b, a] = p.0;
                let a = a.clamp(0.0, 1.0);
                if a <= 1e-6 {
                    [0.0; 4]
                } else {
                    [(r / a).clamp(0.0, 1.0), (g / a).clamp(0.0, 1.0), (b / a).clamp(0.0, 1.0), a]
                }
            })
            .collect();
        Self { width: resized.width() as usize, height: resized.height() as usize, data }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> [f32; 4] {
        self.data[y * self.width + x]
    }

    pub fn adjust(&mut self, adj: &Adjust) {
        if *adj == Adjust::default() {
            return;
        }
        let inv_gamma = 1.0 / adj.gamma.max(0.01);
        for p in &mut self.data {
            let mut c = [p[0], p[1], p[2]];
            if adj.invert {
                c = c.map(|v| 1.0 - v);
            }
            c = c.map(|v| (v - 0.5) * adj.contrast + 0.5 + adj.brightness);
            let l = luma3(c);
            c = c.map(|v| (l + (v - l) * adj.saturation).clamp(0.0, 1.0).powf(inv_gamma));
            p[..3].copy_from_slice(&c);
        }
    }
}

#[inline]
pub fn luma3(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

#[inline]
pub fn luma(p: &[f32; 4]) -> f32 {
    luma3([p[0], p[1], p[2]])
}

#[inline]
pub fn to_rgb(p: &[f32; 4]) -> Rgb {
    [p[0], p[1], p[2]].map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
}
