import { useStore } from "@/store";
import { ReactCompareSlider } from "react-compare-slider";
import { Card } from "@/components/ui/card";

export default function CompareView() {
  const inputBytes = useStore((s) => s.inputBytes);
  const result = useStore((s) => s.result);

  if (!inputBytes || !result) return null;

  const inputUrl = URL.createObjectURL(
    new Blob([inputBytes], { type: "image/png" })
  );

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
              onLoad={() => URL.revokeObjectURL(inputUrl)}
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
