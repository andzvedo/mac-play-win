import { assertNever } from "./assert-never";
import type { DxBackend } from "./types";

export const VCRUN_DLL_OVERRIDE =
  "vcruntime140,vcruntime140_1,msvcp140,msvcp140_1,msvcp140_2,concrt140,vccorlib140=n,b";

export function withVcrunOverride(overrides: string): string {
  return `${overrides};${VCRUN_DLL_OVERRIDE}`;
}

export function envForBackend(
  backend: DxBackend,
  options: { metalHud: boolean },
): Record<string, string> {
  const env: Record<string, string> = {
    MTL_HUD_ENABLED: options.metalHud ? "1" : "0",
  };

  switch (backend) {
    case "d3dmetal":
      env.WINEDLLOVERRIDES = withVcrunOverride("d3d11,d3d12,dxgi=n,b");
      env.D3DMETAL_ENABLE = "1";
      break;
    case "dxmt":
      env.WINEDLLOVERRIDES = withVcrunOverride("d3d10,d3d11,dxgi=n,b");
      env.DXMT_CONFIG = "dxmt";
      break;
    case "dxvk":
      env.WINEDLLOVERRIDES = withVcrunOverride("d3d10core,d3d11,dxgi=n,b");
      env.DXVK_HUD = options.metalHud ? "1" : "0";
      break;
    case "wined3d":
      env.WINEDLLOVERRIDES = withVcrunOverride("d3d11,d3d12,dxgi=b");
      break;
    default:
      return assertNever(backend);
  }

  return env;
}
