import { PonteError } from "./errors";
import { envForBackend } from "./env-for-backend";
import { isWindowsExePath } from "./exe-path";
import { resolveGameFolderFromPath, parentPath, guessEngineFromExePath } from "./discover-game";
import { prefixPathFor } from "./prefix-path";
import { emptyRuntime, firstFoundRuntime } from "./runtime-catalog";
import type { PonteBackend } from "./backend";
import type {
  AddGameInput,
  Game,
  GameDiscovery,
  GameLog,
  GameSetupPlan,
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
  winehq: "",
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
      "winehq",
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
    const runtimes = await this.detectRuntimes();
    const discovery = resolveGameFolderFromPath(input.folderPath, runtimes);
    const name = input.name.trim() || discovery.suggestedName;
    if (!name) {
      throw new PonteError("invalid_folder", "Informe o nome do jogo.");
    }
    const settings = this.readSettings();
    const id = crypto.randomUUID();
    const game: Game = {
      id,
      name,
      exePath: discovery.exePath,
      installDir: discovery.folderPath,
      args: input.args.trim(),
      runtimeKind: discovery.recommendedRuntime,
      dxBackend: discovery.recommendedBackend,
      prefixPath: prefixPathFor(settings.prefixesDir, id),
      prefixStatus: "missing",
      createdAt: nowIso(),
    };
    const library = this.readLibrary();
    library.games.push(game);
    this.writeLibrary(library);
    this.appendLog(id, `Pasta do jogo: ${discovery.folderPath}`);
    this.appendLog(id, `Executável encontrado: ${discovery.exePath}`);
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

  pickGameFolder(): Promise<string | null> {
    return Promise.resolve(null);
  }

  async resolveGameFolder(path: string): Promise<GameDiscovery> {
    const runtimes = await this.detectRuntimes();
    return resolveGameFolderFromPath(path, runtimes);
  }

  async createPrefix(gameId: string, wipe = false): Promise<Game> {
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
    if (wipe) {
      this.setLaunch(gameId, "idle");
      this.appendLog(gameId, "A apagar o prefixo anterior (simulado).");
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

  async installVcrun(gameId: string): Promise<void> {
    const plan = await this.getGameSetup(gameId);
    if (plan.items.some((item) => item.id === "vcrun" && item.status === "ready")) {
      this.appendLog(
        gameId,
        "Visual C++ já está neste prefixo (simulado). Nada a instalar.",
      );
      return;
    }
    await this.requireGame(gameId);
    this.appendLog(
      gameId,
      "A instalar Microsoft Visual C++ 2015-2022 (simulado neste navegador).",
    );
    await delay(300);
    this.appendLog(
      gameId,
      "Visual C++ instalado (simulado). No Mac, use o app nativo.",
    );
  }

  async getGameSetup(gameId: string): Promise<GameSetupPlan> {
    const game = await this.requireGame(gameId);
    const runtimes = await this.detectRuntimes();
    const found = firstFoundRuntime(runtimes);
    const prepared =
      game.prefixStatus === "ready" && game.dxBackend === "dxvk";
    return {
      gameId: game.id,
      engineGuess: guessEngineFromExePath(game.exePath),
      recommendedRuntime: found?.kind ?? "gptk",
      recommendedBackend: found ? found.supportedBackends[0] : "d3dmetal",
      items: [
        {
          id: "runtime",
          label: "Wine",
          status: found ? "ready" : "installable",
          detail: found
            ? `${found.label} (simulado)`
            : "Neste navegador o Ponte simula a instalação.",
        },
        {
          id: "graphics",
          label: "DirectX",
          status: prepared ? "ready" : "installable",
          detail: prepared
            ? "DXVK-macOS no prefixo (simulado)."
            : "DXVK-macOS seria instalado no prefixo no app do Mac.",
        },
        {
          id: "prefix",
          label: "Prefixo Wine",
          status: game.prefixStatus === "ready" ? "ready" : "installable",
          detail: game.prefixPath,
        },
        {
          id: "vcrun",
          label: "Visual C++",
          status: prepared ? "ready" : "installable",
          detail: prepared
            ? "Runtime Microsoft neste prefixo (simulado)."
            : "Runtime Microsoft (simulado neste navegador).",
        },
      ],
      canAutoFix: !prepared,
      summary:
        "Neste navegador a preparação é simulada. No Mac, o Ponte instala Wine, DXVK e Visual C++.",
    };
  }

  async prepareGame(gameId: string): Promise<Game> {
    const already = await this.getGameSetup(gameId);
    if (!already.canAutoFix) {
      this.appendLog(gameId, "Já estava tudo instalado (simulado). Nada a fazer.");
      return this.requireGame(gameId);
    }
    const game = await this.requireGame(gameId);
    this.setLaunch(gameId, "starting");
    this.appendLog(gameId, "A analisar o que este jogo precisa (simulado)…");
    await delay(200);
    this.appendLog(gameId, "DXVK-macOS instalado no prefixo (simulado).");
    this.appendLog(gameId, "Visual C++ instalado (simulado).");
    const updated: Game = {
      ...game,
      dxBackend: "dxvk",
      prefixStatus: "ready",
      prefixError: undefined,
    };
    await this.updateGame(updated);
    this.appendLog(gameId, "Preparação concluída (simulada). Pode iniciar.");
    this.setLaunch(gameId, "idle");
    return updated;
  }

  async installRuntime(kind: RuntimeKind): Promise<Runtime[]> {
    this.appendLog(
      "setup",
      `A instalar ${kind} (simulado neste navegador).`,
    );
    await delay(200);
    return this.detectRuntimes();
  }

  async launchGame(gameId: string): Promise<void> {
    const current = this.launchStates.get(gameId);
    if (current === "running" || current === "starting") {
      return;
    }
    await this.requireGame(gameId);
    const setup = await this.getGameSetup(gameId);
    if (setup.canAutoFix) {
      await this.prepareGame(gameId);
    }
    const game = await this.requireGame(gameId);
    if (!isWindowsExePath(game.exePath)) {
      throw new PonteError(
        "invalid_exe",
        "O caminho deve apontar para um ficheiro .exe.",
      );
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
    return {
      version: 1,
      games: parsed.games.map((game) => this.normalizeGame(game)),
    };
  }

  private normalizeGame(game: Game): Game {
    return {
      ...game,
      installDir: game.installDir?.trim() || parentPath(game.exePath),
    };
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
