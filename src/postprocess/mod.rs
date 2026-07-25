//! Post-processing stage: bg removal, outline, morphology, alpha binarize.
//! See `Config.post_*`. All ops off by default -> zero behavior change.

mod alpha;
mod floodfill;
mod morphology;
mod outline;

use crate::Config;
use image::RgbaImage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgConnectivity {
    Conn4,
    Conn8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgScope {
    Outer,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutlineStyle {
    None,
    Rounded,
    Sharp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaThreshold {
    None,
    Fixed(u8),
    Auto,
}

/// Run enabled postprocess ops in fixed order:
/// flood-fill -> floating cleanup -> morphology -> alpha binarize -> outline.
/// Each op is gated by its own config flag; ops are pure pixel ops on a
/// validated, non-empty image and never fail.
pub fn run(img: RgbaImage, config: &Config) -> RgbaImage {
    let mut img = img;

    if config.post_bg_remove {
        img = floodfill::flood_fill_transparent(img, config);
    }

    if config.post_bg_floating_max_pixels > 0 {
        floodfill::remove_small_floating_components(&mut img, config.post_bg_floating_max_pixels);
    }

    if config.post_morph {
        img = morphology::morph_open_close(img, config);
    }

    if config.post_alpha_threshold != AlphaThreshold::None {
        img = alpha::binarize_alpha(img, config);
    }

    if config.post_outline != OutlineStyle::None {
        img = outline::apply_outline(img, config);
    }

    img
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod dispatch_tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    #[test]
    fn outline_runs_last_and_grows_canvas() {
        // alpha binarize + outline both on: outline must run after binarize
        // (so it sees clean alpha) and grow the canvas by +2.
        let mut img: RgbaImage = ImageBuffer::new(2, 2);
        for y in 0..2 {
            for x in 0..2 {
                img.put_pixel(x, y, Rgba([100, 100, 100, 200]));
            }
        }
        let mut c = Config::default();
        c.post_alpha_threshold = AlphaThreshold::Fixed(128);
        c.post_outline = OutlineStyle::Sharp;
        let out = run(img, &c);
        assert_eq!(out.dimensions(), (4, 4), "outline grew canvas +2");
        // alpha was binarized to 255 (>128), then outline drawn into border ring
        assert_eq!(out.get_pixel(1, 1)[3], 255);
    }

    #[test]
    fn all_off_is_identity() {
        let img: RgbaImage = ImageBuffer::from_pixel(2, 2, Rgba([1, 2, 3, 200]));
        let before = img.clone();
        let out = run(img, &Config::default());
        assert_eq!(out.dimensions(), before.dimensions());
        assert_eq!(out.get_pixel(0, 0), before.get_pixel(0, 0));
    }
}
