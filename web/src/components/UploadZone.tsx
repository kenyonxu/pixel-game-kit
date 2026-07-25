import { useCallback, useRef, useState } from "react";
import { Upload } from "lucide-react";
import { useStore } from "@/store";
import { cn } from "@/lib/utils";
import { analyzeInputMeta } from "@/lib/image-meta";
import { detectInWorker } from "@/wasm/wasm-loader";

/** Load a file: analyze metadata, set image, and run candidate detection.
 *  Shared by UploadZone (drop/browse) and App's global paste handler. */
export async function loadImageFile(file: File): Promise<void> {
  const bytes = new Uint8Array(await file.arrayBuffer());
  const store = useStore.getState();
  store.setStatus("processing");
  try {
    const meta = await analyzeInputMeta(bytes, file.type);
    store.setImage(bytes, meta);
    const cands = await detectInWorker(bytes, store.config.k_colors, null);
    store.setCandidates(cands.slice(0, 3));
    store.setStatus("ready");
  } catch (err) {
    store.setError(String((err as Error)?.message ?? err));
  }
}

export default function UploadZone() {
  const [dragging, setDragging] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const inputMeta = useStore((s) => s.inputMeta);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragging(false);
    const f = e.dataTransfer.files[0];
    if (f && f.type.startsWith("image/")) loadImageFile(f);
  }, []);

  const handleChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const f = e.target.files?.[0];
    if (f) loadImageFile(f);
  }, []);

  return (
    <div
      onDragOver={(e) => (e.preventDefault(), setDragging(true))}
      onDragLeave={() => setDragging(false)}
      onDrop={handleDrop}
      className={cn(
        "relative flex flex-col items-center justify-center gap-3 rounded-lg border-2 border-dashed transition-colors",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
        dragging ? "border-primary bg-primary/5" : "border-border hover:border-muted-foreground/50",
        inputMeta ? "py-4" : "py-12"
      )}
    >
      <input ref={inputRef} type="file" accept="image/png,image/jpeg" className="hidden" onChange={handleChange} />
      {inputMeta ? (
        <div className="flex items-center gap-4 w-full">
          <button onClick={() => inputRef.current?.click()} className="text-sm text-primary underline-offset-4 hover:underline shrink-0">
            Replace
          </button>
          <p className="text-sm text-muted-foreground font-mono">
            {inputMeta.w}&times;{inputMeta.h} &middot; {inputMeta.colors} colours{inputMeta.hasAlpha ? " · alpha" : ""}
          </p>
        </div>
      ) : (
        <>
          <Upload className="w-8 h-8 text-muted-foreground" />
          <p className="text-sm text-muted-foreground">
            Drop an image, paste, or{" "}
            <button onClick={() => inputRef.current?.click()} className="text-primary underline-offset-4 hover:underline">
              browse
            </button>
          </p>
          <p className="text-xs text-muted-foreground/60">PNG or JPEG</p>
        </>
      )}
    </div>
  );
}
