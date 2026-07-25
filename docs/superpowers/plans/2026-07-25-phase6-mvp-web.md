# Phase 6 MVP — Web App Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the Phase 6 MVP web app at `web/` — a React/shadcn tool that does upload → candidate-grid pick → tune params → process (Web Worker, non-blocking) → slider compare → download, replacing the broken root `index.html`.

**Architecture:** Vite + React + TS + shadcn/ui + zustand + RJSF, themed from [design-system/pixel-game-kit/MASTER.md](../../../design-system/pixel-game-kit/MASTER.md). Web Worker wraps the WASM `process_image` / `detect_candidates` (non-blocking). A single `schema/pipeline-config.schema.json` is the source of truth RJSF renders. An `adapter.ts` maps the form's PipelineConfig JSON → the WASM `process_image` 10-positional-arg + `post_config` signature.

**Tech Stack:** Vite, React 18, TypeScript, Tailwind + shadcn/ui, zustand, @rjsf/core v5 + @rjsf/validator-ajv8, react-compare-slider, lucide-react, vite-plugin-wasm + vite-plugin-top-level-await, Vitest (logic tests). Prerequisite: `wasm-pack build --target web --out-dir pkg --release` produces `pkg/pixel_game_kit.js`.

**Spec:** [docs/superpowers/specs/2026-07-25-phase6-mvp-web-design.md](../specs/2026-07-25-phase6-mvp-web-design.md)
**Design brief:** [design-system/pixel-game-kit/MASTER.md](../../../design-system/pixel-game-kit/MASTER.md)

---

## File Structure

**Create:**
- `schema/pipeline-config.schema.json` — JSON Schema (single source of truth)
- `web/package.json`, `web/vite.config.ts`, `web/tsconfig.json`, `web/index.html`, `web/components.json` (shadcn)
- `web/tailwind.config.ts`, `web/postcss.config.js`, `web/src/globals.css` — MASTER.md theme
- `web/src/main.tsx`, `web/src/App.tsx`
- `web/src/store.ts` — zustand
- `web/src/wasm/wasm-loader.ts`, `web/src/wasm/worker.ts`, `web/src/wasm/adapter.ts`
- `web/src/forms/pipeline-uiSchema.ts`
- `web/src/components/{Header,UploadZone,ConfigForm,CandidateGrid,CompareView,Summary}.tsx`
- `web/src/vite-env.d.ts`
- `web/src/__tests__/adapter.test.ts` — Vitest

**Delete:**
- `index.html` (root — broken, imports pre-rename pkg name)

**Frontend testing reality:** React components are verified by dev-server render + manual e2e (not pure unit tests). Pure logic (the `adapter`, store reducers) gets Vitest unit tests. The deterministic core is the Rust WASM (already 68 tests). Each task ends with a concrete verification (dev server / Vitest / e2e check).

---

### Task 1: `schema/pipeline-config.schema.json`

The single source of truth. RJSF renders it; the adapter maps it to WASM; 6B/6H reuse it.

**Files:**
- Create: `schema/pipeline-config.schema.json`

- [ ] **Step 1: Write the schema**

