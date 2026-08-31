import { describe, expect, it } from "vitest";
import {
  engineGuessLabel,
  pendingInstallLabels,
  recommendedStack,
  runtimeCanAutoInstall,
  shouldAutoPrepare,
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

  it("só instala automaticamente Wine Staging gerenciado", () => {
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
    ]);
  });

  it("prefere Wine Staging + DXVK quando o Staging está presente", () => {
    const stack = recommendedStack([
      { ...emptyRuntime("gptk"), found: true, wine64Path: "/opt/gptk/bin/wine64" },
      { ...emptyRuntime("gcenx_staging"), found: true, wine64Path: "/opt/staging/bin/wine" },
      { ...emptyRuntime("winehq"), found: true, wine64Path: "/opt/wine/bin/wine" },
    ]);
    expect(stack).toEqual({ runtimeKind: "gcenx_staging", dxBackend: "dxvk" });
  });

  it("usa GPTK + D3DMetal quando não há Wine Staging", () => {
    const stack = recommendedStack([
      { ...emptyRuntime("gptk"), found: true, wine64Path: "/opt/gptk/bin/wine64" },
      emptyRuntime("gcenx_staging"),
    ]);
    expect(stack).toEqual({ runtimeKind: "gptk", dxBackend: "d3dmetal" });
  });

  it("instala automaticamente enquanto houver itens em falta", () => {
    expect(shouldAutoPrepare(true, "game:exe", null)).toBe(true);
    expect(shouldAutoPrepare(true, "game:exe", "game:exe")).toBe(false);
    expect(shouldAutoPrepare(false, "game:exe", null)).toBe(false);
    expect(shouldAutoPrepare(true, "b:exe", null, "a", "b")).toBe(false);
    expect(shouldAutoPrepare(true, "b:exe", null, "b", "b")).toBe(true);
  });

  it("lista o que ainda falta instalar", () => {
    expect(
      pendingInstallLabels([
        {
          id: "runtime",
          label: "Wine",
          status: "ready",
          detail: "",
        },
        {
          id: "prefix",
          label: "Prefixo Wine",
          status: "installable",
          detail: "",
        },
        {
          id: "vcrun",
          label: "Visual C++",
          status: "missing",
          detail: "",
        },
      ]),
    ).toEqual(["Prefixo Wine", "Visual C++"]);
  });
});
