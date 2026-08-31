import { useState } from "react";
import { Plus, Search, Wine } from "lucide-react";
import { AddGameDialog } from "@/components/add-game-dialog";
import { CompatibilityNotice } from "@/components/compatibility-notice";
import { GameCard } from "@/components/game-card";
import { RuntimeBanner } from "@/components/runtime-banner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { usePonte } from "@/context/ponte-context";
import { filterGames } from "@/lib/filter-games";

export function LibraryPage() {
  const { games, foundRuntimes, launch, loading, actionError } = usePonte();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const visible = filterGames(games, query);

  return (
    <div className="mx-auto flex max-w-5xl flex-col gap-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div className="space-y-1">
          <h1 className="font-heading text-2xl font-semibold tracking-tight">
            Biblioteca
          </h1>
          <p className="max-w-xl text-sm text-muted-foreground">
            Jogos Windows que já estão no disco. Escolha a pasta do título; o
            Ponte encontra o .exe, cria o prefixo Wine e instala o que faltar.
          </p>
        </div>
        <Button onClick={() => setOpen(true)}>
          <Plus />
          Adicionar jogo
        </Button>
      </div>

      <RuntimeBanner foundCount={foundRuntimes.length} />
      {actionError ? (
        <p className="text-sm text-destructive">{actionError}</p>
      ) : null}

      {loading && games.length === 0 ? (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3" aria-busy="true">
          <div className="h-36 animate-pulse rounded-xl border bg-muted/40" />
          <div className="h-36 animate-pulse rounded-xl border bg-muted/40" />
          <div className="h-36 animate-pulse rounded-xl border bg-muted/40" />
        </div>
      ) : null}

      {!loading && games.length === 0 ? (
        <div className="flex flex-col items-start gap-4 rounded-xl border border-dashed p-8">
          <div className="flex size-12 items-center justify-center rounded-full bg-primary/15 text-primary">
            <Wine className="size-6" />
          </div>
          <div className="space-y-2">
            <h2 className="font-heading text-lg font-medium">Nenhum jogo ainda</h2>
            <p className="max-w-lg text-sm text-muted-foreground">
              Adicione a pasta de um jogo local — cópia Steam/GOG, instalador já
              extraído ou dump que você comprou. O Ponte encontra o .exe e, se
              faltar Wine, instala o Staging gerenciado. Não é emulador, loja
              nem garantia de que o título abre.
            </p>
          </div>
          <Button onClick={() => setOpen(true)}>Adicionar jogo</Button>
          <CompatibilityNotice />
        </div>
      ) : (
        <>
          <div className="relative max-w-md">
            <Search className="pointer-events-none absolute top-2.5 left-2.5 size-4 text-muted-foreground" />
            <Input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Buscar por nome, caminho ou motor"
              className="pl-8"
              aria-label="Buscar jogos"
            />
          </div>
          {visible.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              Nenhum jogo corresponde a “{query.trim()}”.
            </p>
          ) : (
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {visible.map((game) => (
                <GameCard
                  key={game.id}
                  game={game}
                  launchState={launch[game.id] ?? "idle"}
                />
              ))}
            </div>
          )}
          <CompatibilityNotice compact />
        </>
      )}

      <AddGameDialog open={open} onOpenChange={setOpen} />
    </div>
  );
}
