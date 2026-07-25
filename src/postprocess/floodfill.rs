//! Flood-fill background removal + floating-island cleanup.
//! Semantics from PixelRefiner floodfill.ts + processor.ts (clean-room).

use crate::postprocess::{BgConnectivity, BgScope};
use crate::Config;
use image::RgbaImage;

const NEIGHBORS_4: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
const NEIGHBORS_8: [(isize, isize); 8] = [
    (-1, 0),
    (1, 0),
    (0, -1),
    (0, 1),
    (-1, -1),
    (-1, 1),
    (1, -1),
    (1, 1),
];

/// Remove background via stack-based flood-fill. Returns a new image with
/// matched pixels set to transparent (alpha = 0). RGB values are left unchanged.
///
/// - `Outer`: flood from every opaque border pixel (each seed uses its own color).
/// - `All`: erase every opaque pixel within tolerance of any distinct opaque
///   border color (auto-derived target set). Aggressive: strips interior
///   bg-colored pockets too, but never touches colors absent from the border.
pub fn flood_fill_transparent(img: RgbaImage, config: &Config) -> RgbaImage {
    if !config.post_bg_remove {
        return img;
    }
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return img;
    }
    let w = w as usize;
    let h = h as usize;
    let len = w * h;

    let mut out = img.clone();
    let mut visited = vec![false; len];

    let tolerance = config.post_bg_tolerance;
    let connectivity = config.post_bg_connectivity;

    match config.post_bg_scope {
        BgScope::Outer => {
            // Seed from every non-transparent border pixel, sharing one visited map.
            let seeds = border_seeds(w, h, &img);
            for (sx, sy) in seeds {
                let seed_idx = sy * w + sx;
                if visited[seed_idx] {
                    continue;
                }
                let seed = img.get_pixel(sx as u32, sy as u32).0;
                flood_from_seed(
                    sx,
                    sy,
                    seed,
                    tolerance,
                    connectivity,
                    w,
                    h,
                    &img,
                    &mut visited,
                    &mut out,
                );
            }
        }
        BgScope::All => {
            // Auto-derived target set: distinct opaque colors touching the border.
            let targets = border_colors(&img, w, h);
            for y in 0..h {
                for x in 0..w {
                    let p = out.get_pixel(x as u32, y as u32).0;
                    if p[3] == 0 {
                        continue;
                    }
                    if targets.iter().any(|c| matches_color_rgb(p, *c, tolerance)) {
                        out.get_pixel_mut(x as u32, y as u32).0[3] = 0;
                    }
                }
            }
        }
    }

    out
}

/// Floating-island cleanup: 4-connected CCL on opaque pixels (alpha >= 16),
/// erase every component whose size is <= `max_pixels` except the largest one.
/// The largest component always survives, even if it is small (anti-foot-gun).
pub fn remove_small_floating_components(img: &mut RgbaImage, max_pixels: usize) {
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 || max_pixels == 0 {
        return;
    }
    let w = w as usize;
    let h = h as usize;
    let len = w * h;
    let px = |x: usize, y: usize| -> usize { y * w + x };

    let mut visited = vec![false; len];
    let mut component_sizes: Vec<usize> = Vec::new();
    let mut component_coords: Vec<Vec<(usize, usize)>> = Vec::new();

    let is_opaque = |idx: usize| {
        let (x, y) = (idx % w, idx / w);
        img.get_pixel(x as u32, y as u32).0[3] >= 16
    };

    for y in 0..h {
        for x in 0..w {
            let idx = px(x, y);
            if visited[idx] || !is_opaque(idx) {
                continue;
            }
            // Stack-based flood fill for this component (4-way only, per spec).
            let mut stack = vec![(x, y)];
            visited[idx] = true;
            let mut size = 0usize;
            let mut coords = Vec::new();

            while let Some((cx, cy)) = stack.pop() {
                size += 1;
                if size <= max_pixels {
                    // Bounds memory: stop storing once we exceed max_pixels,
                    // but keep counting the true component size.
                    coords.push((cx, cy));
                }
                for (dx, dy) in NEIGHBORS_4 {
                    let nx = cx as isize + dx;
                    let ny = cy as isize + dy;
                    if nx < 0 || ny < 0 || nx >= w as isize || ny >= h as isize {
                        continue;
                    }
                    let (nx, ny) = (nx as usize, ny as usize);
                    let nidx = px(nx, ny);
                    if !visited[nidx] && is_opaque(nidx) {
                        visited[nidx] = true;
                        stack.push((nx, ny));
                    }
                }
            }

            component_sizes.push(size);
            component_coords.push(coords);
        }
    }

    if component_sizes.is_empty() {
        return;
    }

    let largest_label = component_sizes
        .iter()
        .enumerate()
        .max_by_key(|&(_, size)| size)
        .map(|(label, _)| label)
        .unwrap_or(0);

    for (label, size) in component_sizes.iter().enumerate() {
        if label == largest_label {
            continue;
        }
        if *size <= max_pixels {
            for &(x, y) in &component_coords[label] {
                img.get_pixel_mut(x as u32, y as u32).0[3] = 0;
            }
        }
    }
}

