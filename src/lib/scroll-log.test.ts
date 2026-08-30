import { describe, expect, it } from "vitest";
import { sameLogLines, shouldStickLogToEnd } from "./scroll-log";

describe("shouldStickLogToEnd", () => {
  it("cola no fim no primeiro paint", () => {
    expect(shouldStickLogToEnd(0, 800, 224, true)).toBe(true);
  });

  it("cola no fim quando o utilizador já está no fundo", () => {
    expect(shouldStickLogToEnd(560, 800, 224, false)).toBe(true);
  });

  it("não cola no fim se o utilizador subiu no painel", () => {
    expect(shouldStickLogToEnd(0, 800, 224, false)).toBe(false);
  });
});

describe("sameLogLines", () => {
  it("trata a mesma sequência como igual", () => {
    expect(sameLogLines(["a", "b"], ["a", "b"])).toBe(true);
  });

  it("deteta linhas novas ou diferentes", () => {
    expect(sameLogLines(["a"], ["a", "b"])).toBe(false);
    expect(sameLogLines(undefined, ["a"])).toBe(false);
    expect(sameLogLines(["a"], ["b"])).toBe(false);
  });
});