`schema/pipeline-config.schema.json`:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "PipelineConfig",
  "type": "object",
  "properties": {
    "k_colors": { "type": "integer", "minimum": 1, "default": 16, "title": "Colors", "description": "k-means palette size" },
    "pixel_size_override": { "type": ["number", "null"], "minimum": 1, "default": null, "title": "Pixel size override", "description": "Empty = auto-detect. Range [1, min(w,h)/2]" },
    "palette": { "type": "array", "items": { "type": "string", "pattern": "^[0-9a-fA-F]{6}$" }, "default": [], "title": "Custom palette", "description": "Hex colors, e.g. 0d2b45" },
    "detect_strategy": { "type": "string", "enum": ["auto", "runs", "tiled", "elastic"], "default": "auto", "title": "Detect" },
    "resample_method": { "type": "string", "enum": ["majority", "median", "dominant", "mode", "qvote"], "default": "majority", "title": "Resample" },
    "colorspace": { "type": "string", "enum": ["oklab", "rgb"], "default": "oklab", "title": "Colorspace" },
    "dither": { "type": "string", "enum": ["none", "fs", "bayer2", "bayer4", "bayer8", "ordered"], "default": "none", "title": "Dither" },
    "preset_palette": { "type": "string", "enum": ["none", "nes", "gameboy", "sgb", "snes", "pc9801", "msx1", "pico8", "sweetie16", "endesga32"], "default": "none", "title": "Preset palette" },
    "postprocess": {
      "type": "object",
      "title": "Postprocess",
      "properties": {
        "bg_remove": { "type": "boolean", "default": false, "title": "Background removal" },
        "bg_tolerance": { "type": "integer", "minimum": 0, "maximum": 255, "default": 64 },
        "bg_connectivity": { "type": "string", "enum": ["4", "8"], "default": "4" },
        "bg_scope": { "type": "string", "enum": ["outer", "all"], "default": "outer" },
        "bg_floating_threshold": { "type": "integer", "minimum": 0, "default": 0 },
        "outline": { "type": "string", "enum": ["none", "rounded", "sharp"], "default": "none" },
        "outline_color": { "type": "string", "pattern": "^[0-9a-fA-F]{6}$", "default": "000000" },
        "morph": { "type": "boolean", "default": false },
        "alpha_threshold": { "type": ["string", "null"], "default": null, "title": "Alpha threshold", "description": "Empty=off, 'auto'=Otsu, or 0-255" }
      },
      "default": {}
    }
  }
}
```

- [ ] **Step 2: Validate it parses**

Run: `node -e "JSON.parse(require('fs').readFileSync('schema/pipeline-config.schema.json','utf8')); console.log('schema OK')"` (or `python -c "import json;json.load(open('schema/pipeline-config.schema.json'));print('OK')"`).
Expected: `schema OK` (valid JSON).

- [ ] **Step 3: Commit**

```bash
git add schema/pipeline-config.schema.json
git commit -m "feat(phase6): pipeline-config JSON schema (single source of truth)"
```

---

### Task 2: `web/` scaffold + deps + MASTER.md theme

Vite React-TS, Tailwind + shadcn, theme from MASTER.md, vite-plugin-wasm + alias. No app logic yet — just a "Hello" page rendering with the dark theme + WASM loading confirmation.

**Files:**
- Create: `web/` (scaffold) + `web/tailwind.config.ts`, `web/postcss.config.js`, `web/src/globals.css`, `web/vite.config.ts`

- [ ] **Step 1: Scaffold Vite React-TS in `web/`**

From repo root:
```bash
npm create vite@latest web -- --template react-ts
cd web
npm install
npm install zustand @rjsf/core @rjsf/utils @rjsf/validator-ajv8 react-compare-slider lucide-react
npm install -D tailwindcss postcss autoprefixer vite-plugin-wasm vite-plugin-top-level-await vitest
```

- [ ] **Step 2: Tailwind + shadcn init**

```bash
npx tailwindcss init -p
# shadcn init (interactive: pick "dark" base; or non-interactive flags per shadcn version)
npx shadcn@latest init -d   # -d uses defaults; adjust CSS vars manually in step 4
```

- [ ] **Step 3: Apply MASTER.md theme — `web/src/globals.css`**

Replace `web/src/globals.css` with shadcn CSS variables mapped from MASTER.md (dark slate + green accent):

```css
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap');
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --background: 222 47% 11%;        /* #0F172A slate-950 */
    --foreground: 210 40% 98%;        /* #F8FAFC */
    --card: 217 33% 17%;              /* #1E293B slate-800 */
    --card-foreground: 210 40% 98%;
    --popover: 217 33% 17%;
    --popover-foreground: 210 40% 98%;
    --primary: 142 71% 45%;           /* #22C55E green-500 — "run green" */
    --primary-foreground: 222 47% 11%;
    --secondary: 215 25% 27%;         /* #334155 slate-700 */
    --secondary-foreground: 210 40% 98%;
    --muted: 222 24% 20%;             /* #272F42 */
    --muted-foreground: 215 20% 65%;
    --accent: 142 71% 45%;
    --accent-foreground: 222 47% 11%;
    --destructive: 0 84% 60%;         /* #EF4444 */
    --destructive-foreground: 210 40% 98%;
    --border: 215 25% 27%;            /* #334155 hairline */
    --input: 222 47% 11%;             /* dark inputs, NOT light */
    --ring: 142 71% 45%;              /* green ring (visible on slate) */
    --radius: 0.5rem;
  }
}

html, body, #root { height: 100%; }
body {
  font-family: 'Inter', system-ui, sans-serif;
  background: hsl(var(--background));
  color: hsl(var(--foreground));
}
/* MASTER.md hard rule: pixel art never blurs */
img.pixelated, canvas.pixelated {
  image-rendering: pixelated;
  image-rendering: crisp-edges;
}
```

- [ ] **Step 4: Tailwind config — `web/tailwind.config.ts`**

```ts
import type { Config } from "tailwindcss";
export default {
  darkMode: "class",
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: { extend: {
    fontFamily: { sans: ['Inter', 'system-ui', 'sans-serif'],
                  mono: ['ui-monospace', 'JetBrains Mono', 'monospace'] },
    colors: {
      background: "hsl(var(--background))", foreground: "hsl(var(--foreground))",
      card: { DEFAULT: "hsl(var(--card))", foreground: "hsl(var(--card-foreground))" },
      primary: { DEFAULT: "hsl(var(--primary))", foreground: "hsl(var(--primary-foreground))" },
      secondary: { DEFAULT: "hsl(var(--secondary))", foreground: "hsl(var(--secondary-foreground))" },
      muted: { DEFAULT: "hsl(var(--muted))", foreground: "hsl(var(--muted-foreground))" },
      border: "hsl(var(--border))", input: "hsl(var(--input))", ring: "hsl(var(--ring))",
      destructive: { DEFAULT: "hsl(var(--destructive))", foreground: "hsl(var(--destructive-foreground))" },
    },
    borderRadius: { lg: "var(--radius)", md: "calc(var(--radius) - 2px)", sm: "calc(var(--radius) - 4px)" },
  }},
  plugins: [],
} satisfies Config;
```

- [ ] **Step 5: Vite config — wasm plugin + `@pkg` alias — `web/vite.config.ts`**

```ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import path from "path";

