//! Morphological open→close on the alpha channel only, preserving palette.
//!
//! Uses a 2×2 kernel with the source pixel as the top-left anchor. Border pixels
//! are handled by replication (clamp to the nearest edge).

use crate::Config;
use image::RgbaImage;

/// Apply 2×2 open→close to the alpha channel only. RGB is left untouched.
pub fn morph_open_close(img: RgbaImage, config: &Config) -> RgbaImage {
    if !config.post_morph {
        return img;
    }
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return img;
    }

    let opened = apply_morph(&img, w, h, MorphOp::Erode, MorphOp::Dilate);
    let closed = apply_morph(&opened, w, h, MorphOp::Dilate, MorphOp::Erode);
    closed
}

#[derive(Clone, Copy)]
enum MorphOp {
    Erode,
    Dilate,
}

fn apply_morph(
    img: &RgbaImage,
    w: u32,
    h: u32,
    first: MorphOp,
    second: MorphOp,
) -> RgbaImage {
    let tmp = morph_pass(img, w, h, first);
    morph_pass(&tmp, w, h, second)
}

fn morph_pass(img: &RgbaImage, w: u32, h: u32, op: MorphOp) -> RgbaImage {
    let mut out = img.clone();
    for y in 0..h {
        for x in 0..w {
            let mut acc = match op {
                MorphOp::Erode => 255u8,
                MorphOp::Dilate => 0u8,
            };
            for dy in 0..2 {
                for dx in 0..2 {
                    let xx = (x + dx).min(w - 1);
                    let yy = (y + dy).min(h - 1);
                    let a = img.get_pixel(xx, yy).0[3];
                    acc = match op {
                        MorphOp::Erode => acc.min(a),
                        MorphOp::Dilate => acc.max(a),
                    };
                }
            }
            out.get_pixel_mut(x, y).0[3] = acc;
        }
    }
    out
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::Config;
    use image::RgbaImage;

    #[test]
    fn alpha_only_no_rgb_synthesis() {
        let mut c = Config::default();
        c.post_morph = true;
        let mut img = RgbaImage::new(3, 3);
        // Center opaque red, surrounded by transparent black.
        img.put_pixel(1, 1, image::Rgba([255, 0, 0, 255]));
        let out = morph_open_close(img, &c);
        // RGB of transparent neighbors is untouched (still 0,0,0,0).
        assert_eq!(out.get_pixel(0, 0).0, [0, 0, 0, 0]);
        // RGB of the center is preserved (alpha may change: open removes the
        // isolated 1px speckle, but RGB is never synthesized or rewritten).
        let center = out.get_pixel(1, 1).0;
        assert_eq!([center[0], center[1], center[2]], [255, 0, 0]);
    }

    #[test]
    fn removes_single_pixel_speckle() {
        let mut c = Config::default();
        c.post_morph = true;
        let mut img = RgbaImage::new(3, 3);
        img.put_pixel(1, 1, image::Rgba([255, 255, 255, 255]));
        let out = morph_open_close(img, &c);
        assert_eq!(out.get_pixel(1, 1).0[3], 0);
    }

    #[test]
    fn fills_two_by_two_hole() {
        let mut c = Config::default();
        c.post_morph = true;
        let mut img = RgbaImage::new(4, 4);
        // Fill a 2×2 hole at (1,1)-(2,2) inside an opaque frame.
        for y in 0..4 {
            for x in 0..4 {
                let inside_hole = (1..=2).contains(&x) && (1..=2).contains(&y);
                if !inside_hole {
                    img.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
                }
            }
        }
        let out = morph_open_close(img, &c);
        assert_eq!(out.get_pixel(1, 1).0[3], 255);
        assert_eq!(out.get_pixel(2, 2).0[3], 255);
    }
}
