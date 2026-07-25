//! Phase 5.5 palette-lock integration tests. Cross-platform (sha2 + temp_dir).
//!
//! These cover the end-to-end guarantees: default-config anchor unchanged
//! (zero-regression), `--palette-from` locks drifted frames to a reference, and
//! explicit `--palette` takes precedence over `--palette-from`.

use sha2::{Digest, Sha256};
use std::fs;
use std::process::Command;

fn tmp(name: &str) -> String {
    let mut p = std::env::temp_dir();
    p.push(format!("pixel-snapper-p55-{}", name));
    p.to_string_lossy().to_string()
}

fn run_cli(args: &[&str]) -> bool {
    let bin = env!("CARGO_BIN_EXE_pixel-game-kit");
    Command::new(bin)
        .args(args)
        .output()
        .expect("failed to run CLI")
        .status
        .success()
}

fn sha256(path: &str) -> String {
    let data = fs::read(path).expect("output file not written");
    let mut hasher = Sha256::new();
    hasher.update(&data);
    format!("{:x}", hasher.finalize())
}

/// Default config (no --palette-from) -> Oklab anchor unchanged. Zero-regression
/// gate for the whole phase.
#[test]
fn default_config_anchor_unchanged() {
    let out = tmp("anchor.png");
    assert!(run_cli(&[
        "tests/fixtures/baseline/ai-sprite.png",
        out.as_str(),
        "16",
    ]));
    assert_eq!(
        sha256(&out),
        "3a589ee93b8cd2e493baa0d6fb314d279b54a1104165ad754ad4ff6d359e4420",
        "default config anchor must be unchanged by Phase 5.5"
    );
}

/// `--palette-from` locks drifted frames to the reference: two frames with
/// slightly different reds, each snapped via the SAME reference, produce the
/// SAME single-color output (the reference's red).
#[test]
fn palette_from_locks_across_frames() {
    let frame_a: image::RgbaImage = image::ImageBuffer::from_pixel(16, 16, image::Rgba([220, 40, 40, 255]));
    let frame_b: image::RgbaImage = image::ImageBuffer::from_pixel(16, 16, image::Rgba([215, 45, 38, 255]));
    let reference: image::RgbaImage =
        image::ImageBuffer::from_pixel(16, 16, image::Rgba([218, 42, 39, 255]));

    let ref_path = tmp("ref.png");
    let in_a = tmp("frame_a.png");
    let in_b = tmp("frame_b.png");
    let out_a = tmp("out_a.png");
    let out_b = tmp("out_b.png");
    reference.save(&ref_path).unwrap();
    frame_a.save(&in_a).unwrap();
    frame_b.save(&in_b).unwrap();

    assert!(run_cli(&[
        in_a.as_str(),
        out_a.as_str(),
        "16",
        "--pixel-size",
        "1",
        "--palette-from",
        ref_path.as_str(),
    ]));
    assert!(run_cli(&[
        in_b.as_str(),
        out_b.as_str(),
        "16",
        "--pixel-size",
        "1",
        "--palette-from",
        ref_path.as_str(),
    ]));

    let img_a = image::open(&out_a).unwrap().to_rgba8();
    let img_b = image::open(&out_b).unwrap().to_rgba8();
    let uniq_a: std::collections::HashSet<[u8; 4]> = img_a.pixels().map(|p| p.0).collect();
    let uniq_b: std::collections::HashSet<[u8; 4]> = img_b.pixels().map(|p| p.0).collect();
    assert_eq!(uniq_a, uniq_b, "both frames snapped to the same palette");
    assert_eq!(
        uniq_a.len(),
        1,
        "single-color reference -> single-color output (drift eliminated)"
    );
}

/// `--palette` (explicit) takes precedence over `--palette-from`.
#[test]
fn explicit_palette_wins_over_palette_from() {
    let reference: image::RgbaImage = image::ImageBuffer::from_pixel(8, 8, image::Rgba([255, 0, 0, 255]));
    let input: image::RgbaImage = image::ImageBuffer::from_pixel(8, 8, image::Rgba([250, 5, 5, 255]));
    let ref_path = tmp("ref2.png");
    let in_path = tmp("in2.png");
    let out_palette = tmp("out_palette.png");
    let out_both = tmp("out_both.png");
    reference.save(&ref_path).unwrap();
    input.save(&in_path).unwrap();

    assert!(run_cli(&[
        in_path.as_str(),
        out_palette.as_str(),
        "16",
        "--pixel-size",
        "1",
        "--palette",
        "0000ff",
    ]));
    assert!(run_cli(&[
        in_path.as_str(),
        out_both.as_str(),
        "16",
        "--pixel-size",
        "1",
        "--palette",
        "0000ff",
        "--palette-from",
        ref_path.as_str(),
    ]));

    let a = image::open(&out_palette).unwrap().to_rgba8();
    let b = image::open(&out_both).unwrap().to_rgba8();
    assert_eq!(
        a.get_pixel(0, 0),
        b.get_pixel(0, 0),
        "--palette overrides --palette-from"
    );
    // And the winner is blue (0000ff), not red — proves --palette won, not --palette-from.
    assert_eq!(&a.get_pixel(0, 0).0[0..3], &[0, 0, 255]);
}