fn border_seeds(w: usize, h: usize, img: &RgbaImage) -> Vec<(usize, usize)> {
    let mut seeds = Vec::new();
    for x in 0..w {
        if img.get_pixel(x as u32, 0).0[3] != 0 {
            seeds.push((x, 0));
        }
        if h > 1 && img.get_pixel(x as u32, (h - 1) as u32).0[3] != 0 {
            seeds.push((x, h - 1));
        }
    }
    for y in 1..h.saturating_sub(1) {
        if img.get_pixel(0, y as u32).0[3] != 0 {
            seeds.push((0, y));
        }
        if w > 1 && img.get_pixel((w - 1) as u32, y as u32).0[3] != 0 {
            seeds.push((w - 1, y));
        }
    }
    seeds
}

/// Distinct opaque colors touching the image border (the auto bg-target set).
fn border_colors(img: &RgbaImage, w: usize, h: usize) -> Vec<[u8; 3]> {
    let mut set = std::collections::HashSet::new();
    let mut push = |x: usize, y: usize| {
        let p = img.get_pixel(x as u32, y as u32).0;
        if p[3] != 0 {
            set.insert([p[0], p[1], p[2]]);
        }
    };
    for x in 0..w {
        push(x, 0);
        push(x, h - 1);
    }
    for y in 0..h {
        push(0, y);
        push(w - 1, y);
    }
    set.into_iter().collect()
}

#[allow(clippy::too_many_arguments)]
fn flood_from_seed(
    sx: usize,
    sy: usize,
    seed: [u8; 4],
    tolerance: u8,
    connectivity: BgConnectivity,
    w: usize,
    h: usize,
    img: &RgbaImage,
    visited: &mut [bool],
    out: &mut RgbaImage,
) {
    let px = |x: usize, y: usize| -> usize { y * w + x };
    let start_idx = px(sx, sy);
    if visited[start_idx] {
        return;
    }

    let neighbors: &[(isize, isize)] = match connectivity {
        BgConnectivity::Conn4 => &NEIGHBORS_4,
        BgConnectivity::Conn8 => &NEIGHBORS_8,
    };

    let mut stack = vec![(sx, sy)];
    visited[start_idx] = true;

    while let Some((cx, cy)) = stack.pop() {
        // Interior pixels reached through the fill must be non-transparent.
        if img.get_pixel(cx as u32, cy as u32).0[3] == 0 {
            continue;
        }
        out.get_pixel_mut(cx as u32, cy as u32).0[3] = 0;

        for &(dx, dy) in neighbors {
            let nx = cx as isize + dx;
            let ny = cy as isize + dy;
            if nx < 0 || ny < 0 || nx >= w as isize || ny >= h as isize {
                continue;
            }
            let (nx, ny) = (nx as usize, ny as usize);
            let nidx = px(nx, ny);
            if visited[nidx] {
                continue;
            }
            let np = img.get_pixel(nx as u32, ny as u32).0;
            if np[3] != 0 && matches_color(np, seed, tolerance) {
                visited[nidx] = true;
                stack.push((nx, ny));
            }
        }
    }
}

fn matches_color(p: [u8; 4], seed: [u8; 4], tolerance: u8) -> bool {
    p[0].abs_diff(seed[0]) <= tolerance
        && p[1].abs_diff(seed[1]) <= tolerance
        && p[2].abs_diff(seed[2]) <= tolerance
}

