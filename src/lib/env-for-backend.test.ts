import { describe, expect, it } from "vitest";
import { envForBackend } from "./env-for-backend";
import type { DxBackend } from "./types";

describe("envForBackend", () => {
  it("liga o HUD Metal quando pedido", () => {
    const env = envForBackend("d3dmetal", { metalHud: true });
    expect(env.MTL_HUD_ENABLED).toBe("1");
    expect(env.D3DMETAL_ENABLE).toBe("1");
    expect(env.WINEDLLOVERRIDES).toContain("d3d12");
  });

  it("usa builtin WineD3D e força o Visual C++ nativo", () => {
    const env = envForBackend("wined3d", { metalHud: false });
    expect(env.MTL_HUD_ENABLED).toBe("0");
    expect(env.WINEDLLOVERRIDES).toBe(
      "d3d11,d3d12,dxgi=b;vcruntime140,vcruntime140_1,msvcp140,msvcp140_1,msvcp140_2,concrt140,vccorlib140=n,b",
    );
  });

  it("cobre todos os backends sem lançar", () => {
    const backends: DxBackend[] = ["d3dmetal", "dxmt", "dxvk", "wined3d"];
    for (const backend of backends) {
      const env = envForBackend(backend, { metalHud: false });
      expect(env.WINEDLLOVERRIDES).toContain("vcruntime140");
    }
  });
});
