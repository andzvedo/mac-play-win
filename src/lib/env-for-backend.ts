import { assertNever } from "./assert-never";
import type { DxBackend } from "./types";

export function envForBackend(
  backend: DxBackend,
  options: { metalHud: boolean },
): Record<string, string> {
  const env: Record<string, string> = {
    MTL_HUD_ENABLED: options.metalHud ? "1" : "0",
  };

  switch (backend) {
    case "d3dmetal":
      env.WINEDLLOVERRIDES = "d3d11,d3d12,dxgi=n,b";
      env.D3DMETAL_ENABLE = "1";
      break;
    case "dxmt":
      env.WINEDLLOVERRIDES = "d3d10,d3d11,dxgi=n,b";
      env.DXMT_CONFIG = "dxmt";
      break;
    case "dxvk":
      env.WINEDLLOVERRIDES = "d3d10core,d3d11,dxgi=n,b";
      env.DXVK_HUD = options.metalHud ? "1" : "0";
      break;
    case "wined3d":
      env.WINEDLLOVERRIDES = "d3d11,d3d12,dxgi=b";
      break;
    default:
      return assertNever(backend);
  }

  return env;
}
