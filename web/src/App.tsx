import { useEffect } from "react";
import { useStore } from "@/store";
import type { InputMeta } from "@/store";
import Header from "@/components/Header";
import UploadZone from "@/components/UploadZone";
import ConfigForm from "@/components/ConfigForm";
import CandidateGrid from "@/components/CandidateGrid";
import CompareView from "@/components/CompareView";
import Summary from "@/components/Summary";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";

declare module "@pkg/pixel_game_kit.js" {
  export default function init(): Promise<void>;
  export function process_image(
    bytes: Uint8Array,
    ...args: unknown[]
  ): Uint8Array;
  export function detect_candidates(
    bytes: Uint8Array,
    kColors: number,
    strategy: string | null
  ): string;
}

export default function App() {
  const status = useStore((s) => s.status);
  const error = useStore((s) => s.error);
  const inputBytes = useStore((s) => s.inputBytes);
  const inputMeta = useStore((s) => s.inputMeta);
  const config = useStore((s) => s.config);
  const setStatus = useStore((s) => s.setStatus);
  const setError = useStore((s) => s.setError);
  const setResult = useStore((s) => s.setResult);
  const setCandidates = useStore((s) => s.setCandidates);
  const setImage = useStore((s) => s.setImage);
  const result = useStore((s) => s.result);
  const reset = useStore((s) => s.reset);

  // Initialize WASM
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const init = (await import("@pkg/pixel_game_kit.js")).default;
        await init();
        if (!cancelled) setStatus("ready");
      } catch (e) {
        if (!cancelled)
          setError(
            "WASM load failed: " + String((e as Error)?.message ?? e)
          );
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [setStatus, setError]);

  const handleProcess = async () => {
    if (!inputBytes || !inputMeta) return;

    setStatus("processing");
    try {
      const adapter = await import("@/wasm/adapter");
      const { configToWasm } = adapter;
      const { positional, post_config } = configToWasm(config);

      const Worker = (
        await import("@/wasm/worker?worker")
      ).default as unknown as new () => Worker;
      const worker = new Worker();

      const resultPromise = new Promise<{
        resultBytes: Uint8Array;
        elapsedMs: number;
      }>((resolve, reject) => {
        worker.onmessage = (e) => {
          const data = e.data;
          if (data.type === "error") reject(new Error(data.error));
          else resolve(data);
        };
      });

      worker.postMessage({
        type: "process",
        bytes: inputBytes,
        positional,
        post_config,
      });

      const { resultBytes, elapsedMs } = await resultPromise;
      worker.terminate();

      const url = URL.createObjectURL(
        new Blob([resultBytes], { type: "image/png" })
      );

      // Decode output dimensions
      const outMeta = await decodeImageMeta(resultBytes);
      setResult({
        bytes: resultBytes,
        url,
        elapsedMs,
        outW: outMeta.w,
        outH: outMeta.h,
      });

      // Also run detection
      handleDetect(inputBytes, config.k_colors, config.detect_strategy);
    } catch (e) {
      setError(String((e as Error)?.message ?? e));
    }
  };

  const handleDetect = async (
    bytes: Uint8Array,
    kColors: number,
    strategy: string
  ) => {
    try {
      const Worker = (
        await import("@/wasm/worker?worker")
      ).default as unknown as new () => Worker;
      const worker = new Worker();

      const detectPromise = new Promise<any[]>((resolve, reject) => {
        worker.onmessage = (e) => {
          if (e.data.type === "error") reject(new Error(e.data.error));
          else resolve(e.data.candidates ?? []);
        };
      });

      worker.postMessage({
        type: "detect",
        bytes,
        k_colors: kColors,
        detect_strategy: strategy === "auto" ? null : strategy,
      });

      const candidates = await detectPromise;
      worker.terminate();
      setCandidates(candidates);
    } catch {
      // Detection is non-critical; silently ignore
    }
  };

  const canProcess = status === "ready" && inputBytes && inputMeta && !result;

  return (
    <div className="h-full flex flex-col">
      <Header />
      <div className="flex-1 flex overflow-hidden">
        {/* Left sidebar */}
        <aside className="w-80 shrink-0 border-r border-border flex flex-col">
          <ScrollArea className="flex-1 p-4">
            <div className="space-y-5">
              <UploadZone />
              <Separator />
              <ConfigForm />
            </div>
          </ScrollArea>
          <div className="p-4 border-t border-border space-y-2">
            {error && (
              <p className="text-xs text-destructive bg-destructive/10 rounded px-2 py-1">
                {error}
              </p>
            )}
            <div className="flex gap-2">
              <Button
                className="flex-1"
                disabled={!canProcess}
                onClick={handleProcess}
              >
                {status === "processing" ? "Processing…" : "Run Pipeline"}
              </Button>
              <Button
                variant="outline"
                size="icon"
                disabled={!inputBytes}
                onClick={reset}
                title="Reset"
              >
                &#x21bb;
              </Button>
            </div>
          </div>
        </aside>

        {/* Main content */}
        <main className="flex-1 flex flex-col overflow-hidden">
          <ScrollArea className="flex-1 p-6">
            {!inputBytes ? (
              <div className="h-full flex items-center justify-center">
                <p className="text-sm text-muted-foreground">
                  Drop an image or paste one to get started
                </p>
              </div>
            ) : (
              <div className="max-w-2xl mx-auto space-y-5">
                <CandidateGrid />
                <CompareView />
                <Summary />
              </div>
            )}
          </ScrollArea>
        </main>
      </div>
    </div>
  );
}

async function decodeImageMeta(
  bytes: Uint8Array
): Promise<{ w: number; h: number }> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => {
      URL.revokeObjectURL(img.src);
      resolve({ w: img.width, h: img.height });
    };
    img.onerror = () => reject(new Error("Failed to decode result image"));
    img.src = URL.createObjectURL(new Blob([bytes], { type: "image/png" }));
  });
}
