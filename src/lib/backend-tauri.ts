import { invoke } from "@tauri-apps/api/core";
import { PonteError, toPonteError } from "./errors";
import type { PonteBackend } from "./backend";
import type {
  AddGameInput,
  Game,
  GameDiscovery,
  GameLog,
  GameSetupPlan,
  LaunchState,
  PonteErrorCode,
  Runtime,
  RuntimeKind,
  Settings,
} from "./types";

interface RustError {
  code?: PonteErrorCode;
  message?: string;
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    if (error && typeof error === "object") {
      const rustError = error as RustError;
      if (rustError.code && rustError.message) {
        throw new PonteError(rustError.code, rustError.message);
      }
    }
    if (typeof error === "string") {
      try {
        const parsed = JSON.parse(error) as RustError;
        if (parsed.code && parsed.message) {
          throw new PonteError(parsed.code, parsed.message);
        }
      } catch (inner) {
        if (inner instanceof PonteError) {
          throw inner;
        }
      }
      throw new PonteError("unknown", error);
    }
    throw toPonteError(error);
  }
}

export const tauriPonteBackend: PonteBackend = {
  detectRuntimes: () => call<Runtime[]>("detect_runtimes"),
  listGames: () => call<Game[]>("list_games"),
  getGame: (id) => call<Game | null>("get_game", { id }),
  addGame: (input: AddGameInput) => call<Game>("add_game", { input }),
  updateGame: (game: Game) => call<Game>("update_game", { game }),
  removeGame: (id) => call<void>("remove_game", { id }),
  pickExe: () => call<string | null>("pick_exe"),
  pickWine: () => call<string | null>("pick_wine"),
  pickDirectory: () => call<string | null>("pick_directory"),
  pickGameFolder: () => call<string | null>("pick_game_folder"),
  resolveGameFolder: (path: string) =>
    call<GameDiscovery>("resolve_game_folder", { path }),
  createPrefix: (gameId, wipe) =>
    call<Game>("create_prefix", { gameId, wipe: wipe ?? false }),
  installVcrun: (gameId) => call<void>("install_vcrun", { gameId }),
  getGameSetup: (gameId) => call<GameSetupPlan>("get_game_setup", { id: gameId }),
  prepareGame: (gameId) => call<Game>("prepare_game", { gameId }),
  installRuntime: (kind: RuntimeKind) =>
    call<Runtime[]>("install_runtime", { kind }),
  launchGame: (gameId) => call<void>("launch_game", { gameId }),
  stopGame: (gameId) => call<void>("stop_game", { gameId }),
  getLogs: (gameId) => call<GameLog>("get_logs", { gameId }),
  getLaunchState: (gameId) => call<LaunchState>("get_launch_state", { gameId }),
  getSettings: () => call<Settings>("get_settings"),
  saveSettings: (settings) => call<Settings>("save_settings", { settings }),
};
