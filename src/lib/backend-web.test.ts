import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { WebPonteBackend } from "./backend-web";
import { PonteError } from "./errors";
import { LIBRARY_STORAGE_KEY, SETTINGS_STORAGE_KEY } from "./types";

function memoryStorage(): Storage {
  const data = new Map<string, string>();
  return {
    get length() {
      return data.size;
    },
    clear() {
      data.clear();
    },
    getItem(key) {
      return data.get(key) ?? null;
    },
    key(index) {
      return [...data.keys()][index] ?? null;
    },
    removeItem(key) {
      data.delete(key);
    },
    setItem(key, value) {
      data.set(key, value);
    },
  };
}

describe("WebPonteBackend", () => {
  beforeEach(() => {
    vi.stubGlobal("localStorage", memoryStorage());
    vi.stubGlobal("crypto", {
      randomUUID: () => "game-test-id",
    });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.useRealTimers();
  });

  it("recupera biblioteca vazia se o JSON estiver corrompido", async () => {
    localStorage.setItem(LIBRARY_STORAGE_KEY, "{nope");
    const backend = new WebPonteBackend();
    await expect(backend.listGames()).resolves.toEqual([]);
  });

  it("adiciona um .exe local e cria prefixo simulado", async () => {
    vi.useFakeTimers();
    const backend = new WebPonteBackend();
    const addPromise = backend.addGame({
      name: "",
      folderPath: "/Games/Hades",
      args: "",
    });
    const game = await addPromise;
    expect(game.id).toBe("game-test-id");
    expect(game.name).toBe("Hades");
    expect(game.exePath).toBe("/Games/Hades/Hades.exe");
    expect(game.installDir).toBe("/Games/Hades");
    expect(game.runtimeKind).toBe("gptk");
    expect(game.dxBackend).toBe("d3dmetal");
    expect(game.prefixStatus).toBe("missing");
    expect(game.prefixPath).toContain("game-test-id");

    const prefixPromise = backend.createPrefix(game.id);
    await vi.runAllTimersAsync();
    const ready = await prefixPromise;
    expect(ready.prefixStatus).toBe("ready");
  });

  it("recusa adicionar sem pasta", async () => {
    const backend = new WebPonteBackend();
    await expect(
      backend.addGame({
        name: "",
        folderPath: "",
        args: "",
      }),
    ).rejects.toBeInstanceOf(PonteError);
  });

  it("recusa um ficheiro que não é pasta de jogo", async () => {
    const backend = new WebPonteBackend();
    await expect(
      backend.addGame({
        name: "",
        folderPath: "/Games/Hades/Hades.app",
        args: "",
      }),
    ).rejects.toMatchObject({ code: "invalid_folder" });
  });

  it("recusa lançar um jogo cujo caminho deixou de ser .exe", async () => {
    const backend = new WebPonteBackend();
    const game = await backend.addGame({
      name: "",
      folderPath: "/Games/Hades",
      args: "",
    });
    await backend.updateGame({ ...game, exePath: "/Games/Hades/Hades.app" });
    await expect(backend.launchGame(game.id)).rejects.toMatchObject({
      code: "invalid_exe",
    });
  });

  it("recriar prefixo regista o wipe simulado", async () => {
    vi.useFakeTimers();
    const backend = new WebPonteBackend();
    const game = await backend.addGame({
      name: "",
      folderPath: "/Games/Hades",
      args: "",
    });
    const prefixPromise = backend.createPrefix(game.id, true);
    await vi.runAllTimersAsync();
    await prefixPromise;
    const log = await backend.getLogs(game.id);
    expect(log.lines.some((line) => line.includes("apagar o prefixo"))).toBe(
      true,
    );
  });

  it("simula a instalação do Visual C++ no prefixo", async () => {
    vi.useFakeTimers();
    const backend = new WebPonteBackend();
    const game = await backend.addGame({
      name: "",
      folderPath: "/Games/Hades",
      args: "",
    });
    const installPromise = backend.installVcrun(game.id);
    await vi.runAllTimersAsync();
    await installPromise;
    const log = await backend.getLogs(game.id);
    expect(
      log.lines.some((line) => line.includes("Visual C++ instalado (simulado)")),
    ).toBe(true);
  });

  it("persiste ajustes", async () => {
    const backend = new WebPonteBackend();
    await backend.saveSettings({
      customWinePath: "/custom/wine64",
      prefixesDir: "/tmp/prefixes",
      metalHud: true,
    });
    const settings = await backend.getSettings();
    expect(settings.metalHud).toBe(true);
    expect(settings.customWinePath).toBe("/custom/wine64");
    expect(JSON.parse(localStorage.getItem(SETTINGS_STORAGE_KEY) ?? "{}").metalHud).toBe(
      true,
    );
  });

  it("prepara o jogo no mock e marca o plano como pronto", async () => {
    vi.useFakeTimers();
    const backend = new WebPonteBackend();
    const game = await backend.addGame({
      name: "",
      folderPath: "/Games/Hades",
      args: "",
    });
    const before = await backend.getGameSetup(game.id);
    expect(before.canAutoFix).toBe(true);
    const preparePromise = backend.prepareGame(game.id);
    await vi.runAllTimersAsync();
    const prepared = await preparePromise;
    expect(prepared.dxBackend).toBe("dxvk");
    expect(prepared.prefixStatus).toBe("ready");
    const after = await backend.getGameSetup(game.id);
    expect(after.canAutoFix).toBe(false);
    expect(after.items.every((item) => item.status === "ready")).toBe(true);

    const again = backend.prepareGame(game.id);
    await vi.runAllTimersAsync();
    await again;
    await backend.installVcrun(game.id);
    const log = await backend.getLogs(game.id);
    expect(
      log.lines.filter((line) => line.includes("Visual C++ instalado (simulado)"))
        .length,
    ).toBe(1);
    expect(
      log.lines.some((line) => line.includes("Já estava tudo instalado")),
    ).toBe(true);
    expect(
      log.lines.some((line) => line.includes("Nada a instalar")),
    ).toBe(true);
  });

  it("detecta GPTK e Gcenx no mock e deixa winecx ausente", async () => {
    const backend = new WebPonteBackend();
    const runtimes = await backend.detectRuntimes();
    const found = runtimes.filter((item) => item.found).map((item) => item.kind);
    expect(found).toContain("gptk");
    expect(found).toContain("gcenx_staging");
    expect(runtimes.find((item) => item.kind === "winecx")?.found).toBe(false);
    expect(runtimes.find((item) => item.kind === "winehq")?.found).toBe(false);
  });
});
