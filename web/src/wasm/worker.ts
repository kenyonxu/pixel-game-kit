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
