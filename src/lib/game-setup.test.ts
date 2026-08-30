import { describe, expect, it } from "vitest";
import {
  engineGuessLabel,
  recommendedStack,
  runtimeCanAutoInstall,
  setupItemStatusLabel,
} from "./game-setup";
import { emptyRuntime } from "./runtime-catalog";
import type { EngineGuess, RuntimeKind, SetupItemStatus } from "./types";

describe("game-setup labels", () => {
  it("nomeia todos os motores detetados", () => {
    const guesses: EngineGuess[] = ["unity", "unreal", "unknown"];
    for (const guess of guesses) {
      expect(engineGuessLabel(guess).length).toBeGreaterThan(0);
    }
  });

  it("nomeia todos os estados do plano", () => {
    const statuses: SetupItemStatus[] = [
      "ready",
      "missing",
      "installable",
      "manual",
    ];
    for (const status of statuses) {
      expect(setupItemStatusLabel(status).length).toBeGreaterThan(0);
    }
  });

  it("só instala automaticamente Wine Staging e WineHQ", () => {
    const kinds: RuntimeKind[] = [
      "gptk",
      "gcenx_staging",
      "winehq",
      "winecx",
      "crossover",
      "custom",
    ];
    expect(kinds.filter(runtimeCanAutoInstall)).toEqual([
      "gcenx_staging",
      "winehq",
    ]);
  });

  it("escolhe GPTK + D3DMetal quando o GPTK está presente", () => {
    const stack = recommendedStack([
      { ...emptyRuntime("gptk"), found: true, wine64Path: "/opt/gptk/bin/wine64" },
      { ...emptyRuntime("winehq"), found: true, wine64Path: "/opt/wine/bin/wine" },
    ]);
    expect(stack).toEqual({ runtimeKind: "gptk", dxBackend: "d3dmetal" });
  });

  it("cai no Wine Staging + DXMT se não houver GPTK", () => {
    const stack = recommendedStack([
      emptyRuntime("gptk"),
      {
        ...emptyRuntime("gcenx_staging"),
        found: true,
        wine64Path: "/opt/homebrew/bin/wine64",
      },
    ]);
    expect(stack).toEqual({
      runtimeKind: "gcenx_staging",
      dxBackend: "dxmt",
    });
  });
});
