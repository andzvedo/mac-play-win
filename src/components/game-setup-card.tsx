import { Wrench } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { engineGuessLabel, setupItemStatusLabel } from "@/lib/game-setup";
import { dxBackendLabel, runtimeKindLabel } from "@/lib/runtime-catalog";
import type { GameSetupPlan, SetupItemStatus } from "@/lib/types";

function statusVariant(
  status: SetupItemStatus,
): "default" | "secondary" | "destructive" | "outline" {
  switch (status) {
    case "ready":
      return "default";
    case "installable":
      return "outline";
    case "manual":
      return "secondary";
    case "missing":
      return "destructive";
    default: {
      const _exhaustive: never = status;
      return _exhaustive;
    }
  }
}

export function GameSetupCard({
  plan,
  busy,
  onPrepare,
}: {
  plan: GameSetupPlan;
  busy: boolean;
  onPrepare: () => void;
}) {
  return (
    <section className="grid gap-3 rounded-xl border p-4">
      <div className="space-y-1">
        <h2 className="font-heading text-sm font-medium">O que este jogo precisa</h2>
        <p className="text-sm text-muted-foreground">{plan.summary}</p>
        <p className="text-xs text-muted-foreground">
          Detetado: {engineGuessLabel(plan.engineGuess)}. Recomendado:{" "}
          {runtimeKindLabel(plan.recommendedRuntime)} +{" "}
          {dxBackendLabel(plan.recommendedBackend)}.
        </p>
      </div>
      <ul className="grid gap-2">
        {plan.items.map((item) => (
          <li
            key={item.id}
            className="flex flex-col gap-1 rounded-lg border p-3 sm:flex-row sm:items-start sm:justify-between"
          >
            <div>
              <p className="text-sm font-medium">{item.label}</p>
              <p className="text-xs text-muted-foreground">{item.detail}</p>
            </div>
            <Badge variant={statusVariant(item.status)}>
              {setupItemStatusLabel(item.status)}
            </Badge>
          </li>
        ))}
      </ul>
      {plan.canAutoFix ? (
        <Button
          type="button"
          variant="outline"
          disabled={busy}
          onClick={onPrepare}
        >
          <Wrench />
          {busy ? "A preparar…" : "Instalar o que falta"}
        </Button>
      ) : plan.items.every((item) => item.status === "ready") ? (
        <p className="text-xs text-muted-foreground">
          Já está instalado neste prefixo. O Ponte não volta a instalar.
        </p>
      ) : null}
    </section>
  );
}
