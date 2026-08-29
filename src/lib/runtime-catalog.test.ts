import { describe, expect, it } from "vitest";
import { catalogEntry, dxBackendLabel, runtimeKindLabel } from "./runtime-catalog";
import type { RuntimeKind } from "./types";

describe("runtime-catalog", () => {
  it("não redistribui D3DMetal", () => {
    const kinds: RuntimeKind[] = [
      "gptk",
      "gcenx_staging",
      "winecx",
      "crossover",
      "custom",
    ];
    for (const kind of kinds) {
      expect(catalogEntry(kind).redistributesD3dmetal).toBe(false);
      expect(runtimeKindLabel(kind).length).toBeGreaterThan(0);
    }
  });

  it("nomeia backends", () => {
    expect(dxBackendLabel("d3dmetal")).toBe("D3DMetal");
    expect(dxBackendLabel("dxvk")).toBe("DXVK-macOS");
  });
});
