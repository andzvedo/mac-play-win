import { assertNever } from "./assert-never";
import type {
  DxBackend,
  EngineGuess,
  Runtime,
  RuntimeKind,
  SetupItemStatus,
} from "./types";

export function engineGuessLabel(guess: EngineGuess): string {
  switch (guess) {
    case "unity":
      return "Unity";
    case "unreal":
      return "Unreal";
    case "unknown":
      return "Motor não identificado";
    default:
      return assertNever(guess);
  }
}

export function setupItemStatusLabel(status: SetupItemStatus): string {
  switch (status) {
    case "ready":
      return "Pronto";
    case "missing":
      return "Em falta";
    case "installable":
      return "Dá para instalar";
    case "manual":
      return "Instalação manual";
    default:
      return assertNever(status);
  }
}

export function recommendedStack(runtimes: Runtime[]): {
  runtimeKind: RuntimeKind;
  dxBackend: DxBackend;
} {
  const found = (kind: RuntimeKind) =>
    runtimes.find((item) => item.kind === kind && item.found);
  if (found("gptk")) {
    return { runtimeKind: "gptk", dxBackend: "d3dmetal" };
  }
  const crossover = found("crossover");
  if (crossover) {
    return {
      runtimeKind: "crossover",
      dxBackend: crossover.supportedBackends.includes("d3dmetal")
        ? "d3dmetal"
        : "dxvk",
    };
  }
  const staging = found("gcenx_staging");
  if (staging) {
    return {
      runtimeKind: "gcenx_staging",
      dxBackend: staging.supportedBackends.includes("dxmt") ? "dxmt" : "dxvk",
    };
  }
  if (found("winecx")) {
    return { runtimeKind: "winecx", dxBackend: "dxvk" };
  }
  if (found("winehq")) {
    return { runtimeKind: "winehq", dxBackend: "dxvk" };
  }
  const custom = found("custom");
  if (custom) {
    return { runtimeKind: "custom", dxBackend: "wined3d" };
  }
  return { runtimeKind: "gcenx_staging", dxBackend: "dxvk" };
}

export function runtimeCanAutoInstall(kind: RuntimeKind): boolean {
  switch (kind) {
    case "gcenx_staging":
    case "winehq":
      return true;
    case "gptk":
    case "winecx":
    case "crossover":
    case "custom":
      return false;
    default:
      return assertNever(kind);
  }
}
