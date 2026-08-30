import { describe, expect, it } from "vitest";
import { prefixStatusLabel } from "./prefix-status";

describe("prefixStatusLabel", () => {
  it("traduz os três estados", () => {
    expect(prefixStatusLabel("missing")).toBe("Ainda não criado");
    expect(prefixStatusLabel("ready")).toBe("Pronto");
    expect(prefixStatusLabel("error")).toBe("Falhou");
  });
});
