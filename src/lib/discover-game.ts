import { PonteError } from "./errors";
import { isWindowsExePath } from "./exe-path";
import { recommendedStack } from "./game-setup";
import type { EngineGuess, GameDiscovery, Runtime } from "./types";

export function normalizeFolderPath(path: string): string {
  return path.trim().replace(/[/\\]+$/, "");
}

export function pathBaseName(path: string): string {
  const trimmed = normalizeFolderPath(path);
  return trimmed.split(/[/\\]/).pop() ?? trimmed;
}

export function parentPath(path: string): string {
  const trimmed = normalizeFolderPath(path);
  const parts = trimmed.split(/[/\\]/);
  if (parts.length <= 1) {
    return trimmed;
  }
  parts.pop();
  const parent = parts.join("/");
  return parent.length > 0 ? parent : trimmed;
}

export function looksLikeNonExeFile(path: string): boolean {
  const base = pathBaseName(path);
  if (isWindowsExePath(path)) {
    return false;
  }
  return /\.[a-z0-9]{1,8}$/i.test(base);
}

export function suggestedGameName(folderPath: string, exePath?: string): string {
  const folder = pathBaseName(folderPath);
  const generic = /^(win64|win32|binaries|bin|game|games)$/i.test(folder);
  if (generic && exePath) {
    const stem = pathBaseName(exePath).replace(/\.exe$/i, "");
    return prettifyGameName(stem);
  }
  if (folder) {
    return prettifyGameName(folder);
  }
  if (exePath) {
    return prettifyGameName(pathBaseName(exePath).replace(/\.exe$/i, ""));
  }
  return "Jogo";
}

export function prettifyGameName(name: string): string {
  const stripped = name
    .replace(/-Win64-Shipping$/i, "")
    .replace(/-Win32-Shipping$/i, "")
    .replace(/-Win64$/i, "")
    .replace(/-Win32$/i, "");
  return stripped.replace(/[_-]+/g, " ").trim() || name;
}

export function guessEngineFromExePath(exePath: string): EngineGuess {
  const lower = exePath.replace(/\\/g, "/").toLowerCase();
  if (lower.includes("win64-shipping") || lower.includes("/binaries/win64/")) {
    return "unreal";
  }
  if (lower.includes("_data")) {
    return "unity";
  }
  return "unknown";
}

export function resolveGameFolderFromPath(
  path: string,
  runtimes: Runtime[] = [],
): GameDiscovery {
  const trimmed = normalizeFolderPath(path);
  if (!trimmed) {
    throw new PonteError(
      "invalid_folder",
      "Escolha a pasta onde o jogo está no disco.",
    );
  }
  const stack = recommendedStack(runtimes);
  if (isWindowsExePath(trimmed)) {
    const folderPath = parentPath(trimmed);
    return {
      folderPath,
      exePath: trimmed,
      suggestedName: suggestedGameName(folderPath, trimmed),
      engineGuess: guessEngineFromExePath(trimmed),
      recommendedRuntime: stack.runtimeKind,
      recommendedBackend: stack.dxBackend,
    };
  }
  if (looksLikeNonExeFile(trimmed)) {
    throw new PonteError(
      "invalid_folder",
      "Escolha a pasta do jogo, não um ficheiro. O Ponte encontra o .exe.",
    );
  }
  const name = pathBaseName(trimmed);
  return {
    folderPath: trimmed,
    exePath: `${trimmed}/${name}.exe`,
    suggestedName: suggestedGameName(trimmed),
    engineGuess: "unknown",
    recommendedRuntime: stack.runtimeKind,
    recommendedBackend: stack.dxBackend,
  };
}

export function gameInstallDir(game: { installDir?: string; exePath: string }): string {
  const dir = game.installDir?.trim();
  if (dir) {
    return dir;
  }
  return parentPath(game.exePath);
}
