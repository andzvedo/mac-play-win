import { assertNever } from "./assert-never";
import type { DxBackend, Runtime, RuntimeKind } from "./types";

export interface RuntimeCatalogEntry {
  kind: RuntimeKind;
  label: string;
  supportedBackends: DxBackend[];
  defaultBackend: DxBackend;
  installUrl: string | null;
  redistributesD3dmetal: false;
}

export const RUNTIME_CATALOG: readonly RuntimeCatalogEntry[] = [
  {
    kind: "gptk",
    label: "Game Porting Toolkit",
    supportedBackends: ["d3dmetal", "wined3d"],
    defaultBackend: "d3dmetal",
    installUrl: "https://developer.apple.com/games/game-porting-toolkit/",
    redistributesD3dmetal: false,
  },
  {
    kind: "gcenx_staging",
    label: "Wine Staging (Gcenx)",
    supportedBackends: ["dxmt", "dxvk", "wined3d"],
    defaultBackend: "dxmt",
    installUrl: "https://github.com/Gcenx/macOS_Wine_builds",
    redistributesD3dmetal: false,
  },
  {
    kind: "winecx",
    label: "Wine CX (Gcenx)",
    supportedBackends: ["dxvk", "dxmt", "wined3d"],
    defaultBackend: "dxvk",
    installUrl: "https://github.com/Gcenx/winecx",
    redistributesD3dmetal: false,
  },
  {
    kind: "crossover",
    label: "CrossOver",
    supportedBackends: ["d3dmetal", "dxmt", "dxvk", "wined3d"],
    defaultBackend: "d3dmetal",
    installUrl: "https://www.codeweavers.com/crossover",
    redistributesD3dmetal: false,
  },
  {
    kind: "custom",
    label: "wine64 manual",
    supportedBackends: ["d3dmetal", "dxmt", "dxvk", "wined3d"],
    defaultBackend: "wined3d",
    installUrl: null,
    redistributesD3dmetal: false,
  },
] as const;

export function catalogEntry(kind: RuntimeKind): RuntimeCatalogEntry {
  switch (kind) {
    case "gptk":
      return RUNTIME_CATALOG[0];
    case "gcenx_staging":
      return RUNTIME_CATALOG[1];
    case "winecx":
      return RUNTIME_CATALOG[2];
    case "crossover":
      return RUNTIME_CATALOG[3];
    case "custom":
      return RUNTIME_CATALOG[4];
    default:
      return assertNever(kind);
  }
}

export function runtimeKindLabel(kind: RuntimeKind): string {
  return catalogEntry(kind).label;
}

export function dxBackendLabel(backend: DxBackend): string {
  switch (backend) {
    case "d3dmetal":
      return "D3DMetal";
    case "dxmt":
      return "DXMT";
    case "dxvk":
      return "DXVK-macOS";
    case "wined3d":
      return "WineD3D";
    default:
      return assertNever(backend);
  }
}

export function emptyRuntime(kind: RuntimeKind): Runtime {
  const entry = catalogEntry(kind);
  return {
    kind: entry.kind,
    label: entry.label,
    wine64Path: "",
    found: false,
    supportedBackends: [...entry.supportedBackends],
    installUrl: entry.installUrl,
  };
}

export function firstFoundRuntime(runtimes: Runtime[]): Runtime | undefined {
  return runtimes.find((runtime) => runtime.found);
}
