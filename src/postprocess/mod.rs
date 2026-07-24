//! Post-processing stage: bg removal, outline, morphology, alpha binarize.
//! See `Config.post_*`. All ops off by default -> zero behavior change.
//!
//! NOTE: sub-module declarations (`mod alpha;`, `mod morphology;`, etc.) are
//! added incrementally in Tasks 3-7 as each op file is created. Declaring them
//! here would fail to compile (files don't exist yet).

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
pub fn postprocess(img: RgbaImage, _config: &Config) -> RgbaImage {
    // Stub: Task 2 wires this into the pipeline; Tasks 3-8 fill in the ops.
    img
}
