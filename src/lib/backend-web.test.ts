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
      name: "Hades",
      exePath: "/Games/Hades/Hades.exe",
      args: "",
      runtimeKind: "gptk",
      dxBackend: "d3dmetal",
    });
    const game = await addPromise;
    expect(game.id).toBe("game-test-id");
    expect(game.prefixStatus).toBe("missing");
    expect(game.prefixPath).toContain("game-test-id");

    const prefixPromise = backend.createPrefix(game.id);
    await vi.runAllTimersAsync();
    const ready = await prefixPromise;
    expect(ready.prefixStatus).toBe("ready");
  });

  it("recusa lançar sem caminho de exe", async () => {
    const backend = new WebPonteBackend();
    await expect(
      backend.addGame({
        name: "",
        exePath: "",
        args: "",
        runtimeKind: "gptk",
        dxBackend: "d3dmetal",
      }),
    ).rejects.toBeInstanceOf(PonteError);
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

  it("detecta GPTK e Gcenx no mock e deixa winecx ausente", async () => {
    const backend = new WebPonteBackend();
    const runtimes = await backend.detectRuntimes();
    const found = runtimes.filter((item) => item.found).map((item) => item.kind);
    expect(found).toContain("gptk");
    expect(found).toContain("gcenx_staging");
    expect(runtimes.find((item) => item.kind === "winecx")?.found).toBe(false);
  });
});
