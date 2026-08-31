export type RuntimeKind =
  | "gptk"
  | "gcenx_staging"
  | "winehq"
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

export type CheckStatus = "pass" | "warning" | "fail" | "unknown";

export type PreflightStatus = "ready" | "warning" | "blocked" | "unknown";

export type LaunchAttemptStatus =
  | "queued"
  | "preflighting"
  | "preparing"
  | "probing"
  | "starting"
  | "running"
  | "stopping"
  | "succeeded"
  | "failed"
  | "blocked"
  | "interrupted";

export type PonteErrorCode =
  | "runtime_missing"
  | "prefix_failed"
  | "launch_failed"
  | "launch_blocked"
  | "launch_busy"
  | "invalid_exe"
  | "invalid_folder"
  | "not_found"
  | "setup_failed"
  | "unknown";

export type EngineGuess = "unity" | "unreal" | "unknown";

export type SetupItemStatus = "ready" | "missing" | "installable" | "manual";

export interface SetupItem {
  id: string;
  label: string;
  status: SetupItemStatus;
  detail: string;
}

export interface GameSetupPlan {
  gameId: string;
  engineGuess: EngineGuess;
  recommendedRuntime: RuntimeKind;
  recommendedBackend: DxBackend;
  items: SetupItem[];
  canAutoFix: boolean;
  summary: string;
}

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
  installDir: string;
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

export interface DiagnosticCheck {
  id: string;
  label: string;
  status: CheckStatus;
  detail: string;
  remediation?: string;
}

export interface SystemDiagnostics {
  checkedAt: string;
  ready: boolean;
  checks: DiagnosticCheck[];
  selectedRuntime: RuntimeKind | null;
  selectedBackend: DxBackend | null;
}

export interface PreflightCheck {
  id: string;
  label: string;
  status: CheckStatus;
  detail: string;
  remediation?: string;
}

export interface GamePreflight {
  gameId: string;
  executable: string;
  fingerprint: string;
  engineGuess: EngineGuess;
  status: PreflightStatus;
  checks: PreflightCheck[];
  recommendedRuntime: RuntimeKind;
  recommendedBackend: DxBackend;
}

export interface LaunchFailure {
  code: string;
  message: string;
  retryable: boolean;
}

export interface LaunchAttempt {
  id: string;
  gameId: string;
  status: LaunchAttemptStatus;
  startedAt: string;
  updatedAt: string;
  finishedAt?: string;
  executable: string;
  workingDir: string;
  args: string[];
  runtimeKind: RuntimeKind;
  runtimePath: string;
  dxBackend: DxBackend;
  prefixPath: string;
  pid?: number;
  exitCode?: number;
  durationMs?: number;
  profileId?: string;
  preflightFingerprint?: string;
  logPath?: string;
  failure?: LaunchFailure;
}

export interface GameDiscovery {
  folderPath: string;
  exePath: string;
  suggestedName: string;
  engineGuess: EngineGuess;
  recommendedRuntime: RuntimeKind;
  recommendedBackend: DxBackend;
  setupItems: SetupItem[];
  canAutoFix: boolean;
}

export interface AddGameInput {
  name: string;
  folderPath: string;
  args: string;
}

export interface LibraryFile {
  version: 1;
  games: Game[];
}

export const LIBRARY_STORAGE_KEY = "ponte.library";
export const SETTINGS_STORAGE_KEY = "ponte.settings";
export const DEFAULT_PREFIXES_DIR =
  "~/Library/Application Support/Ponte/prefixes";
