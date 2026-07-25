# Phase 6 MVP — Web App Design

**Date:** 2026-07-25
**Status:** Draft (awaiting review)
**Related:** [PLAN.md](../../../PLAN.md) Phase 6 (6A + the 🔴 Web stories); [USER_STORIES.md](../../../USER_STORIES.md) v1.0 MVP

## Background

The Rust core (Phases 0–5.5) implements every 🔴 algorithm story. What's missing is a **real Web frontend**. The current root `index.html` is a minimal vanilla-JS trial: 3 params (k_colors / pixel-size / palette), **synchronous** `process_image` on the main thread (blocks UI), side-by-side compare (not slider), no candidate grid — and it imports `./pkg/spritefusion_pixel_snapper.js`, the **pre-rename** package name (the crate is now `pixel-game-kit` → `pkg/pixel_game_kit.js`), so the trial page is effectively broken.

Phase 6 (8 sections 6A–6H) is too large for one implementation plan. This spec defines the **MVP slice** — the first shippable web app — and defers the rest.

## Scope — MVP (single image, the 🔴 Web stories)

A real React/shadcn web app at `web/` that delivers the core loop: **upload → (candidate-grid pick) → tune params → process (Web Worker, non-blocking) → slider compare → download**, replacing the broken `index.html`.

Covers (PLAN 6A + 🔴 Web stories):
- **6A scaffold**: `web/` Vite + React + TS + shadcn/ui + zustand + RJSF + `vite-plugin-wasm`; WASM loaded from `pkg/pixel_game_kit.js` (fixes the rename bug); **Web Worker** wrapping `process_image` / `detect_candidates` (U12.5 non-blocking).
- **Schema**: `schema/pipeline-config.schema.json` — complete JSON Schema of `PipelineConfig` (single source of truth; reused by 6B/6H later).
- **RJSF default form** bound to the schema (shadcn widget mapping deferred — PLAN risk note: MVP uses RJSF defaults).
- **Upload** (U1.3 drag/paste/select) + **input metadata** (U1.5 size/colors/alpha).
- **Candidate grid** (U2.2): `detect_candidates` → top-3 thumbnails + confidence; click to select detector → re-process.
- **Slider compare** (U7.1): react-compare-slider, pixelated render.
- **Summary** (U7.3): detector / step / output dims / colors / time / fallback flag.
- **Download** PNG (U8.1).

## Non-Goals (deferred to post-MVP, sequenced separately)

- **6B** presets (named save/load, built-in scene presets, import/export).
- **6C** sessions / multi-image / history / batch ZIP.
- **6D** palette editor (visualize + edit result palette, export `.hex`/`.gpl`).
- **6E** magnifier (U7.2; slider compare IS in MVP, magnifier is not).
- **6F** export extras (scale x2/x4/…, auto-trim, force-size, ZIP).
- **6G** recipe (PNG `zTXt` embed + drag-back form fill + `--dump-recipe`).
- **6H** Rust-side `Config ↔ PipelineConfig` serde + schema version migration + CI schema validation. (The **schema file** is built here; the Rust serde/migration is deferred.)
- shadcn↔RJSF custom widget mapping (post-MVP polish).
- Live/reprocess-on-every-keystroke (MVP is button-driven).
- Production deployment to spritefusion.com (MVP runs on the local Vite dev server; deployment is a separate concern).

## Architecture

```
web/
  index.html              # Vite entry (replaces root index.html)
  src/
    main.tsx
    App.tsx               # layout: upload + form + result
    store.ts              # zustand: { inputBytes, inputMeta, config, result, candidates, status }
    wasm/worker.ts        # Web Worker: loads wasm, exposes process() + detectCandidates()
    wasm/adapter.ts       # configToWasm(PipelineConfig) -> { positional: [...10], post_config: string }
    components/
      UploadZone.tsx      # U1.3 drag/paste/select + U1.5 metadata
      ConfigForm.tsx      # RJSF bound to schema, uiSchema grouping
      CandidateGrid.tsx   # U2.2 top-3 candidates, click-to-select
      CompareView.tsx     # U7.1 slider compare
      Summary.tsx         # U7.3 processing summary
    forms/
      pipeline-uiSchema.ts  # uiSchema: core params visible, postprocess/advanced collapsed
schema/
  pipeline-config.schema.json   # single source of truth (all backend params)
```

