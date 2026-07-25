//! Alpha binarization: fixed strict threshold or Otsu adaptive.

use crate::postprocess::AlphaThreshold;
use crate::Config;
use image::RgbaImage;

pub fn binarize_alpha(img: RgbaImage, config: &Config) -> RgbaImage {
    let threshold: u8 = match config.post_alpha_threshold {
        AlphaThreshold::None => return img,
        AlphaThreshold::Fixed(t) => t,
        AlphaThreshold::Auto => otsu_threshold(&img).unwrap_or(128),
    };
    let mut out = img;
    for p in out.pixels_mut() {
        p[3] = if p[3] > threshold { 255 } else { 0 };
    }
    out
}

/// Classic Otsu on the alpha-channel histogram. Returns None when the image
/// is empty or the best threshold is degenerate (0 or 255 -> single peak).
fn otsu_threshold(img: &RgbaImage) -> Option<u8> {
    let mut hist = [0u32; 256];
    for p in img.pixels() {
        hist[p[3] as usize] += 1;
    }
    let total = (img.width() as usize * img.height() as usize) as f64;
    if total == 0.0 {
        return None;
    }
    let mut sum: u64 = 0;
    for (i, count) in hist.iter().enumerate() {
        sum += (i as u64) * (*count as u64);
    }
    let mut sum_b: u64 = 0;
    let mut w_b: u32 = 0;
    let mut max_var: f64 = 0.0;
    let mut threshold: u8 = 0;
    for t in 0..256u32 {
        w_b += hist[t as usize];
        if w_b == 0 {
            continue;
        }
        let w_f = (total as u32) - w_b;
        if w_f == 0 {
            break;
        }
        sum_b += (t as u64) * (hist[t as usize] as u64);
        let m_b = sum_b as f64 / w_b as f64;
        let m_f = (sum as f64 - sum_b as f64) / w_f as f64;
        let var = w_b as f64 * w_f as f64 * (m_b - m_f).powi(2);
        if var > max_var {
            max_var = var;
            threshold = t as u8;
        }
    }
    if threshold == 0 || threshold == 255 {
        None
    } else {
        Some(threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    fn img4(a: [u8; 4]) -> RgbaImage {
        ImageBuffer::from_pixel(2, 2, Rgba(a))
    }

    fn config_with_alpha(t: AlphaThreshold) -> Config {
        let mut c = Config::default();
        c.post_alpha_threshold = t;
        c
    }

    #[test]
    fn fixed_threshold_strict_greater() {
        // alpha 128 with threshold 128 -> strict > -> maps to 0
        let c = config_with_alpha(AlphaThreshold::Fixed(128));
        let out = binarize_alpha(img4([10, 20, 30, 128]), &c);
        assert_eq!(out.get_pixel(0, 0)[3], 0);
        // alpha 129 -> 255
        let out = binarize_alpha(img4([10, 20, 30, 129]), &c);
        assert_eq!(out.get_pixel(0, 0)[3], 255);
    }

    #[test]
    fn none_is_noop() {
        let c = config_with_alpha(AlphaThreshold::None);
        let out = binarize_alpha(img4([10, 20, 30, 200]), &c);
        assert_eq!(out.get_pixel(0, 0)[3], 200);
    }

    #[test]
    fn rgb_preserved() {
        let c = config_with_alpha(AlphaThreshold::Fixed(128));
        let out = binarize_alpha(img4([10, 20, 30, 200]), &c);
        let p = out.get_pixel(0, 0);
        assert_eq!([p[0], p[1], p[2]], [10, 20, 30]);
    }

    #[test]
    fn otsu_bimodal_picks_intermediate() {
        // half opaque (255), half semi (64) -> bimodal, threshold separates the
        // two peaks: t in [64, 255) so 64 -> 0 and 255 -> 255 under strict >.
        let mut img: RgbaImage = ImageBuffer::new(2, 1);
        img.put_pixel(0, 0, Rgba([0, 0, 0, 255]));
        img.put_pixel(1, 0, Rgba([0, 0, 0, 64]));
        let t = otsu_threshold(&img).expect("bimodal should yield a threshold");
        assert!(t >= 64 && t < 255);
    }

    #[test]
    fn otsu_single_peak_returns_none() {
        // all opaque -> single peak -> degenerate -> None -> fallback 128
        let img = img4([0, 0, 0, 255]);
        assert_eq!(otsu_threshold(&img), None);
    }

    #[test]
    fn auto_falls_back_to_128_on_single_peak() {
        let c = config_with_alpha(AlphaThreshold::Auto);
        // all opaque 255 > 128 -> 255 (fallback path exercised, no panic)
        let out = binarize_alpha(img4([0, 0, 0, 255]), &c);
        assert_eq!(out.get_pixel(0, 0)[3], 255);
    }
}
