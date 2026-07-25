//! Palette parsing, nearest-color mapping, and palette application.

use crate::error::{PixelSnapperError, Result};
use crate::quantize::{self, Colorspace, DitherMethod, PresetPalette};
use crate::Config;
use image::{Rgba, RgbaImage};
use std::collections::HashMap;

pub const MAX_PALETTE_COLORS: usize = 256;

pub fn parse_palette_hex(value: &str) -> Result<Vec<[u8; 3]>> {
    if value.trim().is_empty() {
        return Err(PixelSnapperError::InvalidInput(
            "Palette must contain at least one color".to_string(),
        ));
    }

    let mut seen = std::collections::HashSet::new();
    let mut palette = Vec::new();
    for part in value.split(',') {
        let hex = part.trim().trim_start_matches('#');
        if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(PixelSnapperError::InvalidInput(format!(
                "invalid palette color '{}', expected a 6-digit hex code",
                part.trim()
            )));
        }
        let color = [
            u8::from_str_radix(&hex[0..2], 16).unwrap(),
            u8::from_str_radix(&hex[2..4], 16).unwrap(),
            u8::from_str_radix(&hex[4..6], 16).unwrap(),
        ];
        if seen.insert(color) {
            palette.push(color);
        }
    }

    if palette.len() > MAX_PALETTE_COLORS {
        return Err(PixelSnapperError::InvalidInput(format!(
            "Palette must contain at most {} distinct colors",
            MAX_PALETTE_COLORS
        )));
    }
    Ok(palette)
}

pub fn nearest_palette_color(rgb: [u8; 3], palette: &[[u8; 3]]) -> [u8; 3] {
    let target = crate::quantize::oklab::rgb_to_oklab(rgb[0], rgb[1], rgb[2]);
    let mut best_color = palette[0];
    let mut best_distance = f32::MAX;
    for color in palette {
        let c = crate::quantize::oklab::rgb_to_oklab(color[0], color[1], color[2]);
        let dl = target[0] - c[0];
        let da = target[1] - c[1];
        let db = target[2] - c[2];
        let distance = dl * dl + da * da + db * db;
        if distance < best_distance {
            best_distance = distance;
            best_color = *color;
        }
    }
    best_color
}

pub fn apply_palette(img: &RgbaImage, palette: &[[u8; 3]]) -> Result<RgbaImage> {
    if palette.is_empty() {
        return Err(PixelSnapperError::InvalidInput(
            "Palette must contain at least one RGB color".to_string(),
        ));
    }

    let mut cache: HashMap<[u8; 3], [u8; 3]> = HashMap::new();
    let mut recolored_img = RgbaImage::new(img.width(), img.height());

    for (x, y, pixel) in img.enumerate_pixels() {
        if pixel[3] == 0 {
            recolored_img.put_pixel(x, y, *pixel);
            continue;
        }

        let key = [pixel[0], pixel[1], pixel[2]];
        let color = *cache
            .entry(key)
            .or_insert_with(|| nearest_palette_color(key, palette));
        recolored_img.put_pixel(x, y, Rgba([color[0], color[1], color[2], pixel[3]]));
    }

    Ok(recolored_img)
}

/// Extract a palette from an image via Oklab k-means (dither/preset off so the
/// output is a clean k-color reduction). Returns unique opaque RGB colors,
/// sorted for deterministic output. Used by `--palette-from` / WASM
/// `extract_palette` to lock a target frame to a reference frame's palette.
pub(crate) fn extract_palette_from_image(
    img: &RgbaImage,
    k_colors: usize,
    colorspace: Colorspace,
) -> Result<Vec<[u8; 3]>> {
    let mut cfg = Config::default();
    cfg.k_colors = k_colors;
    cfg.quantize_colorspace = colorspace;
    cfg.quantize_dither = DitherMethod::None;
    cfg.quantize_preset_palette = PresetPalette::None;
    let quantized = quantize::quantize(img, &cfg)?;
    let mut seen = std::collections::HashSet::new();
    let mut palette: Vec<[u8; 3]> = Vec::new();
    for p in quantized.pixels() {
        if p[3] == 0 {
            continue;
        }
        let c = [p[0], p[1], p[2]];
        if seen.insert(c) {
            palette.push(c);
        }
    }
    palette.sort();
    Ok(palette)
}

