import { describe, it, expect, vi, beforeEach } from "vitest";
import { useStore } from "../store";

describe("store", () => {
  beforeEach(() => useStore.getState().reset());

  it("setResult revokes the previous result URL", () => {
    const revoke = vi.spyOn(URL, "revokeObjectURL");
    useStore.setState({ inputBytes: new Uint8Array([1]) });
    useStore.getState().setResult({
      bytes: new Uint8Array([1]),
      url: "blob:first",
      elapsedMs: 1,
      outW: 2,
      outH: 2,
    });
    useStore.getState().setResult({
      bytes: new Uint8Array([2]),
      url: "blob:second",
      elapsedMs: 2,
      outW: 2,
      outH: 2,
    });
    expect(revoke).toHaveBeenCalledWith("blob:first");
    expect(useStore.getState().result?.url).toBe("blob:second");
    revoke.mockRestore();
  });

  it("setConfig patches config immutably", () => {
    useStore.getState().setConfig({ k_colors: 32 });
    expect(useStore.getState().config.k_colors).toBe(32);
    expect(useStore.getState().config.colorspace).toBe("oklab"); // unchanged
  });

  it("default status is ready (worker lazy-inits)", () => {
    useStore.getState().reset();
    expect(useStore.getState().status).toBe("ready");
  });
});
