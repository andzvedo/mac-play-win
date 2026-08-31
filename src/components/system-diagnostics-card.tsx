import {
  CheckCircle2,
  CircleAlert,
  CircleHelp,
  CircleX,
  RefreshCw,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { dxBackendLabel, runtimeKindLabel } from "@/lib/runtime-catalog";
import type { CheckStatus, SystemDiagnostics } from "@/lib/types";

function statusLabel(status: CheckStatus): string {
  switch (status) {
    case "pass":
      return "OK";
    case "warning":
      return "Atenção";
    case "fail":
      return "Falha";
    case "unknown":
      return "Desconhecido";
  }
}

function statusVariant(
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

function StatusIcon({ status }: { status: CheckStatus }) {
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

function formatCheckedAt(value: string): string {
  const date = /^\d+$/.test(value) ? new Date(Number(value) * 1000) : new Date(value);
  return Number.isNaN(date.getTime())
    ? value
    : date.toLocaleString("pt-BR", { dateStyle: "short", timeStyle: "short" });
}

export function SystemDiagnosticsCard({
  diagnostics,
  loading = false,
  onRefresh,
}: {
  diagnostics: SystemDiagnostics | null;
  loading?: boolean;
  onRefresh?: () => void;
}) {
  return (
    <Card>
      <CardHeader className="border-b">
        <CardTitle>Diagnóstico do sistema</CardTitle>
        <CardDescription>
          Verifica os componentes que o Ponte precisa para iniciar jogos locais.
        </CardDescription>
        {onRefresh ? (
          <CardAction>
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={loading}
              onClick={onRefresh}
            >
              <RefreshCw className={loading ? "animate-spin" : undefined} />
              Atualizar
            </Button>
          </CardAction>
        ) : null}
      </CardHeader>
      <CardContent className="grid gap-4 pt-4">
        {!diagnostics ? (
          <p className="text-sm text-muted-foreground">
            Nenhum diagnóstico foi executado ainda.
          </p>
        ) : (
          <>
            <div className="flex flex-wrap items-center gap-2 text-sm">
              <Badge variant={diagnostics.ready ? "default" : "secondary"}>
                {diagnostics.ready ? "Sistema pronto" : "Requer atenção"}
              </Badge>
              <span className="text-muted-foreground">
                Verificado em {formatCheckedAt(diagnostics.checkedAt)}
              </span>
            </div>
            <ul className="grid gap-2">
              {diagnostics.checks.map((check) => (
                <li
                  key={check.id}
                  className="grid gap-2 rounded-lg border p-3 sm:grid-cols-[auto_1fr_auto] sm:items-start"
                >
                  <StatusIcon status={check.status} />
                  <div className="min-w-0">
                    <p className="text-sm font-medium">{check.label}</p>
                    <p className="break-words text-xs text-muted-foreground">
                      {check.detail}
                    </p>
                    {check.remediation ? (
                      <p className="mt-1 text-xs text-foreground/80">
                        Próximo passo: {check.remediation}
                      </p>
                    ) : null}
                  </div>
                  <Badge variant={statusVariant(check.status)}>
                    {statusLabel(check.status)}
                  </Badge>
                </li>
              ))}
            </ul>
            <div className="grid gap-1 text-xs text-muted-foreground sm:grid-cols-2">
              <span>
                Runtime selecionado: {diagnostics.selectedRuntime
                  ? runtimeKindLabel(diagnostics.selectedRuntime)
                  : "nenhum"}
              </span>
              <span>
                Backend selecionado: {diagnostics.selectedBackend
                  ? dxBackendLabel(diagnostics.selectedBackend)
                  : "nenhum"}
              </span>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}
