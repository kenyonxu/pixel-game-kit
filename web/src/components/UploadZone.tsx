import { useCallback, useRef, useState } from "react";
import { Upload } from "lucide-react";
import { useStore } from "@/store";
import { cn } from "@/lib/utils";

function analyzeInput(
  bytes: Uint8Array,
  mime: string
): Promise<{ w: number; h: number; colors: number; hasAlpha: boolean }> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => {
      const canvas = document.createElement("canvas");
      canvas.width = img.width;
      canvas.height = img.height;
      const ctx = canvas.getContext("2d", { willReadFrequently: true });
      if (!ctx) return reject(new Error("Canvas context unavailable"));
      ctx.drawImage(img, 0, 0);
      const { data } = ctx.getImageData(0, 0, img.width, img.height);
      const set = new Set<number>();
      let hasAlpha = false;
      for (let i = 0; i < data.length; i += 4) {
        if (data[i + 3]! < 255) hasAlpha = true;
        if (data[i + 3]! === 0) continue;
        set.add((data[i]! << 16) | (data[i + 1]! << 8) | data[i + 2]!);
      }
      URL.revokeObjectURL(img.src);
      resolve({ w: img.width, h: img.height, colors: set.size, hasAlpha });
    };
    img.onerror = () => reject(new Error("Failed to decode image"));
    img.src = URL.createObjectURL(new Blob([bytes], { type: mime }));
  });
}

export default function UploadZone() {
  const [dragging, setDragging] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const setImage = useStore((s) => s.setImage);
  const setStatus = useStore((s) => s.setStatus);
  const inputMeta = useStore((s) => s.inputMeta);
  const setError = useStore((s) => s.setError);

  const loadFile = useCallback(
    async (file: File) => {
      const bytes = new Uint8Array(await file.arrayBuffer());
      setStatus("processing");
      try {
        const meta = await analyzeInput(bytes, file.type);
        setImage(bytes, meta);
        setStatus("ready");
      } catch (err) {
        setError(String((err as Error)?.message ?? err));
      }
    },
    [setImage, setStatus, setError]
  );

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setDragging(false);
      const f = e.dataTransfer.files[0];
      if (f && f.type.startsWith("image/")) loadFile(f);
    },
    [loadFile]
  );

  const handlePaste = useCallback(
    (e: React.ClipboardEvent) => {
      const item = Array.from(e.clipboardData.items).find((it) =>
        it.type.startsWith("image/")
      );
      if (!item) return;
      const file = item.getAsFile();
      if (file) loadFile(file);
    },
    [loadFile]
  );

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const f = e.target.files?.[0];
      if (f) loadFile(f);
    },
    [loadFile]
  );

  return (
    <div
      onDragOver={(e) => (e.preventDefault(), setDragging(true))}
      onDragLeave={() => setDragging(false)}
      onDrop={handleDrop}
      onPaste={handlePaste}
      tabIndex={0}
      className={cn(
        "relative flex flex-col items-center justify-center gap-3 rounded-lg border-2 border-dashed p-8 transition-colors",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
        dragging
          ? "border-primary bg-primary/5"
          : "border-border hover:border-muted-foreground/50",
        inputMeta ? "py-4" : "py-12"
      )}
    >
      <input
        ref={inputRef}
        type="file"
        accept="image/png,image/jpeg"
        className="hidden"
        onChange={handleChange}
      />

      {inputMeta ? (
        <div className="flex items-center gap-4 w-full">
          <button
            onClick={() => inputRef.current?.click()}
            className="text-sm text-primary underline-offset-4 hover:underline shrink-0"
          >
            Replace
          </button>
          <p className="text-sm text-muted-foreground font-mono">
            {inputMeta.w}&times;{inputMeta.h} &middot; {inputMeta.colors}{" "}
            colours{inputMeta.hasAlpha ? " · alpha" : ""}
          </p>
        </div>
      ) : (
        <>
          <Upload className="w-8 h-8 text-muted-foreground" />
          <p className="text-sm text-muted-foreground">
            Drop an image, paste, or{" "}
            <button
              onClick={() => inputRef.current?.click()}
              className="text-primary underline-offset-4 hover:underline"
            >
              browse
            </button>
          </p>
          <p className="text-xs text-muted-foreground/60">PNG or JPEG</p>
        </>
      )}
    </div>
  );
}
