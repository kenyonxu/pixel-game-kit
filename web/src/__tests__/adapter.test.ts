import { describe, it, expect } from "vitest";
import { configToWasm, DEFAULT_CONFIG } from "../wasm/adapter";

describe("configToWasm", () => {
  it("maps default config to null palette + empty post_config", () => {
    const { positional, post_config } = configToWasm(DEFAULT_CONFIG);
    expect(positional).toEqual([16, null, null, "auto", "majority", "oklab", "none", "none"]);
    expect(post_config).toBe("{}");
  });

  it("joins palette array to comma hex", () => {
    const out = configToWasm({ ...DEFAULT_CONFIG, palette: ["0d2b45", "ffecd6"] });
    expect(out.positional[2]).toBe("0d2b45,ffecd6");
  });

  it("empty palette array -> null (not empty string)", () => {
    const out = configToWasm({ ...DEFAULT_CONFIG, palette: [] });
    expect(out.positional[2]).toBeNull();
  });

  it("serializes postprocess object to JSON", () => {
    const out = configToWasm({ ...DEFAULT_CONFIG, postprocess: { bg_remove: true, outline: "sharp" } });
    expect(JSON.parse(out.post_config)).toEqual({ bg_remove: true, outline: "sharp" });
  });

  it("is deterministic", () => {
    expect(configToWasm(DEFAULT_CONFIG)).toEqual(configToWasm(DEFAULT_CONFIG));
  });
});