export default defineConfig({
  plugins: [react(), wasm(), topLevelAwait()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "@pkg": path.resolve(__dirname, "../pkg"),
    },
  },
  worker: { format: "es", plugins: () => [wasm(), topLevelAwait()] },
  test: { environment: "jsdom" }, // vitest
});
```

- [ ] **Step 6: Placeholder `App.tsx` confirms theme + WASM path**

`web/src/App.tsx`:
```tsx
export default function App() {
  return (
    <div className="h-full flex items-center justify-center">
      <h1 className="text-2xl font-semibold text-primary">Pixel Game Kit</h1>
    </div>
  );
}
```

`web/src/main.tsx`:
```tsx
import "./globals.css";
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
ReactDOM.createRoot(document.getElementById("root")!).render(<App />);
```

`web/index.html` (replace the Vite default `<script>` already points to /src/main.tsx — just set `<html class="dark">` and `<title>Pixel Game Kit</title>`, lang `zh-CN` or `en`).

- [ ] **Step 7: Verify dev server + theme**

```bash
cd web && npm run dev
```
Expected: serves at `localhost:5173`; green "Pixel Game Kit" heading on dark slate bg (MASTER.md theme applied). `Ctrl+C` to stop.

- [ ] **Step 8: Commit**

```bash
git add web/
git commit -m "feat(phase6): web/ scaffold + MASTER.md dark theme + vite wasm config"
```

---

### Task 3: WASM loader + Worker + adapter

The trickiest wiring (WASM in Vite + module worker). Adapter is unit-tested.

**Files:**
- Create: `web/src/wasm/adapter.ts`, `web/src/wasm/worker.ts`, `web/src/wasm/wasm-loader.ts`, `web/src/vite-env.d.ts`
- Test: `web/src/__tests__/adapter.test.ts`

- [ ] **Step 1: Adapter (pure logic) — `web/src/wasm/adapter.ts`**

```ts
export interface PipelineConfig {
  k_colors: number;
  pixel_size_override: number | null;
  palette: string[];
  detect_strategy: string;
  resample_method: string;
  colorspace: string;
  dither: string;
  preset_palette: string;
  postprocess: Record<string, unknown>;
}

export const DEFAULT_CONFIG: PipelineConfig = {
  k_colors: 16,
  pixel_size_override: null,
  palette: [],
  detect_strategy: "auto",
  resample_method: "majority",
  colorspace: "oklab",
  dither: "none",
  preset_palette: "none",
  postprocess: {},
};

/** Map form config -> process_image(bytes, ...positional[8], post_config). */
export function configToWasm(config: PipelineConfig): { positional: unknown[]; post_config: string } {
  const paletteHex = config.palette && config.palette.length ? config.palette.join(",") : null;
  const positional: unknown[] = [
    config.k_colors,
    config.pixel_size_override,
    paletteHex,
    config.detect_strategy,
    config.resample_method,
    config.colorspace,
    config.dither,
    config.preset_palette,
  ];
  return { positional, post_config: JSON.stringify(config.postprocess ?? {}) };
}
```

- [ ] **Step 2: Adapter unit test — `web/src/__tests__/adapter.test.ts`**

```ts
import { describe, it, expect } from "vitest";
import { configToWasm, DEFAULT_CONFIG } from "../wasm/adapter";

describe("configToWasm", () => {
  it("maps default config to null palette + empty post_config", () => {
    const { positional, post_config } = configToWasm(DEFAULT_CONFIG);
    expect(positional).toEqual([16, null, null, "auto", "majority", "oklab", "none", "none"]);
    expect(post_config).toBe("{}");
  });

  it("joins palette array to comma hex", () => {
    const out = configToWasm({ ...DEFAULT_CONFIG, palette: ["0d2b45", "ffecd6"] });
    expect(out.positional[2]).toBe("0d2b45,ffecd6");
  });

  it("empty palette array -> null (not empty string)", () => {
    const out = configToWasm({ ...DEFAULT_CONFIG, palette: [] });
    expect(out.positional[2]).toBeNull();
  });

  it("serializes postprocess object to JSON", () => {
    const out = configToWasm({ ...DEFAULT_CONFIG, postprocess: { bg_remove: true, outline: "sharp" } });
    expect(JSON.parse(out.post_config)).toEqual({ bg_remove: true, outline: "sharp" });
  });

  it("is deterministic", () => {
    expect(configToWasm(DEFAULT_CONFIG)).toEqual(configToWasm(DEFAULT_CONFIG));
  });
});
```

- [ ] **Step 3: Run adapter tests**

```bash
cd web && npx vitest run src/__tests__/adapter.test.ts
```
Expected: 5 pass.

- [ ] **Step 4: Worker — `web/src/wasm/worker.ts`**

```ts
/// <reference lib="webworker" />
import init, { process_image, detect_candidates } from "@pkg/pixel_game_kit.js";

