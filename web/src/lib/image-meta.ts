import type { InputMeta } from "@/store";
import { bytesToObjectUrl } from "@/lib/blob";

/** Decode image bytes via an Image element, count unique opaque colors, flag alpha. */
export function analyzeInputMeta(bytes: Uint8Array, mime: string): Promise<InputMeta> {
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
    img.src = bytesToObjectUrl(bytes, mime);
  });
}
