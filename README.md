# Pixel Game Kit

A tool that fixes AI-generated pixel art by detecting its implicit pixel grid and re-snapping to it. The same Rust codebase compiles to a **native CLI** and a **WASM module** — deterministic, palette-preserving, and tuned for game-engine-ready output.

<img src="./static/hero.png" alt="Pixel Game Kit" style="width: 100%; image-rendering: pixelated;">

## Why?

**Current AI image models can't produce consistent grid-based pixel art.**

- Pixels are inconsistent in size and position.
- The grid resolution drifts across the image.
- Colors aren't tied to a strict palette.

**With Pixel Game Kit:**

- ✅ Pixels are snapped to a perfect grid (integer or skewed/elastic).
- ✅ The grid is detected automatically — or override it explicitly.
- ✅ Colors are quantized to a strict palette (Oklab, perceptually uniform).
- ✅ Output is cleanable in-place: background removal, outline, morphology, alpha binarize.
- ✅ Deterministic (`seed = 42`): same input + config → byte-identical output, every time.

## Features

- **Multi-detector grid detection** — `runs` (GCD), `tiled` (Sobel + autocorrelation), `elastic` (gradient walker, handles non-integer/skewed grids); `auto` picks the best per image.
- **Resample strategies** — `majority` (palette-preserving default), `median` (anti-alias removal), `dominant`, `mode`, `qvote` (per-cell Oklab k-means).
- **Oklab quantization** (default) for perceptually smooth gradients; `--colorspace rgb` preserves the legacy RGB path.
- **Dithering** — Floyd-Steinberg, Bayer 2/4/8, Ordered.
- **Retro console palettes** — NES, GameBoy, PICO-8, Sweetie16, Endesga32, PC-9801, MSX1 (`sgb`/`snes` accepted as no-ops — no canonical palette).
- **Post-processing** (Phase 4) — flood-fill background removal (`outer`/`all`), floating-island cleanup, 2×2 morphology (alpha-only, preserves palette), alpha binarize (fixed threshold or Otsu auto), 1px outline (sharp/rounded).
- **Dual-target** — one codebase → native CLI binary + WASM module for web.
- **Deterministic** — all RNG flows through a seeded `ChaCha8Rng`; refactors stay byte-identical.

## 💻 CLI

### Install (from source)

```bash
git clone https://github.com/kenyonxu/pixel-game-kit.git
cd pixel-game-kit
cargo install --path .         # installs the `pixel-game-kit` binary
```

### Usage

```text
pixel-game-kit <INPUT> <OUTPUT> [COLOR_COUNT] [OPTIONS]
```

- `<INPUT>`: a PNG/JPEG image, or a directory for batch processing.
- `<OUTPUT>`: an output PNG, or a different output directory for a batch.
- `[COLOR_COUNT]`: number of palette colors. Defaults to `16`.

#### Options

| Flag | Values | Default | Notes |
|---|---|---|---|
| `--pixel-size` | positive number | auto | Override the detected pixel size |
| `--palette` | `HEX,HEX,...` | — | Constrain to a custom palette |
| `--detect` | `auto\|runs\|tiled\|elastic` | `auto` | Grid detection strategy |
| `--resample` | `majority\|median\|dominant\|mode\|qvote` | `majority` | Grid-cell reduction |
| `--sample-window` | `1`–`9` | `3` | Median neighborhood |
| `--colorspace` | `rgb\|oklab` | `oklab` | Quantize colorspace |
| `--dither` | `none\|fs\|bayer2\|bayer4\|bayer8\|ordered` | `none` | Dithering |
| `--dither-strength` | `0`–`2` | `1.0` | Dither strength |
| `--preset` | `none\|nes\|gameboy\|pico8\|sweetie16\|endesga32\|...` | `none` | Snap to a preset palette |
| `--bg-remove` | — | off | Enable background flood-fill removal |
| `--bg-tolerance` | `0`–`255` | `64` | Per-channel bg color tolerance |
| `--bg-connectivity` | `4\|8` | `4` | Flood connectivity |
| `--bg-scope` | `outer\|all` | `outer` | Removal scope |
| `--bg-floating-threshold` | int | `0` (off) | Erase floating blobs ≤ N px (largest always kept) |
| `--outline` | `none\|rounded\|sharp` | `none` | 1px outline (pads canvas +2) |
| `--outline-color` | hex | `000000` | Outline color |
| `--morph` | — | off | 2×2 open→close (alpha-only) |
| `--alpha-threshold` | `0`–`255` or `auto` | off | Alpha binarize (strict `>`) |
| `--json` | — | off | Emit detection candidates as JSON instead of processing |

