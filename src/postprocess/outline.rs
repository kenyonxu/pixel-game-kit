//! Outline: pad canvas by +1px on every side and draw a 1px border around
//! opaque regions.

use crate::postprocess::OutlineStyle;
use crate::Config;
use image::{Rgba, RgbaImage};

/// Apply an outline to opaque regions. The output canvas is larger by +2 on each
/// axis. Transparent pixels adjacent to opaque regions are filled with the
/// configured outline color at alpha = 255. Existing opaque pixels are never
/// overwritten.
pub fn apply_outline(img: RgbaImage, config: &Config) -> RgbaImage {
    let style = config.post_outline;
    if style == OutlineStyle::None {
        return img;
    }

    let (w, h) = img.dimensions();
    let mut out = RgbaImage::new(w + 2, h + 2);

    // Copy original to offset (1,1).
    for y in 0..h {
        for x in 0..w {
            out.put_pixel(x + 1, y + 1, *img.get_pixel(x, y));
        }
    }

    let color = Rgba([
        config.post_outline_color[0],
        config.post_outline_color[1],
        config.post_outline_color[2],
        255,
    ]);

    let neighbor_offsets: &[(isize, isize)] = match style {
        OutlineStyle::Sharp => &[(0, -1), (0, 1), (-1, 0), (1, 0)],
        OutlineStyle::Rounded => &[
            (0, -1),
            (0, 1),
            (-1, 0),
            (1, 0),
            (-1, -1),
            (-1, 1),
            (1, -1),
            (1, 1),
        ],
        OutlineStyle::None => unreachable!(),
    };

    for y in 0..out.height() {
        for x in 0..out.width() {
            if out.get_pixel(x, y).0[3] != 0 {
                // Already opaque: never overwrite.
                continue;
            }
            // Check neighbors in the *source* image coordinates.
            // A destination pixel (x,y) maps to source (x-1,y-1); neighbors are
            // source (sx+dx, sy+dy) = (x-1+dx, y-1+dy).
            for &(dx, dy) in neighbor_offsets {
                let sx = x as isize - 1 + dx;
                let sy = y as isize - 1 + dy;
                if sx < 0 || sy < 0 || sx >= w as isize || sy >= h as isize {
                    continue;
                }
                if img.get_pixel(sx as u32, sy as u32).0[3] > 0 {
                    out.put_pixel(x, y, color);
                    break;
                }
            }
        }
    }

    out
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::postprocess::OutlineStyle;
    use crate::Config;
    use image::RgbaImage;

    #[test]
    fn sharp_outline_grows_canvas() {
        let mut c = Config::default();
        c.post_outline = OutlineStyle::Sharp;
        let mut img = RgbaImage::new(3, 3);
        img.put_pixel(1, 1, image::Rgba([255, 0, 0, 255]));
        let out = apply_outline(img, &c);
        assert_eq!(out.dimensions(), (5, 5));
        assert_eq!(out.get_pixel(2, 2).0, [255, 0, 0, 255]);
        // 4-way neighbors of center.
        assert_eq!(out.get_pixel(2, 1).0, [0, 0, 0, 255]);
        assert_eq!(out.get_pixel(2, 3).0, [0, 0, 0, 255]);
        assert_eq!(out.get_pixel(1, 2).0, [0, 0, 0, 255]);
        assert_eq!(out.get_pixel(3, 2).0, [0, 0, 0, 255]);
        // Diagonal stays transparent under sharp.
        assert_eq!(out.get_pixel(1, 1).0[3], 0);
    }

    #[test]
    fn rounded_outline_fills_diagonals() {
        let mut c = Config::default();
        c.post_outline = OutlineStyle::Rounded;
        let mut img = RgbaImage::new(3, 3);
        img.put_pixel(1, 1, image::Rgba([255, 0, 0, 255]));
        let out = apply_outline(img, &c);
        assert_eq!(out.get_pixel(1, 1).0, [0, 0, 0, 255]);
    }

    #[test]
    fn custom_outline_color() {
        let mut c = Config::default();
        c.post_outline = OutlineStyle::Sharp;
        c.post_outline_color = [255, 128, 64];
        let mut img = RgbaImage::new(1, 1);
        img.put_pixel(0, 0, image::Rgba([0, 0, 0, 255]));
        let out = apply_outline(img, &c);
        // 1x1 becomes 3x3; center opaque, all surrounding pixels colored.
        assert_eq!(out.get_pixel(0, 1).0, [255, 128, 64, 255]);
    }
}
