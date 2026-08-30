import { PonteError } from "./errors";
import { envForBackend } from "./env-for-backend";
import { prefixPathFor } from "./prefix-path";
import { emptyRuntime, firstFoundRuntime } from "./runtime-catalog";
import type { PonteBackend } from "./backend";
import type {
  AddGameInput,
  Game,
  GameLog,
  LaunchState,
  LibraryFile,
  Runtime,
  RuntimeKind,
  Settings,
} from "./types";
import {
  DEFAULT_PREFIXES_DIR,
  LIBRARY_STORAGE_KEY,
  SETTINGS_STORAGE_KEY,
} from "./types";

const LOG_CAP = 500;

const MOCK_PATHS: Record<Exclude<RuntimeKind, "custom">, string> = {
  gptk: "/opt/game-porting-toolkit/bin/wine64",
  gcenx_staging: "/opt/homebrew/bin/wine64",
  winecx: "",
  crossover: "",
};

function nowIso(): string {
  return new Date().toISOString();
}

function formatLogLine(message: string): string {
  const stamp = new Date().toISOString().slice(11, 19);
  return `[${stamp}] ${message}`;
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

function readJson<T>(key: string): T | null {
  try {
    const raw = localStorage.getItem(key);
    if (!raw) {
      return null;
    }
    return JSON.parse(raw) as T;
  } catch {
    return null;
  }
}

function writeJson(key: string, value: unknown): void {
  localStorage.setItem(key, JSON.stringify(value));
}

export class WebPonteBackend implements PonteBackend {
  private launchStates = new Map<string, LaunchState>();
  private logs = new Map<string, string[]>();

  detectRuntimes(): Promise<Runtime[]> {
    const settings = this.readSettings();
    const kinds: RuntimeKind[] = [
      "gptk",
      "gcenx_staging",
      "winecx",
      "crossover",
      "custom",
    ];
    const runtimes = kinds.map((kind) => {
      if (kind === "custom") {
        const path = settings.customWinePath.trim();
        const runtime = emptyRuntime("custom");
        runtime.wine64Path = path;
        runtime.found = path.length > 0;
        return runtime;
      }
      const runtime = emptyRuntime(kind);
      runtime.wine64Path = MOCK_PATHS[kind];
      runtime.found = MOCK_PATHS[kind].length > 0;
      return runtime;
    });
    return Promise.resolve(runtimes);
  }

  listGames(): Promise<Game[]> {
    return Promise.resolve(this.readLibrary().games);
  }

  getGame(id: string): Promise<Game | null> {
    const game = this.readLibrary().games.find((item) => item.id === id) ?? null;
    return Promise.resolve(game);
  }

  async addGame(input: AddGameInput): Promise<Game> {
    const name = input.name.trim();
    const exePath = input.exePath.trim();
    if (!name || !exePath) {
      throw new PonteError(
        "invalid_exe",
        "Informe o nome e o caminho do .exe.",
      );
    }
    const settings = this.readSettings();
    const id = crypto.randomUUID();
    const game: Game = {
      id,
      name,
      exePath,
      args: input.args.trim(),
      runtimeKind: input.runtimeKind,
      dxBackend: input.dxBackend,
      prefixPath: prefixPathFor(settings.prefixesDir, id),
      prefixStatus: "missing",
      createdAt: nowIso(),
    };
    const library = this.readLibrary();
    library.games.push(game);
    this.writeLibrary(library);
    this.appendLog(id, `Jogo adicionado: ${exePath}`);
    return game;
  }

  async updateGame(game: Game): Promise<Game> {
    const library = this.readLibrary();
    const index = library.games.findIndex((item) => item.id === game.id);
    if (index < 0) {
      throw new PonteError("not_found", "Jogo não encontrado.");
    }
    library.games[index] = game;
    this.writeLibrary(library);
    return game;
  }

  async removeGame(id: string): Promise<void> {
    const library = this.readLibrary();
    library.games = library.games.filter((item) => item.id !== id);
    this.writeLibrary(library);
    this.launchStates.delete(id);
    this.logs.delete(id);
  }

  pickExe(): Promise<string | null> {
    return Promise.resolve(null);
  }

  pickWine(): Promise<string | null> {
    return Promise.resolve(null);
  }

  pickDirectory(): Promise<string | null> {
    return Promise.resolve(null);
  }

  async createPrefix(gameId: string): Promise<Game> {
    const game = await this.requireGame(gameId);
    const runtime = await this.resolveRuntime(game.runtimeKind);
    if (!runtime) {
      const updated = {
        ...game,
        prefixStatus: "error" as const,
        prefixError: "Nenhum wine64 encontrado para criar o prefixo.",
      };
      await this.updateGame(updated);
      throw new PonteError("runtime_missing", updated.prefixError);
    }
    this.setLaunch(gameId, "starting");
    this.appendLog(gameId, `WINEPREFIX=${game.prefixPath}`);
    this.appendLog(gameId, `${runtime.wine64Path} wineboot --init`);
    await delay(350);
    const updated: Game = {
      ...game,
      prefixStatus: "ready",
      prefixError: undefined,
    };
    await this.updateGame(updated);
    this.appendLog(gameId, "Prefixo pronto (simulado neste navegador).");
    this.setLaunch(gameId, "idle");
    return updated;
  }

  async launchGame(gameId: string): Promise<void> {
    const current = this.launchStates.get(gameId);
    if (current === "running" || current === "starting") {
      return;
    }
    const game = await this.requireGame(gameId);
    if (!game.exePath.trim()) {
      throw new PonteError("invalid_exe", "Caminho do .exe está vazio.");
    }
    const runtime = await this.resolveRuntime(game.runtimeKind);
    if (!runtime) {
      throw new PonteError(
        "runtime_missing",
        "Nenhum wine64 disponível. Instale o GPTK ou o Wine Gcenx e aponte o binário em Ajustes.",
      );
    }
    const settings = this.readSettings();
    const env = envForBackend(game.dxBackend, { metalHud: settings.metalHud });
    this.setLaunch(gameId, "starting");
    this.appendLog(gameId, `Motor: ${runtime.label} (${runtime.wine64Path})`);
    this.appendLog(gameId, `Backend: ${game.dxBackend}`);
    this.appendLog(
      gameId,
      `WINEDLLOVERRIDES=${env.WINEDLLOVERRIDES ?? ""}`,
    );
    await delay(400);
    if (game.prefixStatus !== "ready") {
      await this.createPrefix(gameId);
    }
    this.appendLog(gameId, `Iniciando ${game.exePath} (simulado).`);
    this.appendLog(
      gameId,
      "Neste navegador o Ponte não dispara o Wine. No Mac, use npm run tauri dev.",
    );
    this.setLaunch(gameId, "running");
  }

  async stopGame(gameId: string): Promise<void> {
    const state = this.launchStates.get(gameId) ?? "idle";
    if (state === "idle") {
      return;
    }
    this.setLaunch(gameId, "stopping");
    this.appendLog(gameId, "Encerrando processo (simulado).");
    await delay(200);
    this.setLaunch(gameId, "idle");
  }

  getLogs(gameId: string): Promise<GameLog> {
    return Promise.resolve({
      gameId,
      lines: [...(this.logs.get(gameId) ?? [])],
      updatedAt: nowIso(),
    });
  }

  getLaunchState(gameId: string): Promise<LaunchState> {
    return Promise.resolve(this.launchStates.get(gameId) ?? "idle");
  }

  getSettings(): Promise<Settings> {
    return Promise.resolve(this.readSettings());
  }

  saveSettings(settings: Settings): Promise<Settings> {
    const next: Settings = {
      customWinePath: settings.customWinePath.trim(),
      prefixesDir: settings.prefixesDir.trim() || DEFAULT_PREFIXES_DIR,
      metalHud: settings.metalHud,
    };
    writeJson(SETTINGS_STORAGE_KEY, next);
    return Promise.resolve(next);
  }

  private readLibrary(): LibraryFile {
    const parsed = readJson<LibraryFile>(LIBRARY_STORAGE_KEY);
    if (!parsed || parsed.version !== 1 || !Array.isArray(parsed.games)) {
      return { version: 1, games: [] };
    }
    return parsed;
  }

  private writeLibrary(library: LibraryFile): void {
    writeJson(LIBRARY_STORAGE_KEY, library);
  }

  private readSettings(): Settings {
    const parsed = readJson<Settings>(SETTINGS_STORAGE_KEY);
    return {
      customWinePath: parsed?.customWinePath ?? "",
      prefixesDir: parsed?.prefixesDir || DEFAULT_PREFIXES_DIR,
      metalHud: parsed?.metalHud ?? false,
    };
  }

  private async requireGame(id: string): Promise<Game> {
    const game = await this.getGame(id);
    if (!game) {
      throw new PonteError("not_found", "Jogo não encontrado.");
    }
    return game;
  }

  private async resolveRuntime(kind: RuntimeKind): Promise<Runtime | null> {
    const runtimes = await this.detectRuntimes();
    const match = runtimes.find((item) => item.kind === kind && item.found);
    if (match) {
      return match;
    }
    if (kind === "custom") {
      return null;
    }
    return firstFoundRuntime(runtimes) ?? null;
  }

  private setLaunch(gameId: string, state: LaunchState): void {
    this.launchStates.set(gameId, state);
  }

  private appendLog(gameId: string, message: string): void {
    const lines = this.logs.get(gameId) ?? [];
    lines.push(formatLogLine(message));
    if (lines.length > LOG_CAP) {
      lines.splice(0, lines.length - LOG_CAP);
    }
    this.logs.set(gameId, lines);
  }
}
