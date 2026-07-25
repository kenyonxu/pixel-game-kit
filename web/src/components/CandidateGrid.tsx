import { useStore } from "@/store";
import { cn } from "@/lib/utils";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import type { Candidate } from "@/wasm/wasm-loader";

/** Clicking a candidate sets detect_strategy (+ pixel_size_override if Uniform)
 *  and triggers re-process — the one auto-reprocess exception (U2.2). */
export default function CandidateGrid({ onProcess }: { onProcess: () => void }) {
  const candidates = useStore((s) => s.candidates);
  const selectedCandidate = useStore((s) => s.selectedCandidate);
  const selectCandidate = useStore((s) => s.selectCandidate);
  const setConfig = useStore((s) => s.setConfig);

  if (!candidates || candidates.length === 0) return null;

  const pick = (c: Candidate, i: number) => {
    const isSelected = selectedCandidate === i;
    selectCandidate(isSelected ? null : i);
    if (isSelected) return; // deselect only — no reprocess
    setConfig({
      detect_strategy: c.detector.toLowerCase(),
      pixel_size_override:
        c.cut_method === "Uniform" && c.scale
          ? c.scale
          : useStore.getState().config.pixel_size_override,
    });
    onProcess();
  };

  const top3 = candidates.slice(0, 3);

  return (
    <div className="space-y-2">
      <h2 className="text-sm font-semibold text-foreground tracking-wide uppercase">
        Candidates
      </h2>
      <div className="grid grid-cols-3 gap-2">
        {top3.map((c: Candidate, i: number) => {
          const score = c.confidence ? (c.confidence * 100).toFixed(0) : "?";
          const isSelected = selectedCandidate === i;
          return (
            <Card
              key={i}
              className={cn(
                "cursor-pointer transition-all hover:ring-1 hover:ring-primary/50",
                isSelected ? "ring-2 ring-primary bg-primary/5" : "ring-1 ring-border"
              )}
              onClick={() => pick(c, i)}
            >
              <CardContent className="p-2 space-y-1">
                <div className="flex items-center justify-between">
                  <span className="text-xs font-medium text-foreground">#{i + 1}</span>
                  <Badge variant={isSelected ? "default" : "secondary"} className="text-[10px] px-1.5 py-0">
                    {score}%
                  </Badge>
                </div>
                <div className="text-[10px] text-muted-foreground font-mono leading-tight">
                  <div>{c.detector ?? "?"} · step {c.step ?? "?"}</div>
                  <div>scale {c.scale ?? "?"} · {c.cut_method ?? "?"}</div>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}
