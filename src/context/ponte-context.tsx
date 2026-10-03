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
  GamePreflight,
  GameSetupPlan,
  LaunchState,
  LaunchAttempt,
  Runtime,
  RuntimeKind,
  Settings,
  SystemDiagnostics,
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
  systemDiagnostics: SystemDiagnostics | null;
  preflights: Record<string, GamePreflight>;
  launchAttempts: Record<string, LaunchAttempt[]>;
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
  getSystemDiagnostics: () => Promise<SystemDiagnostics>;
  getGamePreflight: (id: string) => Promise<GamePreflight>;
  listLaunchAttempts: (id: string, limit?: number) => Promise<LaunchAttempt[]>;
  prepareGame: (id: string) => Promise<Game>;
  installRuntime: (kind: RuntimeKind) => Promise<void>;
  launchGame: (id: string) => Promise<LaunchAttempt>;
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
  const [systemDiagnostics, setSystemDiagnostics] =
    useState<SystemDiagnostics | null>(null);
  const [preflights, setPreflights] = useState<Record<string, GamePreflight>>(
    {},
  );
  const [launchAttempts, setLaunchAttempts] = useState<
    Record<string, LaunchAttempt[]>
  >({});
  const [actionError, setActionError] = useState<string | null>(null);

  const refreshGameMeta = useCallback(
    async (id: string) => {
      const [state, log, attempts] = await Promise.all([
        backend.getLaunchState(id),
        backend.getLogs(id),
        backend.listLaunchAttempts(id, 10),
      ]);
      setLaunch((current) =>
        current[id] === state ? current : { ...current, [id]: state },
      );
      setLogs((current) =>
        sameLogLines(current[id], log.lines)
          ? current
          : { ...current, [id]: log.lines },
      );
      setLaunchAttempts((current) => ({ ...current, [id]: attempts }));
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
      systemDiagnostics,
      preflights,
      launchAttempts,
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
          setPreflights((current) => {
            const next = { ...current };
            delete next[id];
            return next;
          });
          setLaunchAttempts((current) => {
            const next = { ...current };
            delete next[id];
            return next;
          });
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
      getSystemDiagnostics: () =>
        runAction(async () => {
          const diagnostics = await backend.getSystemDiagnostics();
          setSystemDiagnostics(diagnostics);
          return diagnostics;
        }),
      getGamePreflight: (id) =>
        runAction(async () => {
          const preflight = await backend.getGamePreflight(id);
          setPreflights((current) => ({ ...current, [id]: preflight }));
          return preflight;
        }),
      listLaunchAttempts: (id, limit) =>
        runAction(async () => {
          const attempts = await backend.listLaunchAttempts(id, limit);
          setLaunchAttempts((current) => ({ ...current, [id]: attempts }));
          return attempts;
        }),
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
          const attempt = await backend.launchGame(id);
          setLaunchAttempts((current) => ({
            ...current,
            [id]: [
              attempt,
              ...(current[id] ?? []).filter((item) => item.id !== attempt.id),
            ],
          }));
          await refresh();
          await refreshGameMeta(id);
          return attempt;
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
      launchAttempts,
      ready,
      preflights,
      refresh,
      refreshGameMeta,
      runAction,
      runtimes,
      settings,
      systemDiagnostics,
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