let ready: Promise<void> | null = null;
const ensure = () => (ready ??= init());

type MsgIn =
  | { type: "process"; bytes: Uint8Array; positional: unknown[]; post_config: string }
  | { type: "detect"; bytes: Uint8Array; k_colors: number; detect_strategy: string | null };

self.onmessage = async (e: MessageEvent<MsgIn>) => {
  try {
    await ensure();
    const data = e.data;
    if (data.type === "process") {
      const t0 = performance.now();
      const result = process_image(data.bytes, ...data.positional, data.post_config);
      (self as unknown as Worker).postMessage({
        type: "process_done",
        resultBytes: result,
        elapsedMs: performance.now() - t0,
      });
    } else if (data.type === "detect") {
      const json = detect_candidates(data.bytes, data.k_colors, data.detect_strategy);
      (self as unknown as Worker).postMessage({ type: "detect_done", candidates: JSON.parse(json) });
    }
  } catch (err) {
    (self as unknown as Worker).postMessage({
      type: "error",
      error: String((err as Error)?.message ?? err),
    });
  }
};
```

- [ ] **Step 5: Worker client — `web/src/wasm/wasm-loader.ts`**

```ts
import PipelineWorker from "./worker?worker"; // Vite worker import

export interface Candidate {
  detector: string;
  scale: number | null;
  step: number;
  confidence: number;
  cut_method: string;
}

let worker: Worker | null = null;
export function getWorker(): Worker {
  if (!worker) worker = new PipelineWorker();
  return worker;
}

export function processInWorker(
  bytes: Uint8Array,
  positional: unknown[],
  post_config: string
): Promise<{ resultBytes: Uint8Array; elapsedMs: number }> {
  return new Promise((resolve, reject) => {
    const w = getWorker();
    const onMsg = (e: MessageEvent) => {
      if (e.data.type === "process_done") {
        w.removeEventListener("message", onMsg);
        resolve({ resultBytes: e.data.resultBytes, elapsedMs: e.data.elapsedMs });
      } else if (e.data.type === "error") {
        w.removeEventListener("message", onMsg);
        reject(new Error(e.data.error));
      }
    };
    w.addEventListener("message", onMsg);
    w.postMessage({ type: "process", bytes, positional, post_config }, [bytes.buffer]);
  });
}

export function detectInWorker(
  bytes: Uint8Array,
  k_colors: number,
  detect_strategy: string | null
): Promise<Candidate[]> {
  return new Promise((resolve, reject) => {
    const w = getWorker();
    const onMsg = (e: MessageEvent) => {
      if (e.data.type === "detect_done") {
        w.removeEventListener("message", onMsg);
        resolve(e.data.candidates as Candidate[]);
      } else if (e.data.type === "error") {
        w.removeEventListener("message", onMsg);
        reject(new Error(e.data.error));
      }
    };
    w.addEventListener("message", onMsg);
    w.postMessage({ type: "detect", bytes, k_colors, detect_strategy });
  });
}
```

- [ ] **Step 6: TS env — `web/src/vite-env.d.ts`**

```ts
/// <reference types="vite/client" />
```

- [ ] **Step 7: Verify WASM loads in the worker (manual smoke)**

Temporarily wire `App.tsx` to call `processInWorker` on a button with a test PNG (or skip this smoke and rely on Task 10's e2e). Minimal: confirm `npm run dev` builds without "cannot resolve @pkg/pixel_game_kit.js" — i.e., `wasm-pack build` must have run (prerequisite). If pkg/ is missing: `wasm-pack build --target web --out-dir pkg --release` from repo root first.

Expected: dev server compiles, no module-resolution errors.

- [ ] **Step 8: Commit**

```bash
git add web/src/wasm/ web/src/__tests__/ web/src/vite-env.d.ts
git commit -m "feat(phase6): WASM worker + adapter (configToWasm), adapter unit-tested"
```

---

### Task 4: zustand store

Single source of UI state. Holds input, config, result, candidates, status.

**Files:**
- Create: `web/src/store.ts`

- [ ] **Step 1: Write `web/src/store.ts`**

```ts
import { create } from "zustand";
import { PipelineConfig, DEFAULT_CONFIG } from "./wasm/adapter";

export type Status = "loading_wasm" | "ready" | "processing" | "error";

export interface InputMeta {
  w: number;
  h: number;
  colors: number;
  hasAlpha: boolean;
}

export interface Result {
  bytes: Uint8Array;
  url: string; // object URL
  elapsedMs: number;
  outW: number;
  outH: number;
}

interface State {
  status: Status;
  error: string | null;
  inputBytes: Uint8Array | null;
  inputMeta: InputMeta | null;
  config: PipelineConfig;
  result: Result | null;
  candidates: ReturnType<typeof Object> | any[];
  selectedCandidate: number | null;
  // actions
  setStatus: (s: Status) => void;
  setError: (e: string | null) => void;
  setImage: (bytes: Uint8Array, meta: InputMeta) => void;
  setConfig: (patch: Partial<PipelineConfig>) => void;
  setResult: (r: Result) => void;
  setCandidates: (c: any[]) => void;
  selectCandidate: (i: number | null) => void;
  reset: () => void;
}

