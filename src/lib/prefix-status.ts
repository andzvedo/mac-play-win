import { assertNever } from "./assert-never";
import type { PrefixStatus } from "./types";

export function prefixStatusLabel(status: PrefixStatus): string {
  switch (status) {
    case "missing":
      return "Ainda não criado";
    case "ready":
      return "Pronto";
    case "error":
      return "Falhou";
    default:
      return assertNever(status);
  }
}
