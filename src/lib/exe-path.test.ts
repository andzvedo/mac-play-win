import { describe, expect, it } from "vitest";
import { isWindowsExePath } from "./exe-path";

describe("isWindowsExePath", () => {
  it("aceita .exe no fim do ficheiro", () => {
    expect(isWindowsExePath("/Games/Hades/Hades.exe")).toBe(true);
    expect(isWindowsExePath("C:\\Games\\Hades.EXE")).toBe(true);
    expect(isWindowsExePath("  /tmp/jogo.exe  ")).toBe(true);
  });

  it("recusa caminhos vazios ou sem sufixo .exe", () => {
    expect(isWindowsExePath("")).toBe(false);
    expect(isWindowsExePath("   ")).toBe(false);
    expect(isWindowsExePath("/Games/Hades/Hades.app")).toBe(false);
    expect(isWindowsExePath("/Games/Hades/Hades.exe.bak")).toBe(false);
    expect(isWindowsExePath("exe")).toBe(false);
  });
});
