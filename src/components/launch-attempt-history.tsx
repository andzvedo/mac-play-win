import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { dxBackendLabel, runtimeKindLabel } from "@/lib/runtime-catalog";
import type { LaunchAttempt, LaunchAttemptStatus } from "@/lib/types";

function statusLabel(status: LaunchAttemptStatus): string {
  switch (status) {
    case "queued":
      return "Na fila";
    case "preflighting":
      return "A verificar";
    case "preparing":
      return "A preparar";
    case "probing":
      return "A testar";
    case "starting":
      return "A iniciar";
    case "running":
      return "Em execução";
    case "stopping":
      return "A parar";
    case "succeeded":
      return "Concluída";
    case "failed":
      return "Falhou";
    case "blocked":
      return "Bloqueada";
    case "interrupted":
      return "Interrompida";
  }
}

function statusVariant(
  status: LaunchAttemptStatus,
): "default" | "secondary" | "destructive" | "outline" {
  switch (status) {
    case "succeeded":
      return "default";
    case "failed":
    case "blocked":
    case "interrupted":
      return "destructive";
    case "running":
      return "secondary";
    case "queued":
    case "preflighting":
    case "preparing":
    case "probing":
    case "starting":
    case "stopping":
      return "outline";
  }
}

function formatDate(value: string): string {
  const date = /^\d+$/.test(value) ? new Date(Number(value) * 1000) : new Date(value);
  return Number.isNaN(date.getTime())
    ? value
    : date.toLocaleString("pt-BR", { dateStyle: "short", timeStyle: "short" });
}

function formatDuration(durationMs?: number): string {
  if (durationMs === undefined) {
    return "Em andamento";
  }
  if (durationMs < 1000) {
    return `${durationMs} ms`;
  }
  return `${(durationMs / 1000).toFixed(1)} s`;
}

export function LaunchAttemptHistory({
  attempts,
}: {
  attempts: LaunchAttempt[];
}) {
  return (
    <Card>
      <CardHeader className="border-b">
        <CardTitle>Histórico de inicializações</CardTitle>
        <CardDescription>
          Tentativas recentes, incluindo a etapa em que uma inicialização falhou.
        </CardDescription>
      </CardHeader>
      <CardContent className="pt-4">
        {attempts.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            Nenhuma tentativa registrada.
          </p>
        ) : (
          <ol className="grid gap-3">
            {attempts.map((attempt) => (
              <li key={attempt.id} className="grid gap-3 rounded-lg border p-3">
                <div className="flex flex-wrap items-start justify-between gap-2">
                  <div className="min-w-0">
                    <p className="truncate text-sm font-medium">
                      {formatDate(attempt.startedAt)}
                    </p>
                    <p className="break-all text-xs text-muted-foreground">
                      {attempt.executable}
                    </p>
                  </div>
                  <Badge variant={statusVariant(attempt.status)}>
                    {statusLabel(attempt.status)}
                  </Badge>
                </div>
                <dl className="grid gap-1 text-xs text-muted-foreground sm:grid-cols-2">
                  <div>
                    <dt className="inline font-medium text-foreground/80">Runtime: </dt>
                    <dd className="inline">{runtimeKindLabel(attempt.runtimeKind)}</dd>
                  </div>
                  <div>
                    <dt className="inline font-medium text-foreground/80">Backend: </dt>
                    <dd className="inline">{dxBackendLabel(attempt.dxBackend)}</dd>
                  </div>
                  <div>
                    <dt className="inline font-medium text-foreground/80">Duração: </dt>
                    <dd className="inline">{formatDuration(attempt.durationMs)}</dd>
                  </div>
                  {attempt.exitCode !== undefined ? (
                    <div>
                      <dt className="inline font-medium text-foreground/80">Exit code: </dt>
                      <dd className="inline">{attempt.exitCode}</dd>
                    </div>
                  ) : null}
                  {attempt.pid !== undefined ? (
                    <div>
                      <dt className="inline font-medium text-foreground/80">PID: </dt>
                      <dd className="inline">{attempt.pid}</dd>
                    </div>
                  ) : null}
                </dl>
                {attempt.failure ? (
                  <p className="text-xs text-destructive">
                    {attempt.failure.code}: {attempt.failure.message}
                    {attempt.failure.retryable ? " Pode tentar novamente." : ""}
                  </p>
                ) : null}
              </li>
            ))}
          </ol>
        )}
      </CardContent>
    </Card>
  );
}
