import { describe, expect, it } from "vitest";
import { prefixPathFor } from "./prefix-path";
import { DEFAULT_PREFIXES_DIR } from "./types";

describe("prefixPathFor", () => {
  it("junta a pasta padrão com o id do jogo", () => {
    expect(prefixPathFor(DEFAULT_PREFIXES_DIR, "abc")).toBe(
      `${DEFAULT_PREFIXES_DIR}/abc`,
    );
  });

  it("remove barra final da pasta", () => {
    expect(prefixPathFor("/tmp/ponte/prefixes/", "g1")).toBe(
      "/tmp/ponte/prefixes/g1",
    );
  });
});
