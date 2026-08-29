import type { PonteBackend } from "./backend";
import { tauriPonteBackend } from "./backend-tauri";
import { WebPonteBackend } from "./backend-web";

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

let webSingleton: WebPonteBackend | undefined;

export function getBackend(): PonteBackend {
  if (isTauri()) {
    return tauriPonteBackend;
  }
  if (!webSingleton) {
    webSingleton = new WebPonteBackend();
  }
  return webSingleton;
}

export const backend = getBackend();