#### Examples

```bash
# Quantize to a 16-color Oklab palette
pixel-game-kit input.png output.png 16

# Batch a directory
pixel-game-kit sprites/inputs sprites/outputs 16

# Override the detected pixel size
pixel-game-kit input.png output.png --pixel-size 8

# Oklab quantize + Floyd-Steinberg dither + NES palette
pixel-game-kit input.png output.png 16 --dither fs --preset nes

# Custom palette
pixel-game-kit input.png output.png --palette "0d2b45,203c56,544e68,8d697a,d08159,ffaa5e,ffd4a3,ffecd6"

# Postprocess: remove background + sharp black outline + clean speckles
pixel-game-kit input.png output.png 16 --bg-remove --outline sharp --morph
```

Run `pixel-game-kit --help` for the canonical list.

### Build from source

```bash
git clone https://github.com/kenyonxu/pixel-game-kit.git
cd pixel-game-kit
cargo build --release      # → target/release/pixel-game-kit
cargo test                 # unit + integration tests across all phases
```

## 🌐 Web (WASM)

```bash
git clone https://github.com/kenyonxu/pixel-game-kit.git
cd pixel-game-kit
wasm-pack build --target web --out-dir pkg --release   # → pkg/pixel_game_kit.js
```

Then in JS:

```js
import init, { process_image } from "./pkg/pixel_game_kit.js";
await init();

// process_image(inputBytes, kColors?, pixelSizeOverride?, paletteHex?,
//               detectStrategy?, resampleMethod?, colorspace?, dither?,
//               presetPalette?, postConfig?)
// Pass null for any optional arg to leave it at its default.

// Default: 16-color Oklab quantize
const outputBytes = process_image(inputBytes, 16);

// Oklab + Floyd-Steinberg dither + NES palette
process_image(inputBytes, 16, null, null, null, null, "oklab", "fs", "nes", null);

// Custom palette
process_image(inputBytes, 16, null, "0f0f1b,ffecd6,ff4d6d,29adff", null, null, null, null, null, null);

// Postprocess (Phase 4): bg removal + sharp outline, via a JSON config string
process_image(inputBytes, 16, null, null, null, null, null, null, null,
  JSON.stringify({ bg_remove: true, outline: "sharp", alpha_threshold: "auto" }));
```

`detect_candidates(inputBytes, kColors?, detectStrategy?)` returns a JSON string of ranked grid-detection candidates (for building a candidate-picker UI).

## Acknowledgments

This project is a heavily modified fork of
[Hugo-Dz/spritefusion-pixel-snapper](https://github.com/Hugo-Dz/spritefusion-pixel-snapper)
(MIT) — substantially rewritten and renamed to `pixel-game-kit`, with a new modular
pipeline (multi-detector grid detection, resampling strategies, Oklab quantization,
dithering, retro palettes, post-processing).

Algorithm inspiration — clean-room re-implementations in Rust (no source code copied):

- **[PixelRefiner](https://github.com/HappyOnigiri/PixelRefiner)** — flood-fill
  background removal, outline, Oklab color quantization, dithering, console palettes.
- **[unfake.js](https://github.com/jenissimo/unfake.js)** — runs/tiled grid detectors,
  dominant/mode resampling, 2×2 morphology.

Original project by [Hugo Duprez](https://www.hugoduprez.com/), a
[Sprite Fusion](https://www.spritefusion.com/pixel-art-generator) project.

<img src="./static/spritefusion-generator.webp" alt="Sprite Fusion Pixel Art Generator" style="width: 100%;">

## License

MIT License. Based on the original work of [Hugo Duprez](https://www.hugoduprez.com/); modifications in this fork by Kai Xu. See [LICENSE](LICENSE).
