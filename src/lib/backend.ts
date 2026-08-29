import type {
  AddGameInput,
  Game,
  GameLog,
  Runtime,
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
  createPrefix(gameId: string): Promise<Game>;
  launchGame(gameId: string): Promise<void>;
  stopGame(gameId: string): Promise<void>;
  getLogs(gameId: string): Promise<GameLog>;
  getLaunchState(gameId: string): Promise<import("./types").LaunchState>;
  getSettings(): Promise<Settings>;
  saveSettings(settings: Settings): Promise<Settings>;
}