export const useStore = create<State>((set) => ({
  status: "loading_wasm",
  error: null,
  inputBytes: null,
  inputMeta: null,
  config: DEFAULT_CONFIG,
  result: null,
  candidates: [],
  selectedCandidate: null,
  setStatus: (status) => set({ status }),
  setError: (error) => set({ error, status: error ? "error" : "ready" }),
  setImage: (inputBytes, inputMeta) => set({ inputBytes, inputMeta, result: null, candidates: [], selectedCandidate: null }),
  setConfig: (patch) => set((s) => ({ config: { ...s.config, ...patch } })),
  setResult: (result) => set({ result, status: "ready" }),
  setCandidates: (candidates) => set({ candidates }),
  selectCandidate: (selectedCandidate) => set({ selectedCandidate }),
  reset: () => set({ inputBytes: null, inputMeta: null, result: null, candidates: [], selectedCandidate: null, error: null, status: "ready" }),
}));
```

- [ ] **Step 2: Verify it typechecks**

```bash
cd web && npx tsc --noEmit
```
Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add web/src/store.ts
git commit -m "feat(phase6): zustand store (input/config/result/candidates/status)"
```

---

### Task 5: `<UploadZone>` + input metadata

Drag/paste/select + `analyzeInput` (dims + unique opaque color count + alpha flag). Wires to store; triggers candidate detection.

**Files:**
- Create: `web/src/components/UploadZone.tsx`

- [ ] **Step 1: Write `web/src/components/UploadZone.tsx`**

```tsx
import { useCallback, useRef } from "react";
import { Upload } from "lucide-react";
import { useStore, InputMeta } from "../store";
import { detectInWorker } from "../wasm/wasm-loader";

async function analyzeInput(bytes: Uint8Array, type: string): Promise<InputMeta> {
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

export default function UploadZone() {
  const { inputBytes, inputMeta, setImage, setCandidates, setStatus, setError } = useStore();
  const inputRef = useRef<HTMLInputElement>(null);

  const handleBytes = useCallback(async (bytes: Uint8Array, type: string) => {
    try {
      const meta = await analyzeInput(bytes, type);
      setImage(bytes, meta);
      const cands = await detectInWorker(bytes, useStore.getState().config.k_colors, null);
      setCandidates(cands.slice(0, 3));
    } catch (e) {
      setError(String((e as Error).message ?? e));
    }
  }, [setImage, setCandidates, setError]);

  const onFile = (f: File) => f.arrayBuffer().then((b) => handleBytes(new Uint8Array(b), f.type));

  return (
    <div
      className="border-2 border-dashed border-border rounded-lg p-6 text-center cursor-pointer hover:border-primary transition-colors"
      onDragOver={(e) => e.preventDefault()}
      onDrop={(e) => { e.preventDefault(); const f = e.dataTransfer.files[0]; if (f) onFile(f); }}
      onClick={() => inputRef.current?.click()}
    >
      <Upload className="mx-auto mb-2 text-muted-foreground" size={24} />
      <p className="text-sm">拖拽 / 粘贴 / 点击上传 PNG·JPG</p>
      <input
        ref={inputRef}
        type="file"
        accept="image/png,image/jpeg"
        className="hidden"
        onChange={(e) => { const f = e.target.files?.[0]; if (f) onFile(f); }}
      />
      {inputMeta && (
        <p className="mt-2 text-xs font-mono text-muted-foreground">
          {inputMeta.w}×{inputMeta.h} · {inputMeta.colors} 色{inputMeta.hasAlpha ? " · 含 alpha" : ""}
        </p>
      )}
      {inputBytes && <p className="mt-1 text-xs text-primary">已加载</p>}
    </div>
  );
}
```

Add paste handler at the `App` level in Task 10 (`window.addEventListener("paste", ...)`).

- [ ] **Step 2: Verify dev server renders it (temporarily mount in App)**

Temporarily replace `App.tsx` body with `<UploadZone />` import; `npm run dev`; confirm the drop zone shows. Revert App to the Task-2 placeholder after (Task 10 assembles for real).

- [ ] **Step 3: Commit**

```bash
git add web/src/components/UploadZone.tsx
git commit -m "feat(phase6): UploadZone (drag/paste/select) + input metadata + candidate detect"
```

---

### Task 6: `<ConfigForm>` (RJSF) + uiSchema

RJSF bound to the schema, uiSchema groups core params visible + postprocess collapsed. MVP uses RJSF default widgets.

**Files:**
- Create: `web/src/forms/pipeline-uiSchema.ts`, `web/src/components/ConfigForm.tsx`

- [ ] **Step 1: uiSchema — `web/src/forms/pipeline-uiSchema.ts`**

