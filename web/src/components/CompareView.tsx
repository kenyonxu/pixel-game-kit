import { useEffect, useMemo } from "react";
import { useStore } from "@/store";
import { ReactCompareSlider } from "react-compare-slider";
import { Card } from "@/components/ui/card";
import { bytesToObjectUrl } from "@/lib/blob";

export default function CompareView() {
  const inputBytes = useStore((s) => s.inputBytes);
  const result = useStore((s) => s.result);

  // Memoize the input URL (was created every render → leak). Revoke on change/unmount.
  const inputUrl = useMemo(
    () => (inputBytes ? bytesToObjectUrl(inputBytes, "image/png") : null),
    [inputBytes]
  );
  useEffect(() => {
    if (!inputUrl) return;
    return () => URL.revokeObjectURL(inputUrl);
  }, [inputUrl]);

  if (!inputBytes || !result || !inputUrl) return null;

  return (
    <div className="space-y-2">
      <h2 className="text-sm font-semibold text-foreground tracking-wide uppercase">
        Before &amp; After
      </h2>
      <Card className="overflow-hidden">
        <ReactCompareSlider
          itemOne={
            <img
              src={inputUrl}
              alt="Original"
              className="w-full h-full object-contain pixelated"
            />
          }
          itemTwo={
            <img
              src={result.url}
              alt="Result"
              className="w-full h-full object-contain pixelated"
            />
          }
          style={{ height: 400 }}
        />
      </Card>
    </div>
  );
}
