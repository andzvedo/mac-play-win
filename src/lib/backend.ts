import type {
  AddGameInput,
  Game,
  GameDiscovery,
  GameLog,
  GamePreflight,
  GameSetupPlan,
  LaunchAttempt,
  LaunchState,
  Runtime,
  RuntimeKind,
  Settings,
  SystemDiagnostics,
} from "./types";

export interface PonteBackend {
  detectRuntimes(): Promise<Runtime[]>;
  listGames(): Promise<Game[]>;
  getGame(id: string): Promise<Game | null>;
  addGame(input: AddGameInput): Promise<Game>;
  updateGame(game: Game): Promise<Game>;
  removeGame(id: string): Promise<void>;
  pickExe(): Promise<string | null>;
  pickWine(): Promise<string | null>;
  pickDirectory(): Promise<string | null>;
  pickGameFolder(): Promise<string | null>;
  resolveGameFolder(path: string): Promise<GameDiscovery>;
  createPrefix(gameId: string, wipe?: boolean): Promise<Game>;
  installVcrun(gameId: string): Promise<void>;
  getGameSetup(gameId: string): Promise<GameSetupPlan>;
  prepareGame(gameId: string): Promise<Game>;
  installRuntime(kind: RuntimeKind): Promise<Runtime[]>;
  launchGame(gameId: string): Promise<LaunchAttempt>;
  stopGame(gameId: string): Promise<void>;
  getLogs(gameId: string): Promise<GameLog>;
  getLaunchState(gameId: string): Promise<LaunchState>;
  getSystemDiagnostics(): Promise<SystemDiagnostics>;
  getGamePreflight(gameId: string): Promise<GamePreflight>;
  listLaunchAttempts(gameId: string, limit?: number): Promise<LaunchAttempt[]>;
  getSettings(): Promise<Settings>;
  saveSettings(settings: Settings): Promise<Settings>;
}
