import { create } from "zustand";
import type { PipelineConfig } from "./wasm/adapter";
import { DEFAULT_CONFIG } from "./wasm/adapter";
import type { Candidate } from "./wasm/wasm-loader";

export type Status = "loading_wasm" | "ready" | "processing" | "error";

export interface InputMeta {
  w: number;
  h: number;
  colors: number;
  hasAlpha: boolean;
}

export interface Result {
  bytes: Uint8Array;
  url: string;
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
  candidates: Candidate[];
  selectedCandidate: number | null;
  // actions
  setStatus: (s: Status) => void;
  setError: (e: string | null) => void;
  setImage: (bytes: Uint8Array, meta: InputMeta) => void;
  setConfig: (patch: Partial<PipelineConfig>) => void;
  setResult: (r: Result) => void;
  setCandidates: (c: Candidate[]) => void;
  selectCandidate: (i: number | null) => void;
  reset: () => void;
}

export const useStore = create<State>((set) => ({
  status: "ready",
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
  setResult: (result) =>
    set((s) => {
      if (s.result?.url) URL.revokeObjectURL(s.result.url);
      return { result, status: "ready" };
    }),
  setCandidates: (candidates) => set({ candidates }),
  selectCandidate: (selectedCandidate) => set({ selectedCandidate }),
  reset: () =>
    set((s) => {
      if (s.result?.url) URL.revokeObjectURL(s.result.url);
      return { inputBytes: null, inputMeta: null, result: null, candidates: [], selectedCandidate: null, error: null, status: "ready" };
    }),
}));
