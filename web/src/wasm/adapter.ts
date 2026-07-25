export interface PipelineConfig {
  k_colors: number;
  pixel_size_override: number | null;
  palette: string[];
  detect_strategy: string;
  resample_method: string;
  colorspace: string;
  dither: string;
  preset_palette: string;
  postprocess: Record<string, unknown>;
}

export const DEFAULT_CONFIG: PipelineConfig = {
  k_colors: 16,
  pixel_size_override: null,
  palette: [],
  detect_strategy: "auto",
  resample_method: "majority",
  colorspace: "oklab",
  dither: "none",
  preset_palette: "none",
  postprocess: {},
};

/** Map form config -> process_image(bytes, ...positional[8], post_config). */
export function configToWasm(config: PipelineConfig): { positional: unknown[]; post_config: string } {
  const paletteHex = config.palette && config.palette.length ? config.palette.join(",") : null;
  const positional: unknown[] = [
    config.k_colors,
    config.pixel_size_override,
    paletteHex,
    config.detect_strategy,
    config.resample_method,
    config.colorspace,
    config.dither,
    config.preset_palette,
  ];
  return { positional, post_config: JSON.stringify(config.postprocess ?? {}) };
}