fn matches_color_rgb(p: [u8; 4], seed: [u8; 3], tolerance: u8) -> bool {
    p[0].abs_diff(seed[0]) <= tolerance
        && p[1].abs_diff(seed[1]) <= tolerance
        && p[2].abs_diff(seed[2]) <= tolerance
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::postprocess::BgScope;
    use image::Rgba;

    fn cfg() -> Config {
        Config::default()
    }

    /// 3x3: white border (bg) + red center (subject).
    fn sprite_on_white_bg() -> RgbaImage {
        let mut img: RgbaImage = RgbaImage::new(3, 3);
        for y in 0..3 {
            for x in 0..3 {
                img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
        img.put_pixel(1, 1, Rgba([255, 0, 0, 255]));
        img
    }

    #[test]
    fn outer_removes_solid_background() {
        let mut c = cfg();
        c.post_bg_remove = true;
        c.post_bg_tolerance = 0;
        c.post_bg_scope = BgScope::Outer;

        let out = flood_fill_transparent(sprite_on_white_bg(), &c);
        // Center subject pixel stays opaque.
        assert_eq!(out.get_pixel(1, 1).0, [255, 0, 0, 255]);
        // Background becomes transparent.
        assert_eq!(out.get_pixel(0, 0).0[3], 0);
        assert_eq!(out.get_pixel(2, 0).0[3], 0);
        assert_eq!(out.get_pixel(0, 2).0[3], 0);
    }

    #[test]
    fn alpha_zero_skipped_as_seed() {
        let mut c = cfg();
        c.post_bg_remove = true;
        c.post_bg_tolerance = 0;
        c.post_bg_scope = BgScope::Outer;
        let img = RgbaImage::new(3, 3);
        // All transparent: no-op.
        let out = flood_fill_transparent(img, &c);
        assert_eq!(out.get_pixel(1, 1).0[3], 0);
    }

    #[test]
    fn tolerance_widens_match() {
        // bg 250,250,250; tol 10 should treat it as ~white and remove
        let mut img: RgbaImage = RgbaImage::new(3, 3);
        for y in 0..3 {
            for x in 0..3 {
                img.put_pixel(x, y, Rgba([250, 250, 250, 255]));
            }
        }
        img.put_pixel(1, 1, Rgba([0, 0, 0, 255]));
        let mut c = cfg();
        c.post_bg_remove = true;
        c.post_bg_tolerance = 10;
        c.post_bg_scope = BgScope::All;
        let out = flood_fill_transparent(img, &c);
        assert_eq!(out.get_pixel(0, 0).0[3], 0);
        assert_eq!(out.get_pixel(1, 1).0[3], 255);
    }

    #[test]
    fn all_strips_interior_bg_pocket() {
        // 5x5: white border, red inner ring, white interior pocket at center.
        // Border targets = {white}; All strips the pocket but spares the ring.
        let mut img: RgbaImage = RgbaImage::new(5, 5);
        for y in 0..5u32 {
            for x in 0..5u32 {
                let on_border = x == 0 || y == 0 || x == 4 || y == 4;
                let color = if on_border {
                    Rgba([255, 255, 255, 255])
                } else {
                    Rgba([255, 0, 0, 255])
                };
                img.put_pixel(x, y, color);
            }
        }
        img.put_pixel(2, 2, Rgba([255, 255, 255, 255])); // interior white pocket
        let mut c = cfg();
        c.post_bg_remove = true;
        c.post_bg_tolerance = 0;
        c.post_bg_scope = BgScope::All;
        let out = flood_fill_transparent(img, &c);
        assert_eq!(out.get_pixel(2, 2).0[3], 0, "interior white pocket stripped by All");
        assert_eq!(out.get_pixel(0, 0).0[3], 0, "border white stripped");
        assert_eq!(out.get_pixel(1, 1).0[3], 255, "red ring not a border color survives");
        assert_eq!(out.get_pixel(2, 1).0[3], 255, "red ring survives");
    }

    #[test]
    fn floating_cleanup_keeps_largest_component() {
        // 5x1: one 3px cluster (largest) + one 1px speckle.
        let mut img = RgbaImage::new(5, 1);
        for x in 0..3 {
            img.put_pixel(x, 0, Rgba([255, 255, 255, 255]));
        }
        img.put_pixel(4, 0, Rgba([255, 255, 255, 255]));
        remove_small_floating_components(&mut img, 2);
        assert_eq!(img.get_pixel(0, 0).0[3], 255);
        assert_eq!(img.get_pixel(1, 0).0[3], 255);
        assert_eq!(img.get_pixel(2, 0).0[3], 255);
        assert_eq!(img.get_pixel(4, 0).0[3], 0);
    }

    #[test]
    fn floating_zero_threshold_is_noop() {
        let mut img = RgbaImage::new(2, 2);
        img.put_pixel(0, 0, Rgba([0, 0, 0, 255]));
        img.put_pixel(1, 1, Rgba([0, 0, 0, 255]));
        remove_small_floating_components(&mut img, 0);
        assert_eq!(img.get_pixel(0, 0).0[3], 255);
        assert_eq!(img.get_pixel(1, 1).0[3], 255);
    }

    #[test]
    fn floating_small_main_object_survives_as_largest() {
        // A single 1px object is the largest (and only) -> kept.
        let mut img = RgbaImage::new(3, 3);
        img.put_pixel(1, 1, Rgba([99, 99, 99, 255]));
        remove_small_floating_components(&mut img, 5);
        assert_eq!(img.get_pixel(1, 1).0[3], 255, "largest survives even if small");
    }
}
