import { useEffect } from "react";
import { useStore } from "@/store";
import Header from "@/components/Header";
import UploadZone from "@/components/UploadZone";
import { loadImageFile } from "@/components/UploadZone";
import ConfigForm from "@/components/ConfigForm";
import CandidateGrid from "@/components/CandidateGrid";
import CompareView from "@/components/CompareView";
import Summary from "@/components/Summary";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { RotateCcw, X } from "lucide-react";
import { configToWasm } from "@/wasm/adapter";
import { processInWorker } from "@/wasm/wasm-loader";
import { bytesToObjectUrl } from "@/lib/blob";

export default function App() {
  const status = useStore((s) => s.status);
  const error = useStore((s) => s.error);
  const inputBytes = useStore((s) => s.inputBytes);
  const inputMeta = useStore((s) => s.inputMeta);
  const setStatus = useStore((s) => s.setStatus);
  const setError = useStore((s) => s.setError);
  const setResult = useStore((s) => s.setResult);
  const reset = useStore((s) => s.reset);

  // Global paste (works anywhere, not just when UploadZone is focused).
  useEffect(() => {
    const onPaste = (e: ClipboardEvent) => {
      const item = [...(e.clipboardData?.items ?? [])].find((i) => i.type.startsWith("image/"));
      const file = item?.getAsFile();
      if (file) loadImageFile(file);
    };
    window.addEventListener("paste", onPaste);
    return () => window.removeEventListener("paste", onPaste);
  }, []);

  const handleProcess = async () => {
    // Read fresh state at call time (no stale closure).
    const { inputBytes, config } = useStore.getState();
    if (!inputBytes) return;
    setStatus("processing");
    try {
      const { positional, post_config } = configToWasm(config);
      const bytes = inputBytes.slice(); // copy — transfer detaches the original
      const { resultBytes, elapsedMs } = await processInWorker(bytes, positional, post_config);
      const url = bytesToObjectUrl(resultBytes, "image/png");
      const outMeta = await decodeImageMeta(resultBytes);
      setResult({ bytes: resultBytes, url, elapsedMs, outW: outMeta.w, outH: outMeta.h });
    } catch (e) {
      setError(String((e as Error)?.message ?? e));
    }
  };

  const canProcess = !!inputBytes && !!inputMeta && status !== "processing";

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
              <ErrorBoundary label="Config form">
                <ConfigForm />
              </ErrorBoundary>
            </div>
          </ScrollArea>
          <div className="p-4 border-t border-border space-y-2">
            {error && (
              <div className="text-xs text-destructive bg-destructive/10 rounded px-2 py-1 flex items-center justify-between gap-2">
                <span className="truncate">{error}</span>
                <button
                  onClick={() => setError(null)}
                  className="shrink-0 hover:text-destructive cursor-pointer"
                  title="Dismiss"
                >
                  <X size={14} />
                </button>
              </div>
            )}
            <div className="flex gap-2">
              <Button className="flex-1" disabled={!canProcess} onClick={handleProcess}>
                {status === "processing" ? "Processing…" : "Run Pipeline"}
              </Button>
              <Button variant="outline" size="icon" disabled={!inputBytes} onClick={reset} title="Reset">
                <RotateCcw size={16} />
              </Button>
            </div>
          </div>
        </aside>

        {/* Main content */}
        <main className="flex-1 flex flex-col overflow-hidden">
          <ScrollArea className="flex-1 p-6">
            {!inputBytes ? (
              <div className="h-full flex items-center justify-center">
                <p className="text-sm text-muted-foreground">Drop an image or paste one to get started</p>
              </div>
            ) : (
              <div className="max-w-2xl mx-auto space-y-5">
                <ErrorBoundary label="Candidate grid">
                  <CandidateGrid onProcess={handleProcess} />
                </ErrorBoundary>
                <ErrorBoundary label="Compare view">
                  <CompareView />
                </ErrorBoundary>
                <ErrorBoundary label="Summary">
                  <Summary />
                </ErrorBoundary>
              </div>
            )}
          </ScrollArea>
        </main>
      </div>
    </div>
  );
}

async function decodeImageMeta(bytes: Uint8Array): Promise<{ w: number; h: number }> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => {
      URL.revokeObjectURL(img.src);
      resolve({ w: img.width, h: img.height });
    };
    img.onerror = () => reject(new Error("Failed to decode result image"));
    img.src = bytesToObjectUrl(bytes, "image/png");
  });
}