```ts
export const pipelineUiSchema = {
  "ui:order": [
    "k_colors", "pixel_size_override", "palette", "detect_strategy",
    "colorspace", "dither", "preset_palette", "resample_method", "postprocess",
  ],
  k_colors: { "ui:widget": "updown", "ui:title": "Colors" },
  pixel_size_override: { "ui:placeholder": "auto", "ui:title": "Pixel size" },
  palette: { "ui:help": "逗号分隔的 6 位 hex，留空=自动" },
  postprocess: {
    "ui:collapsible": true,
    "ui:collapsed": true,
    "ui:title": "Postprocess（高级）",
    bg_tolerance: { "ui:widget": "range" },
  },
};
```

- [ ] **Step 2: ConfigForm — `web/src/components/ConfigForm.tsx`**

```tsx
import Form from "@rjsf/core";
import validator from "@rjsf/validator-ajv8";
import { useStore } from "../store";
import { pipelineUiSchema } from "../forms/pipeline-uiSchema";
import schema from "../../../../schema/pipeline-config.schema.json";

export default function ConfigForm() {
  const { config, setConfig } = useStore();
  return (
    <Form
      schema={schema as any}
      uiSchema={pipelineUiSchema as any}
      validator={validator}
      formData={config}
      onChange={(e) => setConfig(e.formData)}
      liveValidate={false}
      showErrorList={false}
    >
      <div /> {/* hides default submit button */}
    </Form>
  );
}
```

(Note: the schema import path `../../../../schema/...` is from `web/src/components/`; configure a `@schema` alias in vite.config if preferred.)

- [ ] **Step 3: Verify it renders**

Temporarily mount `<ConfigForm />` in App; `npm run dev`; confirm the form renders with the schema fields, postprocess collapsed. Revert App.

- [ ] **Step 4: Commit**

```bash
git add web/src/components/ConfigForm.tsx web/src/forms/pipeline-uiSchema.ts
git commit -m "feat(phase6): ConfigForm (RJSF + schema + uiSchema, default widgets)"
```

---

### Task 7: `<CandidateGrid>`

Top-3 candidates from `detect_candidates`, pixelated thumbnails, click-to-select (sets `detect_strategy` + `pixel_size_override` if Uniform).

**Files:**
- Create: `web/src/components/CandidateGrid.tsx`

- [ ] **Step 1: Write `web/src/components/CandidateGrid.tsx`**

```tsx
import { useStore } from "../store";

export default function CandidateGrid() {
  const { candidates, selectedCandidate, selectCandidate, setConfig, inputBytes, config } = useStore();
  if (!inputBytes || candidates.length === 0) return null;

  const pick = (i: number) => {
    const c = candidates[i];
    selectCandidate(i);
    setConfig({
      detect_strategy: c.detector.toLowerCase(),
      pixel_size_override: c.cut_method === "Uniform" && c.scale ? c.scale : config.pixel_size_override,
    });
  };

  return (
    <div>
      <h3 className="text-sm font-semibold mb-2">候选网格</h3>
      <div className="grid grid-cols-3 gap-2">
        {candidates.map((c, i) => (
          <button
            key={i}
            onClick={() => pick(i)}
            className={`border rounded p-2 text-left hover:border-primary transition-colors ${
              selectedCandidate === i ? "border-primary bg-secondary" : "border-border"
            }`}
          >
            <img
              src={URL.createObjectURL(new Blob([inputBytes], { type: "image/png" }))}
              alt={`候选 ${i + 1}`}
              className="pixelated w-full h-20 object-contain bg-background mb-1"
            />
            <p className="text-xs font-mono">{c.detector} · {c.step.toFixed(1)}px</p>
            <p className="text-xs text-muted-foreground">置信度 {(c.confidence * 100).toFixed(0)}%</p>
          </button>
        ))}
      </div>
    </div>
  );
}
```

