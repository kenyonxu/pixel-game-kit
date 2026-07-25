# Phase 6 Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the 3 CRITICAL + 5 HIGH + key MEDIUM bugs the post-pull review found in the Phase 6 web app, so it's actually usable (re-process works, candidate clicks update output, no per-click WASM reload, no leaks).

**Architecture:** Mostly localized fixes. The biggest one (Fix A) collapses to "wire `App.tsx` to the existing `wasm-loader.ts` singleton" (already written + correct — App just doesn't use it). Candidate-grid click wires to the same process path via an `onProcess` prop. Object-URL leaks fixed with `useMemo` + revoke-on-replace.

**Tech Stack:** React 19 + TS + zustand + RJSF + shadcn (unchanged). Verification = Vitest (store/adapter) + manual e2e (the plan's 7-step) + `npm run build`.

**Spec:** [docs/superpowers/specs/2026-07-25-phase6-cleanup-design.md](../specs/2026-07-25-phase6-cleanup-design.md)
**Prerequisite:** `pkg/pixel_game_kit.js` exists (run `wasm-pack build --target web --out-dir pkg --release` from repo root if missing — the pulled `pkg/` may be stale with the pre-rename name).

---

### Task 1: `store.ts` — types, URL revoke, default status (Fixes D-partial, E, F)

Foundation: type `candidates`, revoke prior result URL on replace, default status `ready`.

**Files:**
- Modify: `web/src/store.ts`
- Test: `web/src/__tests__/store.test.ts` (new)

- [ ] **Step 1: Write the failing store test — `web/src/__tests__/store.test.ts`**

```ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { useStore } from "../store";

describe("store", () => {
  beforeEach(() => useStore.getState().reset());

  it("setResult revokes the previous result URL", () => {
    const revoke = vi.spyOn(URL, "revokeObjectURL");
    useStore.setState({ inputBytes: new Uint8Array([1]) });
    useStore.getState().setResult({
      bytes: new Uint8Array([1]), url: "blob:first", elapsedMs: 1, outW: 2, outH: 2,
    });
    useStore.getState().setResult({
      bytes: new Uint8Array([2]), url: "blob:second", elapsedMs: 2, outW: 2, outH: 2,
    });
    expect(revoke).toHaveBeenCalledWith("blob:first");
    expect(useStore.getState().result?.url).toBe("blob:second");
    revoke.mockRestore();
  });

  it("setConfig patches config immutably", () => {
    useStore.getState().setConfig({ k_colors: 32 });
    expect(useStore.getState().config.k_colors).toBe(32);
    expect(useStore.getState().config.colorspace).toBe("oklab"); // unchanged
  });

  it("default status is ready (worker lazy-inits)", () => {
    useStore.getState().reset();
    expect(useStore.getState().status).toBe("ready");
  });
});
```

- [ ] **Step 2: Run test — expect FAIL** (`setResult` doesn't revoke; default status is `loading_wasm`)

`cd web && npx vitest run src/__tests__/store.test.ts` → 2 of 3 fail.

- [ ] **Step 3: Apply the store fixes**

In `web/src/store.ts`:

(a) Import the `Candidate` type from wasm-loader (Fix E):
```ts
import { create } from "zustand";
import type { PipelineConfig } from "./wasm/adapter";
import { DEFAULT_CONFIG } from "./wasm/adapter";
import type { Candidate } from "./wasm/wasm-loader";
```

(b) Replace `candidates: any[]` and `setCandidates: (c: any[]) => void` in the `State` interface:
```ts
  candidates: Candidate[];
  // ...
  setCandidates: (c: Candidate[]) => void;
```

(c) Revoke prior URL in `setResult` (Fix D):
```ts
  setResult: (result) =>
    set((s) => {
      if (s.result?.url) URL.revokeObjectURL(s.result.url);
      return { result, status: "ready" };
    }),
```

(d) Default status `ready` (Fix F):
```ts
  status: "ready",
```

- [ ] **Step 4: Run test — expect PASS (3/3)**

`cd web && npx vitest run src/__tests__/store.test.ts` → 3 pass.

- [ ] **Step 5: Typecheck**

`cd web && npx tsc --noEmit` → no errors (no `any` in store).

- [ ] **Step 6: Commit**

```bash
git add web/src/store.ts web/src/__tests__/store.test.ts
git commit -m "fix(phase6): typed candidates + revoke prior result URL + default status ready"
```

---

### Task 2: `App.tsx` rework — wire wasm-loader, allow re-process, drop main-thread init (Fixes A, B, F-app, G-app)

The big one. Replace inline per-call worker logic with `wasm-loader` calls; read state via `getState()` (no stale closures); remove `!result` gate; delete the wasteful main-thread WASM init; add global paste + dismissible error + Lucide Reset.

**Files:**
- Modify: `web/src/App.tsx`

- [ ] **Step 1: Replace `handleProcess` + `handleDetect` with wasm-loader calls**

In `web/src/App.tsx`, delete the inline `new Worker()` logic in `handleProcess` (lines ~61-115) and `handleDetect` (lines ~117-148). Replace `handleProcess` with:

```ts
const handleProcess = async () => {
  const { inputBytes, config } = useStore.getState();
  if (!inputBytes) return;
  setStatus("processing");
  try {
    const { configToWasm } = await import("@/wasm/adapter");
    const { processInWorker } = await import("@/wasm/wasm-loader");
    const { positional, post_config } = configToWasm(config);
    const bytes = inputBytes.slice(); // copy (transfer detaches the original)
    const { resultBytes, elapsedMs } = await processInWorker(bytes, positional, post_config);
    const url = URL.createObjectURL(new Blob([resultBytes], { type: "image/png" }));
    const outMeta = await decodeImageMeta(resultBytes);
    useStore.getState().setResult({
      bytes: resultBytes, url, elapsedMs, outW: outMeta.w, outH: outMeta.h,
    });
  } catch (e) {
    setError(String((e as Error)?.message ?? e));
  }
};
```

Delete `handleDetect` entirely (UploadZone already detects on upload — re-running it post-process was redundant and spawned a second worker).

- [ ] **Step 2: Delete the main-thread WASM init `useEffect` (lines ~42-59)**

Remove the whole `useEffect(() => { ... init() ... }, [...])` block that dynamically imports `@pkg/pixel_game_kit.js` on the main thread. The worker owns the WASM lifecycle now.

- [ ] **Step 3: Fix `canProcess` — remove `!result` (Fix B)**

```ts
const canProcess = !!inputBytes && !!inputMeta && status !== "processing";
```

(Keeps it disabled only while a process is in flight; re-processing after a result is now allowed.)

- [ ] **Step 4: Add global paste handler (Fix G)**

Add a `useEffect` (replacing the deleted init one) that registers a window paste listener:

```ts
useEffect(() => {
  const onPaste = async (e: ClipboardEvent) => {
    const item = [...(e.clipboardData?.items ?? [])].find((i) => i.type.startsWith("image/"));
    const file = item?.getAsFile();
    if (!file) return;
    const bytes = new Uint8Array(await file.arrayBuffer());
    // reuse UploadZone's analyze — import it or move analyzeInput to a util; simplest: dispatch setImage + detect
    const { detectInWorker } = await import("@/wasm/wasm-loader");
    try {
      const meta = await analyzeInputMeta(bytes, file.type); // see Step 5
      useStore.getState().setImage(bytes, meta);
      const cands = await detectInWorker(bytes, useStore.getState().config.k_colors, null);
      useStore.getState().setCandidates(cands.slice(0, 3));
    } catch (err) {
      setError(String((err as Error)?.message ?? err));
    }
  };
  window.addEventListener("paste", onPaste);
  return () => window.removeEventListener("paste", onPaste);
}, [setError]);
```

- [ ] **Step 5: Extract `analyzeInputMeta` to a shared util**

`UploadZone.tsx` currently has an inline `analyzeInput`. Extract it to `web/src/lib/image-meta.ts` so both `UploadZone` and `App`'s paste handler use it:

```ts
// web/src/lib/image-meta.ts
import type { InputMeta } from "@/store";

export async function analyzeInputMeta(bytes: Uint8Array, type: string): Promise<InputMeta> {
  const bmp = await createImageBitmap(new Blob([bytes], { type }));
  const w = bmp.width, h = bmp.height;
  const canvas = document.createElement("canvas");
  canvas.width = w; canvas.height = h;
  const ctx = canvas.getContext("2d", { willReadFrequently: true })!;
  ctx.drawImage(bmp, 0, 0);
  bmp.close();
  const { data } = ctx.getImageData(0, 0, w, h);
  const set = new Set<number>();
  let hasAlpha = false;
  for (let i = 0; i < data.length; i += 4) {
    if (data[i + 3] === 0) { hasAlpha = true; continue; }
    set.add((data[i] << 16) | (data[i + 1] << 8) | data[i + 2]);
  }
  return { w, h, colors: set.size, hasAlpha };
}
```

Update `UploadZone.tsx` to import from `@/lib/image-meta` and delete its inline copy.

- [ ] **Step 6: Dismissible error + Lucide Reset (Fix G)**

Add an × on the error banner and swap the unicode Reset icon:

```tsx
import { RotateCcw, X } from "lucide-react";
// ...
{error && (
  <div className="text-xs text-destructive bg-destructive/10 rounded px-2 py-1 flex items-center justify-between gap-2">
    <span className="truncate">{error}</span>
    <button onClick={() => useStore.getState().setError(null)} className="shrink-0 hover:text-destructive">
      <X size={14} />
    </button>
  </div>
)}
// Reset button:
<Button variant="outline" size="icon" disabled={!inputBytes} onClick={reset} title="Reset">
  <RotateCcw size={16} />
</Button>
```

- [ ] **Step 7: Verify dev server compiles + Run button works**

Prerequisite: `pkg/pixel_game_kit.js` exists. `cd web && npm run dev` → no compile errors; upload → Run → result; **change k_colors → Run again works** (was blocked before).

- [ ] **Step 8: Commit**

```bash
git add web/src/App.tsx web/src/lib/image-meta.ts web/src/components/UploadZone.tsx
git commit -m "fix(phase6): wire wasm-loader singleton, allow re-process, drop main-thread init, global paste"
```

---

### Task 3: `CandidateGrid` — click wires to config + process (Fix C), de-any (Fix E)

Clicking a candidate must set `detect_strategy` (+ `pixel_size_override` if Uniform) and trigger re-process — the one auto-reprocess exception.

**Files:**
- Modify: `web/src/components/CandidateGrid.tsx`

- [ ] **Step 1: Accept `onProcess` prop + wire click to config**

```tsx
import { useStore } from "@/store";
import { cn } from "@/lib/utils";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import type { Candidate } from "@/wasm/wasm-loader";

export default function CandidateGrid({ onProcess }: { onProcess: () => void }) {
  const candidates = useStore((s) => s.candidates);
  const selectedCandidate = useStore((s) => s.selectedCandidate);
  const selectCandidate = useStore((s) => s.selectCandidate);
  const setConfig = useStore((s) => s.setConfig);

  if (!candidates || candidates.length === 0) return null;

  const pick = (c: Candidate, i: number) => {
    const isSelected = selectedCandidate === i;
    selectCandidate(isSelected ? null : i);
    if (isSelected) return; // deselect only
    setConfig({
      detect_strategy: c.detector.toLowerCase(),
      pixel_size_override: c.cut_method === "Uniform" && c.scale ? c.scale : useStore.getState().config.pixel_size_override,
    });
    onProcess(); // auto-reprocess — the one exception
  };

  const top3 = candidates.slice(0, 3);

  return (
    <div className="space-y-2">
      <h2 className="text-sm font-semibold text-foreground tracking-wide uppercase">Candidates</h2>
      <div className="grid grid-cols-3 gap-2">
        {top3.map((c: Candidate, i: number) => {
          const score = c.confidence ? (c.confidence * 100).toFixed(0) : "?";
          const isSelected = selectedCandidate === i;
          return (
            <Card
              key={i}
              className={cn("cursor-pointer transition-all hover:ring-1 hover:ring-primary/50",
                isSelected ? "ring-2 ring-primary bg-primary/5" : "ring-1 ring-border")}
              onClick={() => pick(c, i)}
            >
              <CardContent className="p-2 space-y-1">
                <div className="flex items-center justify-between">
                  <span className="text-xs font-medium text-foreground">#{i + 1}</span>
                  <Badge variant={isSelected ? "default" : "secondary"} className="text-[10px] px-1.5 py-0">{score}%</Badge>
                </div>
                <div className="text-[10px] text-muted-foreground font-mono leading-tight">
                  <div>{c.detector ?? "?"} · step {c.step ?? "?"}</div>
                  <div>scale {c.scale ?? "?"} · {c.cut_method ?? "?"}</div>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Pass `onProcess` from `App.tsx`**

In `App.tsx`, change `<CandidateGrid />` to `<CandidateGrid onProcess={handleProcess} />`.

- [ ] **Step 3: Verify — click a candidate changes the result**

`npm run dev` → upload → Run → click candidate #2 → detect strategy updates (visible in form) → result re-renders differently.

- [ ] **Step 4: Commit**

```bash
git add web/src/components/CandidateGrid.tsx web/src/App.tsx
git commit -m "fix(phase6): candidate click sets detect strategy + triggers re-process (U2.2)"
```

---

### Task 4: `CompareView` — `useMemo` the input URL (Fix D)

Stop creating a new object URL every render.

**Files:**
- Modify: `web/src/components/CompareView.tsx`

- [ ] **Step 1: Memoize the input URL, revoke on unmount**

```tsx
import { useEffect, useMemo } from "react";
import { useStore } from "@/store";
import { ReactCompareSlider } from "react-compare-slider";
import { Card } from "@/components/ui/card";

export default function CompareView() {
  const inputBytes = useStore((s) => s.inputBytes);
  const result = useStore((s) => s.result);

  const inputUrl = useMemo(
    () => (inputBytes ? URL.createObjectURL(new Blob([inputBytes], { type: "image/png" })) : null),
    [inputBytes]
  );
  useEffect(() => {
    if (inputUrl) return () => URL.revokeObjectURL(inputUrl);
  }, [inputUrl]);

  if (!inputBytes || !result || !inputUrl) return null;

  return (
    <div className="space-y-2">
      <h2 className="text-sm font-semibold text-foreground tracking-wide uppercase">Before &amp; After</h2>
      <Card className="overflow-hidden">
        <ReactCompareSlider
          itemOne={<img src={inputUrl} alt="Original" className="w-full h-full object-contain pixelated" />}
          itemTwo={<img src={result.url} alt="Result" className="w-full h-full object-contain pixelated" />}
          style={{ height: 400 }}
        />
      </Card>
    </div>
  );
}
```

- [ ] **Step 2: Verify no per-render URL growth**

`npm run dev` → DevTools Memory → take snapshot, change a form field 5× (re-renders), snapshot again → no Blob-URL growth.

- [ ] **Step 3: Commit**

```bash
git add web/src/components/CompareView.tsx
git commit -m "fix(phase6): memoize CompareView input URL (no per-render leak)"
```

---

### Task 5: Form/Header polish (Fix G rest)

Collapse postprocess, disable liveValidate, Header 4-state.

**Files:**
- Modify: `web/src/forms/pipeline-uiSchema.ts`, `web/src/components/ConfigForm.tsx`, `web/src/components/Header.tsx`

- [ ] **Step 1: Collapse postprocess in uiSchema — `web/src/forms/pipeline-uiSchema.ts`**

Add to the `postprocess` object in the uiSchema:
```ts
  postprocess: {
    "ui:collapsible": true,
    "ui:collapsed": true,
    "ui:title": "Postprocess（高级）",
    // ...existing field widgets
    bg_tolerance: { "ui:widget": "range" },
  },
```

- [ ] **Step 2: Disable liveValidate — `web/src/components/ConfigForm.tsx`**

Change the `<Form>` prop from `liveValidate` (shorthand true) to `liveValidate={false}`.

- [ ] **Step 3: Header shows all 4 states — `web/src/components/Header.tsx`**

```tsx
import { useStore } from "@/store";

const STATE = {
  loading_wasm: { label: "加载 WASM…", color: "text-muted-foreground" },
  ready: { label: "就绪", color: "text-primary" },
  processing: { label: "处理中…", color: "text-primary animate-pulse" },
  error: { label: "出错", color: "text-destructive" },
};

export default function Header() {
  const status = useStore((s) => s.status);
  const s = STATE[status];
  return (
    <header className="flex items-center justify-between px-6 py-3 border-b border-border">
      <h1 className="text-lg font-semibold">Pixel Game Kit</h1>
      <span className={`text-xs font-mono ${s.color}`}>● {s.label}</span>
    </header>
  );
}
```

- [ ] **Step 4: Verify form is usable**

`npm run dev` → ConfigForm shows core params; "Postprocess" section collapsed (click to expand); typing in a field doesn't spam validation errors.

- [ ] **Step 5: Commit**

```bash
git add web/src/forms/pipeline-uiSchema.ts web/src/components/ConfigForm.tsx web/src/components/Header.tsx
git commit -m "fix(phase6): collapse postprocess, disable liveValidate, Header 4-state"
```

---

### Task 6: Full verification + docs

End-to-end gate + document the cleanup.

**Files:**
- Modify: `CLAUDE.md` (Phase 6 note), `PLAN.md` (Phase 6 实施记录)

- [ ] **Step 1: Run all Vitest**

`cd web && npx vitest run` → adapter (5) + store (3) all pass.

- [ ] **Step 2: Build**

Prerequisite: `pkg/pixel_game_kit.js` exists. `cd web && npm run build` → tsc + vite build clean. Note the bundle size (`ls -lh web/dist/assets/`).

- [ ] **Step 3: Rust + wasm still green**

`cargo test` → 68 pass. `cargo build --target wasm32-unknown-unknown` → 0 warnings.

- [ ] **Step 4: Manual e2e (the 7-step)**

`cd web && npm run dev`, then verify:
1. Upload → metadata + candidates populate.
2. Run → slider compare + Summary.
3. **Change k_colors → Run again** ✓ (was blocked).
4. **Click candidate → strategy updates → result changes** ✓.
5. **Run 5× rapidly** → no UI freeze, no per-click WASM-reload lag.
6. Download → valid PNG.
7. DevTools Memory across 5 runs → no Blob-URL growth.

- [ ] **Step 5: Docs**

In `PLAN.md` Phase 6, append a 实施记录 note: "Phase 6 MVP landed (commits …) with review-found bugs; cleanup (commit …) fixed 3 CRITICAL + 5 HIGH (worker singleton wiring, re-process, candidate→config, URL leaks, types, postprocess collapse, …). Manual e2e + Vitest green."

In `CLAUDE.md`, if Phase 6 isn't noted yet, add a one-liner under the pipeline section: "Web app at `web/` (Phase 6 MVP) — React + Vite + shadcn + RJSF; `cd web && npm run dev`."

- [ ] **Step 6: Commit docs**

```bash
git add CLAUDE.md PLAN.md
git commit -m "docs(phase6): cleanup record + web app note"
```

---

## Self-Review (run after writing — already applied)

**1. Spec coverage:**
- Fix A (wire wasm-loader) → Task 2 Step 1 ✓
- Fix B (re-process) → Task 2 Step 3 ✓
- Fix C (candidate click) → Task 3 ✓
- Fix D (URL leaks — store + CompareView) → Task 1 Step 3c + Task 4 ✓
- Fix E (Candidate type) → Task 1 Step 3a-b + Task 3 ✓
- Fix F (drop main-thread init + default ready) → Task 1 Step 3d + Task 2 Step 2 ✓
- Fix G (postprocess collapse / liveValidate / Lucide / global paste / Header / error dismissible) → Task 2 Steps 4-6 + Task 5 ✓

**2. Placeholder scan:** none — each fix has concrete current→target code. (Task 6 Step 5 doc text is templated; executor fills commit hashes.)

**3. Type consistency:** `Candidate` imported from `wasm-loader` into `store` (Task 1) and `CandidateGrid` (Task 3) — single source. `processInWorker`/`detectInWorker` signatures match wasm-loader.ts. `onProcess: () => void` prop consistent between App (Task 2/3) and CandidateGrid (Task 3). `analyzeInputMeta` extracted to lib (Task 2 Step 5) used by both App + UploadZone.

**Highest-risk task:** Task 2 (App rework) — largest diff, touches the core process flow. Verify with e2e steps 3 + 5 especially.
