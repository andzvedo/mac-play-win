import {
  CheckCircle2,
  CircleAlert,
  CircleHelp,
  CircleX,
  RefreshCw,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { engineGuessLabel } from "@/lib/game-setup";
import { dxBackendLabel, runtimeKindLabel } from "@/lib/runtime-catalog";
import type { CheckStatus, GamePreflight, PreflightStatus } from "@/lib/types";

function checkLabel(status: CheckStatus): string {
  switch (status) {
    case "pass":
      return "OK";
    case "warning":
      return "Atenção";
    case "fail":
      return "Bloqueio";
    case "unknown":
      return "Incerto";
  }
}

function checkVariant(
  status: CheckStatus,
): "default" | "secondary" | "destructive" | "outline" {
  switch (status) {
    case "pass":
      return "default";
    case "warning":
      return "secondary";
    case "fail":
      return "destructive";
    case "unknown":
      return "outline";
  }
}

function CheckIcon({ status }: { status: CheckStatus }) {
  switch (status) {
    case "pass":
      return <CheckCircle2 className="size-4 text-primary" />;
    case "warning":
      return <CircleAlert className="size-4 text-amber-400" />;
    case "fail":
      return <CircleX className="size-4 text-destructive" />;
    case "unknown":
      return <CircleHelp className="size-4 text-muted-foreground" />;
  }
}

function overallLabel(status: PreflightStatus): string {
  switch (status) {
    case "ready":
      return "Pronto para iniciar";
    case "warning":
      return "Pronto após preparação";
    case "blocked":
      return "Bloqueado até corrigir";
    case "unknown":
      return "Validação inconclusiva";
  }
}

export function GamePreflightCard({
  preflight,
  loading = false,
  onRefresh,
}: {
  preflight: GamePreflight | null;
  loading?: boolean;
  onRefresh?: () => void;
}) {
  return (
    <Card>
      <CardHeader className="border-b">
        <CardTitle>Preflight do jogo</CardTitle>
        <CardDescription>
          Confere o executável, a arquitetura, o runtime e as dependências antes do Wine.
        </CardDescription>
        {onRefresh ? (
          <CardAction>
            <Button type="button" variant="outline" size="sm" disabled={loading} onClick={onRefresh}>
              <RefreshCw className={loading ? "animate-spin" : undefined} />
              Verificar
            </Button>
          </CardAction>
        ) : null}
      </CardHeader>
      <CardContent className="grid gap-4 pt-4">
        {!preflight ? (
          <p className="text-sm text-muted-foreground">A verificar o jogo…</p>
        ) : (
          <>
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant={preflight.status === "blocked" ? "destructive" : "default"}>
                {overallLabel(preflight.status)}
              </Badge>
              <span className="text-xs text-muted-foreground">
                {engineGuessLabel(preflight.engineGuess)} · {runtimeKindLabel(preflight.recommendedRuntime)} · {dxBackendLabel(preflight.recommendedBackend)}
              </span>
            </div>
            <ul className="grid gap-2">
              {preflight.checks.map((check) => (
                <li key={check.id} className="grid gap-2 rounded-lg border p-3 sm:grid-cols-[auto_1fr_auto] sm:items-start">
                  <CheckIcon status={check.status} />
                  <div className="min-w-0">
                    <p className="text-sm font-medium">{check.label}</p>
                    <p className="break-words text-xs text-muted-foreground">{check.detail}</p>
                    {check.remediation ? <p className="mt-1 text-xs">Próximo passo: {check.remediation}</p> : null}
                  </div>
                  <Badge variant={checkVariant(check.status)}>{checkLabel(check.status)}</Badge>
                </li>
              ))}
            </ul>
            <p className="break-all text-[11px] text-muted-foreground">
              Fingerprint: {preflight.fingerprint}
            </p>
          </>
        )}
      </CardContent>
    </Card>
  );
}
