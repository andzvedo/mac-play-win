import { dxBackendLabel, runtimeKindLabel } from "./runtime-catalog";
import type { Game } from "./types";

export function filterGames(games: Game[], query: string): Game[] {
  const needle = query.trim().toLowerCase();
  if (!needle) {
    return games;
  }
  return games.filter((game) => {
    const haystack = [
      game.name,
      game.exePath,
      runtimeKindLabel(game.runtimeKind),
      dxBackendLabel(game.dxBackend),
    ]
      .join(" ")
      .toLowerCase();
    return haystack.includes(needle);
  });
}