(Note: thumbnail shows the input, not a per-candidate preview — generating per-candidate previews is post-MVP. For MVP, the thumbnail is the input + the candidate's metadata. Acceptable per spec U2.2 "top-3 + confidence"; a true per-candidate rendered preview is a polish item.)

- [ ] **Step 2: Commit**

```bash
git add web/src/components/CandidateGrid.tsx
git commit -m "feat(phase6): CandidateGrid (top-3, click-to-select detector)"
```

---

### Task 8: `<CompareView>`

react-compare-slider, pixelated original vs result.

**Files:**
- Create: `web/src/components/CompareView.tsx`

- [ ] **Step 1: Write `web/src/components/CompareView.tsx`**

```tsx
import { CompareSlider, CompareSliderHandle } from "react-compare-slider";
import { useStore } from "../store";

function PixelImg({ src, alt }: { src: string; alt: string }) {
  return <img src={src} alt={alt} className="pixelated max-w-full max-h-full object-contain" />;
}

export default function CompareView() {
  const { inputBytes, result, status } = useStore();
  if (!inputBytes) {
    return <div className="flex items-center justify-center h-full text-muted-foreground text-sm">上传图片后显示</div>;
  }
  if (!result) {
    return <div className="flex items-center justify-center h-full text-muted-foreground text-sm">{status === "processing" ? "处理中…" : "点「处理」生成结果"}</div>;
  }
  const origUrl = URL.createObjectURL(new Blob([inputBytes], { type: "image/png" }));
  return (
    <CompareSlider
      one={<PixelImg src={origUrl} alt="原图" />}
      two={<PixelImg src={result.url} alt="结果" />}
      handle={<CompareSliderHandle className="bg-primary" />}
      className="w-full h-full bg-background"
    />
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add web/src/components/CompareView.tsx
git commit -m "feat(phase6): CompareView (react-compare-slider, pixelated)"
```

---

### Task 9: `<Summary>` + `<Header>`

Summary shows output dims + color count + elapsed; candidate grid shows detector/step/confidence. Header shows app title + WASM status.

**Files:**
- Create: `web/src/components/Summary.tsx`, `web/src/components/Header.tsx`

- [ ] **Step 1: `<Summary>` — `web/src/components/Summary.tsx`**

```tsx
import { useEffect, useState } from "react";
import { useStore } from "../store";

export default function Summary() {
  const { result, candidates, selectedCandidate } = useStore();
  const [outColors, setOutColors] = useState<number | null>(null);

  useEffect(() => {
    if (!result) return setOutColors(null);
    createImageBitmap(new Blob([result.bytes], { type: "image/png" })).then(async (bmp) => {
      const c = document.createElement("canvas");
      c.width = bmp.width; c.height = bmp.height;
      const ctx = c.getContext("2d")!; ctx.drawImage(bmp, 0, 0); bmp.close();
      const { data } = ctx.getImageData(0, 0, c.width, c.height);
      const set = new Set<number>();
      for (let i = 0; i < data.length; i += 4) if (data[i + 3] !== 0) set.add((data[i] << 16) | (data[i + 1] << 8) | data[i + 2]);
      setOutColors(set.size);
      result.outW = c.width; result.outH = c.height;
    });
  }, [result]);

  if (!result) return null;
  const sel = selectedCandidate != null ? candidates[selectedCandidate] : null;
  return (
    <div className="text-xs font-mono text-muted-foreground flex flex-wrap gap-x-4 gap-y-1">
      <span>输出 {result.outW}×{result.outH}</span>
      {outColors != null && <span>{outColors} 色</span>}
      <span>{result.elapsedMs.toFixed(0)} ms</span>
      {sel && <span className="text-primary">{sel.detector} · step {sel.step.toFixed(1)}</span>}
    </div>
  );
}
```

- [ ] **Step 2: `<Header>` — `web/src/components/Header.tsx`**

```tsx
import { useStore } from "../store";

export default function Header() {
  const status = useStore((s) => s.status);
  const label = { loading_wasm: "加载 WASM 中…", ready: "就绪", processing: "处理中…", error: "出错" }[status];
  const color = status === "error" ? "text-destructive" : status === "ready" ? "text-primary" : "text-muted-foreground";
  return (
    <header className="flex items-center justify-between px-6 py-3 border-b border-border">
      <h1 className="text-lg font-semibold">Pixel Game Kit</h1>
      <span className={`text-xs font-mono ${color}`}>● {label}</span>
    </header>
  );
}
```

- [ ] **Step 3: Commit**

```bash
git add web/src/components/Summary.tsx web/src/components/Header.tsx
git commit -m "feat(phase6): Summary (output dims/colors/time) + Header (status)"
```

---

### Task 10: `<App>` assembly + process/download flow + delete old index.html

Compose all into the two-pane workspace (MASTER.md layout), wire the process button to the worker, download, paste handler, WASM-ready status, and delete the broken root `index.html`.

**Files:**
- Modify: `web/src/App.tsx`
- Delete: `index.html` (root)

- [ ] **Step 1: Write `web/src/App.tsx` (full assembly)**

```tsx
import { useEffect } from "react";
import { Button } from "@/components/ui/button"; // shadcn
import { Loader2, Download } from "lucide-react";
import { useStore } from "./store";
import { configToWasm } from "./wasm/adapter";
import { processInWorker } from "./wasm/wasm-loader";
import Header from "./components/Header";
import UploadZone from "./components/UploadZone";
import ConfigForm from "./components/ConfigForm";
import CandidateGrid from "./components/CandidateGrid";
import CompareView from "./components/CompareView";
import Summary from "./components/Summary";

export default function App() {
  const { status, inputBytes, config, result, setStatus, setResult, setError } = useStore();

  // Mark ready once mounted (worker lazy-inits on first use; status reflects that)
  useEffect(() => {
    useStore.getState().setStatus("ready");
    // paste handler
    const onPaste = (e: ClipboardEvent) => {
      const item = [...(e.clipboardData?.items ?? [])].find((i) => i.type.startsWith("image/"));
      item?.getAsFile()?.arrayBuffer().then((b) =>
        useStore.getState().setImage(new Uint8Array(b), { w: 0, h: 0, colors: 0, hasAlpha: false })
      );
    };
    window.addEventListener("paste", onPaste);
    return () => window.removeEventListener("paste", onPaste);
  }, []);

  const onProcess = async () => {
    if (!inputBytes) return;
    setStatus("processing");
    try {
      const { positional, post_config } = configToWasm(config);
      const bytes = inputBytes.slice(); // copy (transfer detaches original)
      const { resultBytes, elapsedMs } = await processInWorker(bytes, positional, post_config);
      const url = URL.createObjectURL(new Blob([resultBytes], { type: "image/png" }));
      setResult({ bytes: resultBytes, url, elapsedMs, outW: 0, outH: 0 });
    } catch (e) {
      setError(String((e as Error).message ?? e));
    }
  };

  const onDownload = () => {
    if (!result) return;
    const a = document.createElement("a");
    a.href = result.url; a.download = "pixel-game-kit.png"; a.click();
  };

  return (
    <div className="h-full flex flex-col">
      <Header />
      <div className="flex-1 flex flex-col lg:flex-row min-h-0">
        {/* Left controls */}
        <aside className="lg:w-[340px] lg:border-r border-border p-4 overflow-y-auto space-y-4">
          <UploadZone />
          <ConfigForm />
          <CandidateGrid />
          <div className="flex gap-2">
            <Button onClick={onProcess} disabled={!inputBytes || status === "processing"} className="flex-1">
              {status === "processing" ? <><Loader2 className="animate-spin" size={16} /> 处理中</> : "处理"}
            </Button>
            <Button variant="secondary" onClick={onDownload} disabled={!result}>
              <Download size={16} /> 下载
            </Button>
          </div>
        </aside>
        {/* Right canvas */}
        <main className="flex-1 p-4 flex flex-col gap-2 min-h-0">
          <div className="flex-1 min-h-0 rounded-lg overflow-hidden border border-border bg-card">
            <CompareView />
          </div>
          <Summary />
        </main>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Add shadcn Button component**

```bash
cd web && npx shadcn@latest add button
```

- [ ] **Step 3: Delete the broken root `index.html`**

```bash
git rm index.html
```

- [ ] **Step 4: Full e2e verification**

Prerequisite: `wasm-pack build --target web --out-dir pkg --release` (from repo root) is done.

```bash
cd web && npm run dev
```
Manually verify:
1. App loads, dark theme, status "就绪".
2. Upload an AI sprite (e.g. copy `tests/fixtures/baseline/ai-sprite.png` in) → metadata shows → candidate grid populates.
3. Click 处理 → result appears in slider compare (drag slider, both sides pixelated) → Summary shows dims/colors/time.
4. Click a candidate → detect strategy updates → 处理 again reflects it.
5. Adjust k_colors → 处理 → different result.
6. Download → saves a valid PNG.
7. **Non-blocking (U12.5)**: while processing a large image, the form/sliders stay interactive (no freeze).

Expected: all 7 pass.

- [ ] **Step 5: Bundle size check**

```bash
cd web && npm run build && ls -lh dist/assets/ | head
```
Expected: total gzipped assets ideally < 250KB (PLAN target). If RJSF pushes it over, note it — lazy-load `<CandidateGrid>` / `<CompareView>` as a follow-up (don't block MVP).

- [ ] **Step 6: Commit**

```bash
git add web/src/App.tsx web/src/components/ui/
git commit -m "feat(phase6): App assembly (two-pane workspace) + process/download + delete broken index.html"
```

---

## Self-Review (run after writing — already applied)

**1. Spec coverage:**
- 6A scaffold (Vite/React/TS/shadcn/WASM loader/Worker) → Task 2 + 3 ✓
- schema (single source of truth) → Task 1 ✓
- RJSF default form → Task 6 ✓
- Upload U1.3 (drag/paste/select) + metadata U1.5 → Task 5 (+ paste in Task 10) ✓
- Candidate grid U2.2 → Task 7 ✓
- Slider compare U7.1 → Task 8 ✓
- Summary U7.3 → Task 9 ✓
- Download U8.1 → Task 10 ✓
- Worker non-blocking U12.5 → Task 3 (worker) + Task 10 e2e check #7 ✓
- adapter (schema→WASM) → Task 3 ✓
- Replace broken index.html → Task 10 Step 3 ✓
- MASTER.md theme → Task 2 Step 3-4 ✓

**2. Placeholder scan:** no TBDs. Schema complete. Adapter + worker + store + components have full code. (Per-candidate rendered thumbnails noted as post-MVP polish in Task 7 — MVP shows input thumbnail + candidate metadata, which satisfies U2.2's "top-3 + confidence".)

**3. Type consistency:** `PipelineConfig` (adapter.ts) is the single type referenced by store (Task 4), ConfigForm (Task 6), CandidateGrid (Task 7), App (Task 10). Worker message types (`process`/`detect`/`*_done`/`error`) consistent between worker.ts and wasm-loader.ts. `configToWasm` return shape (`{positional, post_config}`) consistent across adapter test, worker call, App.onProcess.

**Known risks called out in plan:**
- WASM-in-Vite + module worker is the highest-risk wiring (Task 3) — if `@pkg` alias or worker wasm loading fails, that's the blocker to debug first.
- CandidateGrid thumbnail is the input image, not a per-candidate preview (polish deferred).
- Bundle size with RJSF may approach 250KB — monitor, lazy-load if needed.
