# Phase 6 Cleanup Design

**Date:** 2026-07-25
**Status:** Draft (awaiting review)
**Related:** Phase 6 MVP (commits `72e7951`→`2b32c1c`); [review findings](../plans/2026-07-25-phase6-mvp-web.md) from the post-pull review

## Background

Phase 6 MVP landed (all 10 plan tasks done, Rust core still 68 tests green, adapter logic correct: 5 Vitest pass). But the post-pull code review found **3 CRITICAL + 5 HIGH + several MEDIUM** issues that make the app effectively unusable in its current state: re-processing is blocked after the first result, candidate-grid clicks don't update config, and a new Web Worker (with full WASM re-init) is created on every click — directly violating the U12.5 non-blocking guarantee. This cleanup fixes them.

Good news: the `wasm/wasm-loader.ts` singleton (with proper worker pooling, message-type checking, and buffer transfer) is **already written and tested** — `App.tsx` just doesn't use it (it inlines per-call worker logic instead). So several fixes collapse to "wire App to the existing wasm-loader."

## Scope — the fixes

Grouped by the fix (not by severity), since several findings share one root cause.

### Fix A: Wire `App.tsx` to `wasm-loader.ts` (kills 4 findings)

Replace `App.tsx`'s inline per-call `new Worker()` + `worker.terminate()` logic with calls to the existing `processInWorker` / `detectInWorker` from `wasm/wasm-loader.ts`. This single change fixes:

- **CRITICAL #1** — worker-per-call (WASM re-init every click, defeats U12.5). `wasm-loader.getWorker()` is a singleton; `init()` runs once.
- **HIGH #5** — no transferable buffer. `wasm-loader.processInWorker` already passes `[bytes.buffer]`.
- **CRITICAL/HIGH #6** — loose `onmessage` matching (resolves on any non-error message) + the fire-and-forget `handleDetect` race. `wasm-loader` checks `e.data.type === "process_done"` / `"detect_done"` explicitly, and its promise-per-call model is race-free.
- **Dead code** — `wasm-loader.ts` becomes live.

Also **remove the redundant post-process `handleDetect` call** (`App.tsx:111`). Detection already runs on upload (`UploadZone`); re-running it after every process spawns a second worker for no benefit. If candidate refresh on detect-strategy change is wanted later, wire it to a `detect_strategy` `useEffect` (post-cleanup).

### Fix B: Allow re-processing (CRITICAL #2)

`canProcess` in `App.tsx:150` includes `&& !result`, permanently disabling "Run" after the first result. Remove the `!result` term so users can tune params and re-run. New gate: `status === "ready" && inputBytes && inputMeta && status !== "processing"` (i.e., disabled only while a process is in flight).

### Fix C: Wire candidate-grid click to config (CRITICAL #3)

`CandidateGrid.tsx` currently only toggles visual selection. Per U2.2, clicking a candidate must set `config.detect_strategy = candidate.detector` (and `pixel_size_override = candidate.scale` when `cut_method === "Uniform"`), then auto-reprocess (the one auto-reprocess exception in the button-driven model). Wire the click handler to `setConfig` + trigger the same process path as the Run button.

### Fix D: Stop leaking object URLs (HIGH #4 + #8)

- `CompareView.tsx` creates a new `URL.createObjectURL` on **every render** (line ~11). Move to `useMemo` keyed on `inputBytes`/`result`, revoke on unmount/replace.
- `store.setResult` doesn't revoke the previous `result.url`. Revoke the old URL before replacing (and in `reset`).

### Fix E: Type the candidates (HIGH #7)

`store.ts` uses `candidates: any[]`; `CandidateGrid` uses `(c: any)`. Move the `Candidate` interface out of `wasm-loader.ts` into a shared `web/src/types.ts` (or `store.ts`), and type the store + `CandidateGrid` against it.

### Fix F: Delete the wasted main-thread WASM init (reviewer-adjacent)

`App.tsx:42-59` dynamically imports + `init()`s the WASM on the **main thread**, but the main thread never calls `process_image` (all processing goes through the worker, which has its own `init`). This needlessly downloads/parses 480KB on the main thread, blocking initial render. Delete the `useEffect` init block — the worker owns the WASM lifecycle. Change the store's default `status` from `"loading_wasm"` to `"ready"` (the worker lazy-inits on first use; there's no main-thread async to wait for, so `"loading_wasm"` would never resolve on its own anyway).

### Fix G: Cheap MEDIUM cleanups (bundle)

- **postprocess collapsed**: `forms/pipeline-uiSchema.ts` — add `"ui:collapsible": true, "ui:collapsed": true` to the `postprocess` object (currently all 10 fields expanded → overwhelming form, the exact risk the spec flagged).
- **`liveValidate` off**: `ConfigForm.tsx:28` — `liveValidate` is shorthand `true`; set `liveValidate={false}` (validation noise while typing).
- **Reset icon**: `App.tsx:187` — replace the unicode `↻` (`&#x21bb;`) with `<RotateCcw />` from lucide-react (MASTER.md: no unicode/emoji icons).
- **Global paste**: add a `window` `"paste"` listener in `App` (the plan had it; current code only pastes when `UploadZone` is focused).
- **Header status**: `Header.tsx` — show all 4 states (`loading_wasm` / `ready` / `processing` / `error`) with the colored dot, not just `processing`.
- **Error dismissible**: make the error banner dismissible (an × that calls `setError(null)` → status back to `ready`), so an error isn't a dead end requiring Reset.

## Non-Goals

- Layout rework (download-button placement, two-pane vs stacked refinements) — defer; current layout is functional.
- `prefers-reduced-motion` polish, broader `.pixelated` CSS selector, reactivity nits — defer (LOW).
- Per-candidate rendered preview thumbnails (currently shows input thumbnail + metadata) — post-MVP polish.
- New features (presets/sessions/palette editor) — those are post-MVP Phase 6 sections, not cleanup.

## Verification / acceptance

- **Vitest**: existing `adapter.test.ts` still 5 pass; add a `store.test.ts` for `setResult` URL revocation + `setConfig` patch behavior (the two store bugs).
- **Manual e2e** (the plan's 7-step, now actually possible):
  1. Upload → metadata + candidate grid populate.
  2. Run → result in slider compare, Summary shows dims/colors/time.
  3. **Change k_colors → Run again** (was blocked; now works — Fix B).
  4. **Click a candidate → detect strategy updates → result changes** (Fix C).
  5. **Run repeatedly** — no UI freeze, no per-click WASM reload lag (Fix A).
  6. Download → valid PNG.
  7. DevTools Memory: re-process 5× — no Blob-URL leak growth (Fix D).
- **Build**: `cd web && npm run build` (tsc + vite) clean; `cargo test` still 68; `wasm-pack build` produces `pkg/pixel_game_kit.js`; bundle size monitored (RJSF may push ~250KB — note, don't block).
- **Type safety**: `npx tsc --noEmit` clean, no new `any` (Fix E).

## Risks

| Risk | Mitigation |
|------|------------|
| Wiring App to wasm-loader changes the message-flow shape | wasm-loader is already tested-by-design (singleton + type checks); verify with the e2e steps 3-5 |
| Candidate auto-reprocess could surprise users (click = re-run) | spec already designated candidate-click as the one auto-reprocess exception; show a brief "processing" state on click |
| Removing main-thread init changes "ready" timing | flip status to ready on first worker success (or default ready); e2e step 1 confirms upload still works |
| Fix B (allow re-processing) + Fix D (revoke URLs) interact — revoking the displayed result URL mid-render | revoke the OLD url when setting a NEW result (not before); CompareView's useMemo re-derives from new result |
