import { useEffect, useRef, useState } from "react";
import { FolderOpen } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { usePonte } from "@/context/ponte-context";
import { engineGuessLabel } from "@/lib/game-setup";
import { isPonteError } from "@/lib/errors";
import { isTauri } from "@/lib/ponte";
import { dxBackendLabel, runtimeKindLabel } from "@/lib/runtime-catalog";
import type { GameDiscovery } from "@/lib/types";

interface AddGameDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

function DiscoverySummary({ discovery }: { discovery: GameDiscovery }) {
  return (
    <dl className="grid gap-2 rounded-lg border p-3 text-sm">
      <div className="grid gap-0.5">
        <dt className="text-xs text-muted-foreground">Nome</dt>
        <dd className="font-medium">{discovery.suggestedName}</dd>
      </div>
      <div className="grid gap-0.5">
        <dt className="text-xs text-muted-foreground">Executável</dt>
        <dd className="break-all text-xs text-muted-foreground">{discovery.exePath}</dd>
      </div>
      <div className="grid gap-0.5 sm:grid-cols-2 sm:gap-3">
        <div>
          <dt className="text-xs text-muted-foreground">Motor do jogo</dt>
          <dd>{engineGuessLabel(discovery.engineGuess)}</dd>
        </div>
        <div>
          <dt className="text-xs text-muted-foreground">Wine e DirectX</dt>
          <dd>
            {runtimeKindLabel(discovery.recommendedRuntime)} +{" "}
            {dxBackendLabel(discovery.recommendedBackend)}
          </dd>
        </div>
      </div>
    </dl>
  );
}

export function AddGameDialog({ open, onOpenChange }: AddGameDialogProps) {
  const { addGame, pickGameFolder, resolveGameFolder } = usePonte();
  const navigate = useNavigate();
  const [folderPath, setFolderPath] = useState("");
  const [discovery, setDiscovery] = useState<GameDiscovery | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [resolving, setResolving] = useState(false);
  const folderInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!open) {
      return;
    }
    setFolderPath("");
    setDiscovery(null);
    setError(null);
  }, [open]);

  async function applyFolder(path: string) {
    const trimmed = path.trim();
    setFolderPath(trimmed);
    if (!trimmed) {
      setDiscovery(null);
      return;
    }
    setResolving(true);
    setError(null);
    try {
      const found = await resolveGameFolder(trimmed);
      setDiscovery(found);
    } catch (err) {
      setDiscovery(null);
      setError(
        isPonteError(err)
          ? err.message
          : "Não foi possível encontrar o jogo nesta pasta.",
      );
    } finally {
      setResolving(false);
    }
  }

  async function handlePick() {
    const path = await pickGameFolder();
    if (path) {
      await applyFolder(path);
      return;
    }
    if (!isTauri()) {
      folderInputRef.current?.focus();
    }
  }

  async function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    setError(null);
    if (!folderPath.trim()) {
      setError("Escolha a pasta onde o jogo está no disco.");
      return;
    }
    setPending(true);
    try {
      const game = await addGame({
        name: discovery?.suggestedName ?? "",
        folderPath,
        args: "",
      });
      onOpenChange(false);
      navigate(`/games/${game.id}?preparar=1`);
    } catch (err) {
      setError(isPonteError(err) ? err.message : "Não foi possível adicionar o jogo.");
    } finally {
      setPending(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <form onSubmit={handleSubmit} className="grid gap-4">
          <DialogHeader>
            <DialogTitle>Adicionar jogo local</DialogTitle>
            <DialogDescription>
              Escolha a pasta do jogo. O Ponte encontra o .exe, o nome e o
              Wine/DirectX certos, e instala o que faltar. Não baixa jogos.
            </DialogDescription>
          </DialogHeader>
          <div className="grid gap-2">
            <Label htmlFor="game-folder">Pasta do jogo</Label>
            {isTauri() ? (
              <Button
                type="button"
                variant={folderPath ? "outline" : "default"}
                onClick={() => void handlePick()}
              >
                <FolderOpen />
                {folderPath ? "Trocar pasta" : "Escolher pasta"}
              </Button>
            ) : (
              <div className="flex gap-2">
                <Input
                  id="game-folder"
                  ref={folderInputRef}
                  value={folderPath}
                  onChange={(event) => {
                    setFolderPath(event.target.value);
                    setDiscovery(null);
                  }}
                  onBlur={() => {
                    if (folderPath.trim()) {
                      void applyFolder(folderPath);
                    }
                  }}
                  placeholder="/Games/Hades"
                  required
                />
                <Button type="button" variant="outline" onClick={() => void handlePick()}>
                  Escolher
                </Button>
              </div>
            )}
            {folderPath && isTauri() ? (
              <p className="break-all text-xs text-muted-foreground">{folderPath}</p>
            ) : null}
            {!isTauri() ? (
              <p className="text-xs text-muted-foreground">
                Neste navegador cole o caminho da pasta. No app do Mac,
                Escolher abre o diálogo nativo.
              </p>
            ) : null}
            {resolving ? (
              <p className="text-xs text-muted-foreground">A procurar o executável…</p>
            ) : null}
            {discovery ? <DiscoverySummary discovery={discovery} /> : null}
          </div>
          {error ? <p className="text-sm text-destructive">{error}</p> : null}
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancelar
            </Button>
            <Button type="submit" disabled={pending || resolving || !folderPath.trim()}>
              {pending ? "A guardar…" : "Adicionar"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
