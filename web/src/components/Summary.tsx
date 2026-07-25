import { useStore } from "@/store";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Download } from "lucide-react";

export default function Summary() {
  const result = useStore((s) => s.result);
  const inputMeta = useStore((s) => s.inputMeta);

  if (!result || !inputMeta) return null;

  const handleDownload = () => {
    const a = document.createElement("a");
    a.href = result.url;
    a.download = "pixel-snapped.png";
    a.click();
  };

  return (
    <Card className="p-4 space-y-3">
      <div className="flex items-center justify-between">
        <h2 className="text-sm font-semibold text-foreground tracking-wide uppercase">
          Result
        </h2>
        <Button size="sm" variant="outline" onClick={handleDownload}>
          <Download className="w-3.5 h-3.5 mr-1.5" />
          Download
        </Button>
      </div>
      <div className="grid grid-cols-3 gap-3 text-xs">
        <div>
          <span className="text-muted-foreground">Output</span>
          <p className="font-mono text-foreground">
            {result.outW}&times;{result.outH}
          </p>
        </div>
        <div>
          <span className="text-muted-foreground">Input</span>
          <p className="font-mono text-foreground">
            {inputMeta.w}&times;{inputMeta.h}
          </p>
        </div>
        <div>
          <span className="text-muted-foreground">Time</span>
          <p className="font-mono text-foreground">
            {result.elapsedMs.toFixed(0)} ms
          </p>
        </div>
      </div>
    </Card>
  );
}
