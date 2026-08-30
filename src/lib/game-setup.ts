import { assertNever } from "./assert-never";
import type {
  DxBackend,
  EngineGuess,
  Runtime,
  RuntimeKind,
  SetupItem,
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

export function previewSetup(runtimes: Runtime[]): {
  items: SetupItem[];
  canAutoFix: boolean;
} {
  const wineFound = runtimes.some((item) => item.found);
  const items: SetupItem[] = [
    {
      id: "runtime",
      label: "Wine",
      status: wineFound ? "ready" : "installable",
      detail: wineFound
        ? "Wine já detetado neste Mac."
        : "O Ponte instala o Wine Staging via Homebrew se faltar.",
    },
    {
      id: "graphics",
      label: "DirectX",
      status: "installable",
      detail: "D3DMetal, DXMT ou DXVK-macOS conforme o motor disponível.",
    },
    {
      id: "prefix",
      label: "Prefixo Wine",
      status: "installable",
      detail: "Criado automaticamente na preparação deste jogo.",
    },
    {
      id: "vcrun",
      label: "Visual C++",
      status: "installable",
      detail: "Runtime Microsoft 2015–2022 neste prefixo.",
    },
  ];
  return {
    items,
    canAutoFix: items.some((item) => item.status === "installable"),
  };
}

export function pendingInstallLabels(items: SetupItem[]): string[] {
  return items
    .filter(
      (item) => item.status === "installable" || item.status === "missing",
    )
    .map((item) => item.label);
}

export function gamePrepareKey(gameId: string, exePath: string): string {
  return `${gameId}:${exePath}`;
}

export function shouldAutoPrepare(
  canAutoFix: boolean,
  prepareKey: string,
  alreadyTriedKey: string | null,
  planGameId = "",
  gameId = "",
): boolean {
  if (planGameId && gameId && planGameId !== gameId) {
    return false;
  }
  return canAutoFix && prepareKey.length > 0 && alreadyTriedKey !== prepareKey;
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
