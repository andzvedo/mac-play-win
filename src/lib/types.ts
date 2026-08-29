export type RuntimeKind =
  | "gptk"
  | "gcenx_staging"
  | "winecx"
  | "crossover"
  | "custom";

export type DxBackend = "d3dmetal" | "dxmt" | "dxvk" | "wined3d";

export type PrefixStatus = "missing" | "ready" | "error";

export type LaunchState =
  | "idle"
  | "starting"
  | "running"
  | "stopping"
  | "error";

export type PonteErrorCode =
  | "runtime_missing"
  | "prefix_failed"
  | "launch_failed"
  | "invalid_exe"
  | "not_found"
  | "unknown";

export interface Runtime {
  kind: RuntimeKind;
  label: string;
  wine64Path: string;
  found: boolean;
  supportedBackends: DxBackend[];
  installUrl: string | null;
}

export interface Game {
  id: string;
  name: string;
  exePath: string;
  args: string;
  runtimeKind: RuntimeKind;
  dxBackend: DxBackend;
  prefixPath: string;
  prefixStatus: PrefixStatus;
  prefixError?: string;
  createdAt: string;
}

export interface Settings {
  customWinePath: string;
  prefixesDir: string;
  metalHud: boolean;
}

export interface GameLog {
  gameId: string;
  lines: string[];
  updatedAt: string;
}

export interface AddGameInput {
  name: string;
  exePath: string;
  args: string;
  runtimeKind: RuntimeKind;
  dxBackend: DxBackend;
}

export interface LibraryFile {
  version: 1;
  games: Game[];
}

export const LIBRARY_STORAGE_KEY = "ponte.library";
export const SETTINGS_STORAGE_KEY = "ponte.settings";
export const DEFAULT_PREFIXES_DIR =
  "~/Library/Application Support/Ponte/prefixes";