Replaces root `index.html` (deleted — it's broken). Stack decided in USER_STORIES: React + Vite + shadcn/ui + RJSF + zustand.

## Schema (single source of truth)

`schema/pipeline-config.schema.json` describes the **complete** `PipelineConfig` — every backend parameter, so 6B (presets) and 6H (serde) reuse it without rework. Fields (snake_case to align with the Rust `Config` serde that 6H will wire):

- `k_colors` (integer, >0, default 16)
- `pixel_size_override` (number | null, range [1, min(w,h)/2])
- `palette` (array of hex strings, e.g. `["0d2b45","ffecd6"]`; adapter joins to comma string)
- `detect_strategy` (enum: auto/runs/tiled/elastic)
- `resample_method` (enum: majority/median/dominant/mode/qvote)
- `colorspace` (enum: oklab/rgb)
- `dither` (enum: none/fs/bayer2/bayer4/bayer8/ordered)
- `preset_palette` (enum: none/nes/gameboy/sgb/snes/pc9801/msx1/pico8/sweetie16/endesga32)
- `postprocess` (object: `bg_remove`, `bg_tolerance`, `bg_connectivity`, `bg_scope`, `bg_floating_threshold`, `outline`, `outline_color`, `morph`, `alpha_threshold`)

RJSF renders the schema; `pipeline-uiSchema.ts` shows **core params** (k_colors, pixel_size_override, palette, detect_strategy, colorspace, dither, preset_palette) prominently and collapses `postprocess` + `resample_method` (advanced) by default.

## Components

- **`<UploadZone>`** — drag-drop, paste (Ctrl+V), file-picker. On upload: store bytes, run `analyzeInput` (canvas decode → unique opaque color count + dims), show metadata (U1.5).
- **`<ConfigForm>`** — RJSF `<Form schema={pipelineSchema} uiSchema={pipelineUiSchema} formData={config} onChange={...} />`. MVP uses RJSF default widgets (shadcn mapping deferred).
- **`<CandidateGrid>`** — on upload (or detect-strategy change), call `detect_candidates(bytes, k_colors, detect_strategy)` → render top-3 candidates as pixelated thumbnails with confidence + detector label. Click a candidate → set `config.detect_strategy = candidate.detector` (and if `cut_method == Uniform`, `config.pixel_size_override = candidate.scale`) → auto-reprocess. Highlight the selected candidate.
- **`<CompareView>`** — react-compare-slider with original (left) vs result (right), both `image-rendering: pixelated`. Empty state until a result exists.
- **`<Summary>`** — output dims / output color count / processing time. The detector / detected step / confidence (U7.3) are surfaced in the `<CandidateGrid>` for the selected candidate (the wasm `process_image` returns only PNG bytes, so detector/step metadata comes from `detect_candidates`, not the process result).
- **Download button** — `<a download>` on the result blob (U8.1).

## Data flow

1. **Upload** → `inputBytes` + `inputMeta` into the zustand store → `<UploadZone>` renders preview + metadata; triggers `detect_candidates` → `<CandidateGrid>` populates.
2. **Tune form** → `<ConfigForm>` `onChange` updates `config` in the store (no auto-process).
3. **Process** (button click, or candidate-grid click): store dispatches `process` → `worker.postMessage({ bytes: inputBytes, config })`.
4. **Worker**: `configToWasm(config)` → calls `process_image(bytes, ...10 positional, post_config)` → returns PNG bytes → `{ resultBytes, summary }` back to store.
5. Store updates `result` + `status` → `<CompareView>` + `<Summary>` render.
6. **Download**: blob from `resultBytes`.

Processing is **button-driven** (not live) to avoid worker thrash/debounce complexity. The single auto-reprocess exception is candidate-grid selection (explicit user intent).

## Worker + WASM adapter

**`web/src/wasm/worker.ts`** — loads `pkg/pixel_game_kit.js` (via `vite-plugin-wasm`), exposes two message handlers:
- `{type: "process", bytes, config}` → `{resultBytes, elapsedMs}` (errors as `{error}`).
- `{type: "detectCandidates", bytes, config}` → `{candidates}` (the JSON string from `detect_candidates`, parsed).

**`web/src/wasm/adapter.ts`** — `configToWasm(config: PipelineConfig)` maps the JSON form state to the WASM `process_image` signature (10 positional `Option` params + `post_config` JSON string):

```
process_image(
  bytes,
  config.k_colors,
  config.pixel_size_override,
  config.palette?.length ? config.palette.join(",") : null,   // palette_hex (empty array -> null = auto)
  config.detect_strategy,
  config.resample_method,
  config.colorspace,
  config.dither,
  config.preset_palette,
  JSON.stringify(config.postprocess)                   // post_config (serde(default) tolerates off fields)
)
```

This adapter is the glue between the schema-driven form and the existing positional WASM API. (When 6H lands Rust serde, the adapter may simplify to a single config-JSON arg — but that's out of scope here.)

## Testing / acceptance

- `cd web && npm install && npm run dev` serves the app; `wasm-pack build` must run first (prerequisite).
- **End-to-end**: upload an AI sprite → defaults produce a snapped result → slider compare works → download produces a valid PNG.
- **Worker non-blocking (U12.5)**: processing a large image (>2s) does not freeze the UI (form still editable, candidate grid still clickable).
- **Candidate grid (U2.2)**: populates from `detect_candidates`; selecting a candidate updates `detect_strategy` and re-processes.
- **Adapter unit test (Vitest)**: `configToWasm` maps representative configs correctly (positional order + `post_config` JSON shape); determinism (same config → same args).
- **Bundle size**: monitor gzipped `dist/` (target < 250KB per PLAN; RJSF + React + shadcn may approach this — flag if exceeded, code-split `<CandidateGrid>`/`<CompareView>` lazily as mitigation).
- Frontend test scope is minimal for MVP (adapter unit test + a Vitest component smoke or two); the deterministic core is Rust, already covered by 68 tests.

## Risks

| Risk | Mitigation |
|------|------------|
| RJSF default form is ugly / overwhelming (25 fields) | uiSchema groups: core visible, postprocess + resample collapsed; shadcn mapping deferred to post-MVP polish |
| Bundle size: RJSF + React + shadcn may exceed 250KB gz | monitor; lazy-load `<CandidateGrid>` / `<CompareView>` if needed |
| WASM loading in Vite needs `vite-plugin-wasm` + top-level await | async loading state in the store; show "加载 WASM 中…" until ready |
| Schema authored by hand may drift from Rust `Config` | keep snake_case aligned; 6H will wire Rust serde to enforce it (CI schema validation) |
| `process_image` 10 positional params are error-prone in the adapter | `configToWasm` is unit-tested (Vitest) for positional order; documented inline |
| Trial `index.html` deletion removes a zero-install demo | the new `web/` app IS the zero-install demo (U1.2), served by Vite / static host |

## Open questions

None — all forks resolved during brainstorming:
1. MVP cut = 6A + 🔴 Web stories, single image (6B–6H deferred).
2. Form = build the schema now + RJSF default widgets (shadcn mapping deferred).
3. Stack = React + Vite + shadcn/ui + RJSF + zustand (decided in USER_STORIES).
4. Processing = button-driven (not live); candidate-grid click is the one auto-reprocess exception.
