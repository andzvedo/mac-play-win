import { useState } from "react";
import { Plus, Wine } from "lucide-react";
import { AddGameDialog } from "@/components/add-game-dialog";
import { CompatibilityNotice } from "@/components/compatibility-notice";
import { GameCard } from "@/components/game-card";
import { RuntimeBanner } from "@/components/runtime-banner";
import { Button } from "@/components/ui/button";
import { usePonte } from "@/context/ponte-context";

export function LibraryPage() {
  const { games, foundRuntimes, launch, loading, actionError } = usePonte();
  const [open, setOpen] = useState(false);

  return (
    <div className="mx-auto flex max-w-5xl flex-col gap-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div className="space-y-1">
          <h1 className="font-heading text-2xl font-semibold tracking-tight">
            Biblioteca
          </h1>
          <p className="max-w-xl text-sm text-muted-foreground">
            Jogos Windows que já estão no disco. O Ponte cria um prefixo Wine e
            dispara o .exe com o motor que você escolher.
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
        <p className="text-sm text-muted-foreground">A carregar a biblioteca…</p>
      ) : null}

      {!loading && games.length === 0 ? (
        <div className="flex flex-col items-start gap-4 rounded-xl border border-dashed p-8">
          <div className="flex size-12 items-center justify-center rounded-full bg-primary/15 text-primary">
            <Wine className="size-6" />
          </div>
          <div className="space-y-2">
            <h2 className="font-heading text-lg font-medium">Nenhum jogo ainda</h2>
            <p className="max-w-lg text-sm text-muted-foreground">
              Adicione um .exe local — pasta do jogo, instalador ou cópia Steam/GOG
              que você comprou. É preciso um wine64 no Mac (GPTK ou Gcenx). O Ponte
              não é emulador, loja nem garantia de que o título abre.
            </p>
          </div>
          <Button onClick={() => setOpen(true)}>Adicionar jogo</Button>
          <CompatibilityNotice />
        </div>
      ) : (
        <>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {games.map((game) => (
              <GameCard
                key={game.id}
                game={game}
                launchState={launch[game.id] ?? "idle"}
              />
            ))}
          </div>
          <CompatibilityNotice compact />
        </>
      )}

      <AddGameDialog open={open} onOpenChange={setOpen} />
    </div>
  );
}
