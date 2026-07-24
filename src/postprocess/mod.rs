//! Post-processing stage: bg removal, outline, morphology, alpha binarize.
//! See `Config.post_*`. All ops off by default -> zero behavior change.

mod alpha;

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
/// Each op is gated by its own config flag; ops are infallible (pure pixel ops
/// on a validated, non-empty image).
pub fn postprocess(img: RgbaImage, config: &Config) -> RgbaImage {
    let mut img = img;
    if !matches!(config.post_alpha_threshold, AlphaThreshold::None) {
        img = alpha::binarize_alpha(img, config);
    }
    img
}