/// Bytes-loading wrapper around `extract_palette_from_image`. Used by the CLI
/// `--palette-from` resolver and the WASM `extract_palette` export.
pub(crate) fn extract_palette_from_image_via_bytes(
    bytes: &[u8],
    k_colors: usize,
    colorspace: Colorspace,
) -> Result<Vec<[u8; 3]>> {
    let img = image::load_from_memory(bytes)?.to_rgba8();
    extract_palette_from_image(&img, k_colors, colorspace)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod palette_tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    fn solid(color: [u8; 4]) -> RgbaImage {
        ImageBuffer::from_pixel(4, 4, Rgba(color))
    }

    #[test]
    fn extract_single_color_palette() {
        let img = solid([220, 40, 40, 255]);
        let pal = extract_palette_from_image(&img, 1, Colorspace::Oklab).unwrap();
        assert_eq!(pal.len(), 1);
        // k=1 k-means centroid of a single-color image ≈ that color
        assert_eq!(pal[0], [220, 40, 40]);
    }

    #[test]
    fn extract_is_deterministic() {
        let img = solid([10, 20, 30, 255]);
        let a = extract_palette_from_image(&img, 4, Colorspace::Oklab).unwrap();
        let b = extract_palette_from_image(&img, 4, Colorspace::Oklab).unwrap();
        assert_eq!(a, b, "same input -> byte-identical sorted palette (R1)");
    }

    #[test]
    fn extract_skips_transparent() {
        let mut img: RgbaImage = ImageBuffer::new(2, 1);
        img.put_pixel(0, 0, Rgba([5, 5, 5, 0])); // transparent
        img.put_pixel(1, 0, Rgba([200, 100, 50, 255]));
        let pal = extract_palette_from_image(&img, 4, Colorspace::Oklab).unwrap();
        assert!(
            pal.iter().all(|c| c != &[5, 5, 5]),
            "transparent pixel color excluded"
        );
        assert!(pal.contains(&[200, 100, 50]));
    }

    #[test]
    fn cross_frame_lock() {
        // frame1 red (220,40,40); frame2 "drifted" red (215,45,38).
        // Extract palette from frame1 (k=1), snap frame2 to it -> frame2 uses
        // frame1's red. This is the drift-elimination guarantee.
        let frame1 = solid([220, 40, 40, 255]);
        let frame2 = solid([215, 45, 38, 255]);
        let pal = extract_palette_from_image(&frame1, 1, Colorspace::Oklab).unwrap();
        assert_eq!(pal.len(), 1);
        let snapped = apply_palette(&frame2, &pal).unwrap();
        assert_eq!(
            &snapped.get_pixel(0, 0).0[0..3],
            &pal[0],
            "frame2 snapped to frame1's palette entry -> cross-frame consistency"
        );
    }

    #[test]
    fn nearest_uses_oklab_distance() {
        // Regression guard: nearest_palette_color must match an independent
        // Oklab-nearest computation. Breaks if someone reverts to RGB distance
        // for a target where Oklab and RGB disagree.
        let target = [180, 60, 60];
        let palette = [[255, 0, 0], [180, 180, 60], [60, 60, 60]];
        let got = nearest_palette_color(target, &palette);
        let t = crate::quantize::oklab::rgb_to_oklab(target[0], target[1], target[2]);
        let mut expected = palette[0];
        let mut best = f32::MAX;
        for c in &palette {
            let o = crate::quantize::oklab::rgb_to_oklab(c[0], c[1], c[2]);
            let d = (t[0] - o[0]).powi(2) + (t[1] - o[1]).powi(2) + (t[2] - o[2]).powi(2);
            if d < best {
                best = d;
                expected = *c;
            }
        }
        assert_eq!(got, expected);
    }
}
