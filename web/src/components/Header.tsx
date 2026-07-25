import { useStore } from "@/store";

export default function Header() {
  const status = useStore((s) => s.status);
  const isProcessing = status === "processing";

  return (
    <header className="flex items-center justify-between px-6 py-3 border-b border-border">
      <h1 className="text-lg font-semibold text-foreground tracking-tight">
        Pixel Game Kit
      </h1>
      <div className="flex items-center gap-3">
        {isProcessing && (
          <span className="inline-flex items-center gap-2 text-sm text-muted-foreground">
            <span className="w-2 h-2 rounded-full bg-primary animate-pulse" />
            Processing…
          </span>
        )}
        <span className="text-xs text-muted-foreground">
          all processing runs locally in your browser
        </span>
      </div>
    </header>
  );
}
