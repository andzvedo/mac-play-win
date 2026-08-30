import { describe, expect, it } from "vitest";
import { filterGames } from "./filter-games";
import type { Game } from "./types";

function game(partial: Partial<Game> & Pick<Game, "id" | "name" | "exePath">): Game {
  return {
    args: "",
    runtimeKind: "gptk",
    dxBackend: "d3dmetal",
    prefixPath: "/tmp/p",
    prefixStatus: "missing",
    installDir: "",
    createdAt: "0",
    ...partial,
  };
}

describe("filterGames", () => {
  const games = [
    game({ id: "1", name: "Hades", exePath: "/Games/Hades/Hades.exe" }),
    game({
      id: "2",
      name: "Celeste",
      exePath: "/Games/Celeste/Celeste.exe",
      runtimeKind: "gcenx_staging",
      dxBackend: "dxmt",
    }),
  ];

  it("devolve tudo se a busca estiver vazia", () => {
    expect(filterGames(games, "  ")).toHaveLength(2);
  });

  it("filtra por nome", () => {
    expect(filterGames(games, "cel").map((item) => item.id)).toEqual(["2"]);
  });

  it("filtra por motor", () => {
    expect(filterGames(games, "gcenx").map((item) => item.id)).toEqual(["2"]);
  });
});
