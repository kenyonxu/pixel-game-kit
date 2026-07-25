/// <reference types="vite/client" />

declare module "@pkg/pixel_game_kit.js" {
  export default function init(): Promise<void>;
  export function process_image(bytes: Uint8Array, ...args: unknown[]): Uint8Array;
  export function detect_candidates(
    bytes: Uint8Array,
    kColors: number,
    strategy: string | null
  ): string;
}
