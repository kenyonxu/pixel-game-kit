//! Phase 4 postprocess integration tests.
//!
//! Cross-platform: `sha2` crate + `std::env::temp_dir` (mirrors `tests/resample.rs`).
//! Inline unit tests under `src/postprocess/` cover algorithm correctness; these
//! cover the end-to-end guarantees: default-config anchor lock, determinism (R1),
//! and CLI wiring of the new flags.

use sha2::{Digest, Sha256};
use std::fs;
use std::process::Command;

fn tmp(name: &str) -> String {
    let mut p = std::env::temp_dir();
    p.push(format!("pixel-snapper-p4-{}", name));
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

/// Default config (all postprocess off) -> Phase 3 Oklab anchor unchanged.
/// This is the zero-regression gate for the whole stage.
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
        "default config must match Phase 3 Oklab anchor (postprocess off)"
    );
}

/// Determinism (R1): same image + same postprocess config twice -> byte-identical.
/// All postprocess ops are RNG-free, so this must hold exactly.
#[test]
fn determinism_byte_identical() {
    let a = tmp("det_a.png");
    let b = tmp("det_b.png");
    assert!(run_cli(&[
        "tests/fixtures/baseline/ai-sprite.png",
        a.as_str(),
        "16",
        "--bg-remove",
        "--alpha-threshold",
        "auto",
        "--morph",
        "--bg-floating-threshold",
        "4",
    ]));
    assert!(run_cli(&[
        "tests/fixtures/baseline/ai-sprite.png",
        b.as_str(),
        "16",
        "--bg-remove",
        "--alpha-threshold",
        "auto",
        "--morph",
        "--bg-floating-threshold",
        "4",
    ]));
    assert_eq!(sha256(&a), sha256(&b), "same config must be byte-identical");
}

/// End-to-end: --outline grows the output by exactly +2 on each axis (vs the
/// same run without --outline) and the default black color appears. Compares
/// against a no-outline baseline so the assertion is independent of which grid
/// detection picks.
#[test]
fn outline_grows_output_via_cli() {
    let mut img: image::RgbaImage = image::ImageBuffer::new(16, 16);
    for y in 6..10 {
        for x in 6..10 {
            img.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
        }
    }
    let input = tmp("outline_in.png");
    let base = tmp("outline_base.png");
    let outlined = tmp("outline_out.png");
    img.save(&input).expect("save input");
    assert!(run_cli(&[input.as_str(), base.as_str(), "16"]));
    assert!(run_cli(&[
        input.as_str(),
        outlined.as_str(),
        "16",
        "--outline",
        "sharp",
    ]));
    let base_img = image::open(&base).expect("open baseline").to_rgba8();
    let out_img = image::open(&outlined).expect("open output").to_rgba8();
    assert_eq!(
        out_img.dimensions(),
        (base_img.width() + 2, base_img.height() + 2),
        "outline must pad +1 on every side"
    );
    let has_black = out_img
        .pixels()
        .any(|p| p[0] == 0 && p[1] == 0 && p[2] == 0 && p[3] == 255);
    assert!(has_black, "default black outline color should appear");
}

/// --bg-remove produces a valid PNG with transparent pixels (background removed).
#[test]
fn bg_remove_via_cli_produces_transparency() {
    let out = tmp("bgremove.png");
    assert!(run_cli(&[
        "tests/fixtures/baseline/ai-sprite.png",
        out.as_str(),
        "16",
        "--bg-remove",
    ]));
    let img = image::open(&out).expect("open output").to_rgba8();
    let any_transparent = img.pixels().any(|p| p[3] == 0);
    assert!(
        any_transparent,
        "bg-remove should yield some transparent pixels"
    );
}
