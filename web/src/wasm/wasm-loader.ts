import PipelineWorker from "./worker?worker";

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
