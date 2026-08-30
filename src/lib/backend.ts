import type {
  AddGameInput,
  Game,
  GameDiscovery,
  GameLog,
  GameSetupPlan,
  LaunchState,
  Runtime,
  RuntimeKind,
  Settings,
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
  launchGame(gameId: string): Promise<void>;
  stopGame(gameId: string): Promise<void>;
  getLogs(gameId: string): Promise<GameLog>;
  getLaunchState(gameId: string): Promise<LaunchState>;
  getSettings(): Promise<Settings>;
  saveSettings(settings: Settings): Promise<Settings>;
}

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
  createPrefix(gameId: string, wipe?: boolean): Promise<Game>;
  installVcrun(gameId: string): Promise<void>;
  getGameSetup(gameId: string): Promise<GameSetupPlan>;
  prepareGame(gameId: string): Promise<Game>;
  installRuntime(kind: RuntimeKind): Promise<Runtime[]>;
  launchGame(gameId: string): Promise<void>;
  stopGame(gameId: string): Promise<void>;
  getLogs(gameId: string): Promise<GameLog>;
  getLaunchState(gameId: string): Promise<LaunchState>;
  getSettings(): Promise<Settings>;
  saveSettings(settings: Settings): Promise<Settings>;
}
