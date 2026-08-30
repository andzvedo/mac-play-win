import { Badge } from "@/components/ui/badge";
import { Card, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { prefixStatusLabel } from "@/lib/prefix-status";
import { dxBackendLabel, runtimeKindLabel } from "@/lib/runtime-catalog";
import { assertNever } from "@/lib/assert-never";
import type { Game, LaunchState } from "@/lib/types";
import { Link } from "react-router-dom";

function statusLabel(state: LaunchState): string {
  switch (state) {
    case "idle":
      return "Parado";
    case "starting":
      return "A iniciar";
    case "running":
      return "A correr";
    case "stopping":
      return "A parar";
    case "error":
      return "Erro";
    default:
      return assertNever(state);
  }
}

export function GameCard({
  game,
  launchState,
}: {
  game: Game;
  launchState: LaunchState;
}) {
  return (
    <Link to={`/games/${game.id}`} className="block h-full">
      <Card className="h-full transition-colors hover:border-primary/50">
        <CardHeader>
          <div className="flex items-start justify-between gap-2">
            <CardTitle className="text-base">{game.name}</CardTitle>
            <Badge variant={launchState === "running" ? "default" : "secondary"}>
              {statusLabel(launchState)}
            </Badge>
          </div>
          <CardDescription className="break-all">{game.exePath}</CardDescription>
          <div className="flex flex-wrap gap-1.5 pt-1">
            <Badge variant="outline">{runtimeKindLabel(game.runtimeKind)}</Badge>
            <Badge variant="outline">{dxBackendLabel(game.dxBackend)}</Badge>
            <Badge variant="outline">{prefixStatusLabel(game.prefixStatus)}</Badge>
          </div>
        </CardHeader>
      </Card>
    </Link>
  );
}
