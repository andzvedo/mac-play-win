import { describe, expect, it } from "vitest";
import {
  gameInstallDir,
  looksLikeNonExeFile,
  resolveGameFolderFromPath,
  suggestedGameName,
} from "./discover-game";
import { PonteError } from "./errors";

describe("discover-game", () => {
  it("sugere o nome a partir da pasta", () => {
    expect(suggestedGameName("/Games/Hollow_Knight")).toBe("Hollow Knight");
    expect(suggestedGameName("/Games/Hades")).toBe("Hades");
  });

  it("usa o shipping se a pasta for Win64", () => {
    expect(
      suggestedGameName(
        "/Games/Hotel/Binaries/Win64",
        "/Games/Hotel/Binaries/Win64/PenguinHotel-Win64-Shipping.exe",
      ),
    ).toBe("PenguinHotel");
  });

  it("resolve pasta no mock web para pasta/Nome.exe", () => {
    const found = resolveGameFolderFromPath("/Games/Hades");
    expect(found.folderPath).toBe("/Games/Hades");
    expect(found.exePath).toBe("/Games/Hades/Hades.exe");
    expect(found.suggestedName).toBe("Hades");
    expect(found.canAutoFix).toBe(true);
    expect(found.setupItems.some((item) => item.status === "installable")).toBe(
      true,
    );
  });

  it("aceita um .exe colado no mock", () => {
    const found = resolveGameFolderFromPath("/Games/Hades/Hades.exe");
    expect(found.exePath).toBe("/Games/Hades/Hades.exe");
    expect(found.folderPath).toBe("/Games/Hades");
  });

  it("recusa ficheiros que não são pasta nem .exe", () => {
    expect(() => resolveGameFolderFromPath("/Games/Hades/Hades.app")).toThrow(
      PonteError,
    );
    expect(looksLikeNonExeFile("/Games/Hades/Hades.app")).toBe(true);
    expect(looksLikeNonExeFile("/Games/Hades")).toBe(false);
  });

  it("recupera a pasta a partir do exe se installDir estiver vazio", () => {
    expect(
      gameInstallDir({
        installDir: "",
        exePath: "/Games/Hades/Hades.exe",
      }),
    ).toBe("/Games/Hades");
  });
});
