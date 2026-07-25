import { useStore } from "@/store";
import { cn } from "@/lib/utils";

const STATE: Record<string, { label: string; dot: string }> = {
  loading_wasm: { label: "Loading…", dot: "bg-muted-foreground" },
  ready: { label: "Ready", dot: "bg-primary" },
  processing: { label: "Processing…", dot: "bg-primary animate-pulse" },
  error: { label: "Error", dot: "bg-destructive" },
};

export default function Header() {
  const status = useStore((s) => s.status);
  const s = STATE[status] ?? STATE.ready;

  return (
    <header className="flex items-center justify-between px-6 py-3 border-b border-border">
      <h1 className="text-lg font-semibold text-foreground tracking-tight">
        Pixel Game Kit
      </h1>
      <div className="flex items-center gap-3">
        <span className={cn("inline-flex items-center gap-2 text-sm", status === "error" ? "text-destructive" : "text-muted-foreground")}>
          <span className={cn("w-2 h-2 rounded-full", s.dot)} />
          {s.label}
        </span>
        <span className="text-xs text-muted-foreground/70">all processing runs locally</span>
      </div>
    </header>
  );
}
