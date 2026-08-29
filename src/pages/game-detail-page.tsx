import { useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { ArrowLeft, LoaderCircle, Play, Square } from "lucide-react";
import { CompatibilityNotice } from "@/components/compatibility-notice";
import { LogPanel } from "@/components/log-panel";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { usePonte } from "@/context/ponte-context";
import { assertNever } from "@/lib/assert-never";
import { catalogEntry, dxBackendLabel, runtimeKindLabel } from "@/lib/runtime-catalog";
import type { DxBackend, LaunchState, RuntimeKind } from "@/lib/types";

function statusCopy(state: LaunchState): string {
  switch (state) {
    case "idle":
      return "Parado";
    case "starting":
      return "A iniciar…";
    case "running":
      return "A correr";
    case "stopping":
      return "A parar…";
    case "error":
      return "Erro no último lançamento";
    default:
      return assertNever(state);
  }
}

export function GameDetailPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const {
    games,
    foundRuntimes,
    launch,
    logs,
    actionError,
    loading,
    updateGame,
    createPrefix,
    launchGame,
    stopGame,
    removeGame,
  } = usePonte();
  const [busy, setBusy] = useState(false);
  const game = games.find((item) => item.id === id);

  if (!loading && id && !game) {
    return (
      <div className="mx-auto max-w-xl space-y-4">
        <p>Jogo não encontrado.</p>
        <Button asChild variant="outline">
          <Link to="/">Voltar à biblioteca</Link>
        </Button>
      </div>
    );
  }

  if (!game) {
    return <p className="text-sm text-muted-foreground">A carregar…</p>;
  }

  const state = launch[game.id] ?? "idle";
  const running = state === "running" || state === "starting";
  const motors =
    foundRuntimes.length > 0
      ? foundRuntimes
      : [{ kind: game.runtimeKind, label: runtimeKindLabel(game.runtimeKind) }];
  const backends = catalogEntry(game.runtimeKind).supportedBackends;

  async function withBusy(fn: () => Promise<void>) {
    setBusy(true);
    try {
      await fn();
    } catch {
      /* mensagem já vai em actionError */
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="mx-auto flex max-w-3xl flex-col gap-6">
      <div className="flex items-center gap-2">
        <Button asChild variant="ghost" size="sm">
          <Link to="/">
            <ArrowLeft />
            Biblioteca
          </Link>
        </Button>
      </div>

      <div className="space-y-1">
        <h1 className="font-heading text-2xl font-semibold">{game.name}</h1>
        <p className="break-all text-sm text-muted-foreground">{game.exePath}</p>
        <p className="text-sm">{statusCopy(state)}</p>
      </div>

      {actionError ? (
        <Alert variant="destructive">
          <AlertTitle>Falha</AlertTitle>
          <AlertDescription>{actionError}</AlertDescription>
        </Alert>
      ) : null}

      <div className="flex flex-wrap gap-2">
        {running ? (
          <Button
            variant="secondary"
            disabled={busy}
            onClick={() => void withBusy(() => stopGame(game.id))}
          >
            {busy ? <LoaderCircle className="animate-spin" /> : <Square />}
            Parar
          </Button>
        ) : (
          <Button
            disabled={busy}
            onClick={() => void withBusy(() => launchGame(game.id))}
          >
            {busy ? <LoaderCircle className="animate-spin" /> : <Play />}
            Iniciar
          </Button>
        )}
        <Button
          variant="outline"
          disabled={busy}
          onClick={() => void withBusy(() => createPrefix(game.id))}
        >
          Recriar prefixo
        </Button>
        <AlertDialog>
          <AlertDialogTrigger asChild>
            <Button variant="destructive" disabled={busy}>
              Remover
            </Button>
          </AlertDialogTrigger>
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>Remover da biblioteca?</AlertDialogTitle>
              <AlertDialogDescription>
                O .exe no disco não é apagado. O prefixo Wine fica no disco nesta
                versão.
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel>Cancelar</AlertDialogCancel>
              <AlertDialogAction
                variant="destructive"
                onClick={() => {
                  void removeGame(game.id).then(() => navigate("/"));
                }}
              >
                Remover
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      </div>

      <div className="grid gap-4 sm:grid-cols-2">
        <div className="grid gap-2">
          <Label>Motor</Label>
          <Select
            value={game.runtimeKind}
            onValueChange={(value) => {
              const kind = value as RuntimeKind;
              const nextBackend = catalogEntry(kind).supportedBackends.includes(
                game.dxBackend,
              )
                ? game.dxBackend
                : catalogEntry(kind).defaultBackend;
              void updateGame({
                ...game,
                runtimeKind: kind,
                dxBackend: nextBackend,
              });
            }}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {motors.map((runtime) => (
                <SelectItem key={runtime.kind} value={runtime.kind}>
                  {runtime.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="grid gap-2">
          <Label>Backend DirectX</Label>
          <Select
            value={game.dxBackend}
            onValueChange={(value) =>
              void updateGame({ ...game, dxBackend: value as DxBackend })
            }
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {backends.map((backend) => (
                <SelectItem key={backend} value={backend}>
                  {dxBackendLabel(backend)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      <div className="grid gap-2">
        <Label htmlFor="args">Argumentos</Label>
        <Input
          id="args"
          defaultValue={game.args}
          onBlur={(event) => {
            const value = event.target.value;
            if (value !== game.args) {
              void updateGame({ ...game, args: value });
            }
          }}
        />
      </div>

      <div className="grid gap-2">
        <Label>Prefixo</Label>
        <p className="break-all text-sm text-muted-foreground">{game.prefixPath}</p>
        <p className="text-xs text-muted-foreground">
          Estado: {game.prefixStatus}
          {game.prefixError ? ` — ${game.prefixError}` : ""}
        </p>
      </div>

      <div className="grid gap-2">
        <Label>Log</Label>
        <LogPanel lines={logs[game.id] ?? []} />
      </div>

      <CompatibilityNotice compact />
    </div>
  );
}
