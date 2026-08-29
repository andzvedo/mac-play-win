import { useEffect, useState } from "react";
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { usePonte } from "@/context/ponte-context";
import { isPonteError } from "@/lib/errors";
import { catalogEntry, dxBackendLabel } from "@/lib/runtime-catalog";
import { isTauri } from "@/lib/ponte";
import type { DxBackend, RuntimeKind } from "@/lib/types";

interface AddGameDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function AddGameDialog({ open, onOpenChange }: AddGameDialogProps) {
  const { foundRuntimes, addGame, pickExe } = usePonte();
  const navigate = useNavigate();
  const defaultRuntime = foundRuntimes[0]?.kind ?? "gptk";
  const [name, setName] = useState("");
  const [exePath, setExePath] = useState("");
  const [args, setArgs] = useState("");
  const [runtimeKind, setRuntimeKind] = useState<RuntimeKind>(defaultRuntime);
  const [dxBackend, setDxBackend] = useState<DxBackend>(
    catalogEntry(defaultRuntime).defaultBackend,
  );
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  useEffect(() => {
    if (!open) {
      return;
    }
    const kind = foundRuntimes[0]?.kind ?? "gptk";
    setName("");
    setExePath("");
    setArgs("");
    setRuntimeKind(kind);
    setDxBackend(catalogEntry(kind).defaultBackend);
    setError(null);
  }, [open, foundRuntimes]);

  const supported = catalogEntry(runtimeKind).supportedBackends;
  const motorOptions =
    foundRuntimes.length > 0
      ? foundRuntimes.map((runtime) => ({
          kind: runtime.kind,
          label: runtime.label,
        }))
      : [{ kind: "gptk" as const, label: catalogEntry("gptk").label }];

  async function handlePick() {
    const path = await pickExe();
    if (path) {
      setExePath(path);
    }
  }

  async function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    setPending(true);
    setError(null);
    try {
      const game = await addGame({
        name,
        exePath,
        args,
        runtimeKind,
        dxBackend,
      });
      onOpenChange(false);
      navigate(`/games/${game.id}`);
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
              Aponte um .exe que já está no disco. O Ponte não baixa jogos e não
              é uma loja.
            </DialogDescription>
          </DialogHeader>
          <div className="grid gap-2">
            <Label htmlFor="game-name">Nome</Label>
            <Input
              id="game-name"
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder="Hades"
              required
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="game-exe">Caminho do .exe</Label>
            <div className="flex gap-2">
              <Input
                id="game-exe"
                value={exePath}
                onChange={(event) => setExePath(event.target.value)}
                placeholder="/Games/Hades/Hades.exe"
                required
              />
              <Button type="button" variant="outline" onClick={() => void handlePick()}>
                Escolher
              </Button>
            </div>
            {!isTauri() ? (
              <p className="text-xs text-muted-foreground">
                Neste navegador cole o caminho. No app do Mac, Escolher abre o
                diálogo nativo.
              </p>
            ) : null}
          </div>
          <div className="grid gap-2">
            <Label htmlFor="game-args">Argumentos (opcional)</Label>
            <Input
              id="game-args"
              value={args}
              onChange={(event) => setArgs(event.target.value)}
              placeholder="-windowed"
            />
          </div>
          <div className="grid gap-2">
            <Label>Motor</Label>
            <Select
              value={runtimeKind}
              onValueChange={(value) => {
                const kind = value as RuntimeKind;
                setRuntimeKind(kind);
                const next = catalogEntry(kind);
                if (!next.supportedBackends.includes(dxBackend)) {
                  setDxBackend(next.defaultBackend);
                }
              }}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {motorOptions.map((runtime) => (
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
              value={dxBackend}
              onValueChange={(value) => setDxBackend(value as DxBackend)}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {supported.map((backend) => (
                  <SelectItem key={backend} value={backend}>
                    {dxBackendLabel(backend)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          {error ? <p className="text-sm text-destructive">{error}</p> : null}
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancelar
            </Button>
            <Button type="submit" disabled={pending}>
              {pending ? "A guardar…" : "Adicionar"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
