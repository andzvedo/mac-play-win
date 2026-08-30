import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { getBackend } from "@/lib/ponte";
import { isPonteError, toPonteError } from "@/lib/errors";
import { sameLogLines } from "@/lib/scroll-log";
import type {
  AddGameInput,
  Game,
  GameDiscovery,
  GameSetupPlan,
  LaunchState,
  Runtime,
  RuntimeKind,
  Settings,
} from "@/lib/types";
import { DEFAULT_PREFIXES_DIR } from "@/lib/types";

interface PonteContextValue {
  ready: boolean;
  loading: boolean;
  games: Game[];
  runtimes: Runtime[];
  settings: Settings;
  launch: Record<string, LaunchState>;
  logs: Record<string, string[]>;
  actionError: string | null;
  foundRuntimes: Runtime[];
  refresh: () => Promise<void>;
  addGame: (input: AddGameInput) => Promise<Game>;
  updateGame: (game: Game) => Promise<Game>;
  removeGame: (id: string) => Promise<void>;
  pickExe: () => Promise<string | null>;
  pickWine: () => Promise<string | null>;
  pickDirectory: () => Promise<string | null>;
  pickGameFolder: () => Promise<string | null>;
  resolveGameFolder: (path: string) => Promise<GameDiscovery>;
  createPrefix: (id: string, wipe?: boolean) => Promise<void>;
  installVcrun: (id: string) => Promise<void>;
  getGameSetup: (id: string) => Promise<GameSetupPlan>;
  prepareGame: (id: string) => Promise<Game>;
  installRuntime: (kind: RuntimeKind) => Promise<void>;
  launchGame: (id: string) => Promise<void>;
  stopGame: (id: string) => Promise<void>;
  saveSettings: (settings: Settings) => Promise<void>;
  clearActionError: () => void;
}

const PonteContext = createContext<PonteContextValue | null>(null);

export function PonteProvider({ children }: { children: ReactNode }) {
  const backend = useMemo(() => getBackend(), []);
  const [ready, setReady] = useState(false);
  const [loading, setLoading] = useState(true);
  const [games, setGames] = useState<Game[]>([]);
  const [runtimes, setRuntimes] = useState<Runtime[]>([]);
  const [settings, setSettings] = useState<Settings>({
    customWinePath: "",
    prefixesDir: DEFAULT_PREFIXES_DIR,
    metalHud: false,
  });
  const [launch, setLaunch] = useState<Record<string, LaunchState>>({});
  const [logs, setLogs] = useState<Record<string, string[]>>({});
  const [actionError, setActionError] = useState<string | null>(null);

  const refreshGameMeta = useCallback(
    async (id: string) => {
      const [state, log] = await Promise.all([
        backend.getLaunchState(id),
        backend.getLogs(id),
      ]);
      setLaunch((current) =>
        current[id] === state ? current : { ...current, [id]: state },
      );
      setLogs((current) =>
        sameLogLines(current[id], log.lines)
          ? current
          : { ...current, [id]: log.lines },
      );
    },
    [backend],
  );

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const [nextGames, nextRuntimes, nextSettings] = await Promise.all([
        backend.listGames(),
        backend.detectRuntimes(),
        backend.getSettings(),
      ]);
      setGames(nextGames);
      setRuntimes(nextRuntimes);
      setSettings(nextSettings);
      await Promise.all(nextGames.map((game) => refreshGameMeta(game.id)));
    } finally {
      setLoading(false);
      setReady(true);
    }
  }, [backend, refreshGameMeta]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const busyIds = Object.entries(launch)
    .filter(([, state]) => state === "running" || state === "starting")
    .map(([id]) => id)
    .join(",");

  useEffect(() => {
    if (!busyIds) {
      return;
    }
    const ids = busyIds.split(",");
    const timer = window.setInterval(() => {
      void Promise.all(ids.map((id) => refreshGameMeta(id)));
    }, 1200);
    return () => window.clearInterval(timer);
  }, [busyIds, refreshGameMeta]);

  const runAction = useCallback(async <T,>(fn: () => Promise<T>): Promise<T> => {
    try {
      setActionError(null);
      return await fn();
    } catch (error) {
      const ponteError = isPonteError(error) ? error : toPonteError(error);
      setActionError(ponteError.message);
      throw ponteError;
    }
  }, []);

  const value = useMemo<PonteContextValue>(
    () => ({
      ready,
      loading,
      games,
      runtimes,
      settings,
      launch,
      logs,
      actionError,
      foundRuntimes: runtimes.filter((item) => item.found),
      refresh,
      addGame: (input) =>
        runAction(async () => {
          const game = await backend.addGame(input);
          await refresh();
          return game;
        }),
      updateGame: (game) =>
        runAction(async () => {
          const updated = await backend.updateGame(game);
          await refresh();
          return updated;
        }),
      removeGame: (id) =>
        runAction(async () => {
          await backend.removeGame(id);
          await refresh();
        }),
      pickExe: () => backend.pickExe(),
      pickWine: () => backend.pickWine(),
      pickDirectory: () => backend.pickDirectory(),
      pickGameFolder: () => backend.pickGameFolder(),
      resolveGameFolder: (path) => backend.resolveGameFolder(path),
      createPrefix: (id, wipe) =>
        runAction(async () => {
          await backend.createPrefix(id, wipe);
          await refresh();
          await refreshGameMeta(id);
        }),
      installVcrun: (id) =>
        runAction(async () => {
          await backend.installVcrun(id);
          await refreshGameMeta(id);
        }),
      getGameSetup: (id) => backend.getGameSetup(id),
      prepareGame: (id) =>
        runAction(async () => {
          setLaunch((current) => ({ ...current, [id]: "starting" }));
          const game = await backend.prepareGame(id);
          await refresh();
          await refreshGameMeta(id);
          return game;
        }),
      installRuntime: (kind) =>
        runAction(async () => {
          const next = await backend.installRuntime(kind);
          setRuntimes(next);
        }),
      launchGame: (id) =>
        runAction(async () => {
          setLaunch((current) => ({ ...current, [id]: "starting" }));
          await backend.launchGame(id);
          await refresh();
          await refreshGameMeta(id);
        }),
      stopGame: (id) =>
        runAction(async () => {
          await backend.stopGame(id);
          await refreshGameMeta(id);
        }),
      saveSettings: (next) =>
        runAction(async () => {
          const saved = await backend.saveSettings(next);
          setSettings(saved);
          await refresh();
        }),
      clearActionError: () => setActionError(null),
    }),
    [
      actionError,
      backend,
      games,
      launch,
      loading,
      logs,
      ready,
      refresh,
      refreshGameMeta,
      runAction,
      runtimes,
      settings,
    ],
  );

  return <PonteContext.Provider value={value}>{children}</PonteContext.Provider>;
}

export function usePonte(): PonteContextValue {
  const value = useContext(PonteContext);
  if (!value) {
    throw new Error("usePonte deve ser usado dentro de PonteProvider");
  }
  return value;
}
