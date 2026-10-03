import { useEffect, useRef, useState } from "react";
import { Check } from "lucide-react";
import { SystemDiagnosticsCard } from "@/components/system-diagnostics-card";
import { RuntimeBanner } from "@/components/runtime-banner";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { usePonte } from "@/context/ponte-context";
import { runtimeCanAutoInstall } from "@/lib/game-setup";
import { isTauri } from "@/lib/ponte";
import type { RuntimeKind } from "@/lib/types";

export function SettingsPage() {
  const {
    settings,
    runtimes,
    foundRuntimes,
    saveSettings,
    actionError,
    pickWine,
    pickDirectory,
    installRuntime,
    systemDiagnostics,
    getSystemDiagnostics,
  } = usePonte();
  const [installing, setInstalling] = useState<RuntimeKind | null>(null);
  const [customWinePath, setCustomWinePath] = useState(settings.customWinePath);
  const [prefixesDir, setPrefixesDir] = useState(settings.prefixesDir);
  const [metalHud, setMetalHud] = useState(settings.metalHud);
  const [saved, setSaved] = useState(false);
  const wineInputRef = useRef<HTMLInputElement>(null);
  const prefixInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setCustomWinePath(settings.customWinePath);
    setPrefixesDir(settings.prefixesDir);
    setMetalHud(settings.metalHud);
  }, [settings]);

  function markDirty() {
    setSaved(false);
  }

  async function chooseWine() {
    const path = await pickWine();
    if (path) {
      markDirty();
      setCustomWinePath(path);
      return;
    }
    if (!isTauri()) {
      wineInputRef.current?.focus();
    }
  }

  async function choosePrefixDir() {
    const path = await pickDirectory();
    if (path) {
      markDirty();
      setPrefixesDir(path);
      return;
    }
    if (!isTauri()) {
      prefixInputRef.current?.focus();
    }
  }

  async function handleSave(event: React.FormEvent) {
    event.preventDefault();
    try {
      await saveSettings({ customWinePath, prefixesDir, metalHud });
      setSaved(true);
    } catch {
      setSaved(false);
    }
  }

  return (
    <form onSubmit={(event) => void handleSave(event)} className="mx-auto flex max-w-2xl flex-col gap-6">
      <div className="space-y-1">
        <h1 className="font-heading text-2xl font-semibold">Ajustes</h1>
        <p className="text-sm text-muted-foreground">
          O Ponte não empacota Wine nem D3DMetal. Ele detecta o que já está no
          Mac, instala o Wine Staging gerenciado se faltar, valida o runtime e
          dispara o .exe local.
        </p>
      </div>

      <RuntimeBanner foundCount={foundRuntimes.length} />

      <SystemDiagnosticsCard
        diagnostics={systemDiagnostics}
        onRefresh={() => {
          void getSystemDiagnostics();
        }}
      />

      <section className="space-y-3">
        <h2 className="font-heading text-sm font-medium">Motores detectados</h2>
        <ul className="grid gap-2">
          {runtimes.map((runtime) => (
            <li
              key={runtime.kind}
              className="flex flex-col gap-1 rounded-lg border p-3 sm:flex-row sm:items-center sm:justify-between"
            >
              <div>
                <p className="text-sm font-medium">{runtime.label}</p>
                <p className="break-all text-xs text-muted-foreground">
                  {runtime.found
                    ? runtime.wine64Path
                    : runtime.kind === "gptk"
                      ? "Wine x64 + payload D3DMetal não validados"
                      : "Não encontrado"}
                </p>
              </div>
              <div className="flex items-center gap-2">
                {!runtime.found && runtimeCanAutoInstall(runtime.kind) ? (
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    disabled={installing !== null}
                    onClick={() => {
                      void (async () => {
                        setInstalling(runtime.kind);
                        try {
                          await installRuntime(runtime.kind);
                        } catch {
                          /* mensagem já vai em actionError */
                        } finally {
                          setInstalling(null);
                        }
                      })();
                    }}
                  >
                    {installing === runtime.kind ? "A instalar…" : "Instalar"}
                  </Button>
                ) : null}
                {!runtime.found &&
                !runtimeCanAutoInstall(runtime.kind) &&
                runtime.installUrl ? (
                  <Button asChild type="button" variant="outline" size="sm">
                    <a href={runtime.installUrl} target="_blank" rel="noreferrer">
                      Obter
                    </a>
                  </Button>
                ) : null}
                <Badge variant={runtime.found ? "default" : "secondary"}>
                  {runtime.found
                    ? runtime.kind === "gptk"
                      ? "Validado"
                      : "Encontrado"
                    : "Ausente"}
                </Badge>
              </div>
            </li>
          ))}
        </ul>
      </section>

      <div className="grid gap-2">
        <Label htmlFor="wine-path">Caminho manual do Wine</Label>
        <div className="flex gap-2">
          <Input
            id="wine-path"
            ref={wineInputRef}
            value={customWinePath}
            onChange={(event) => {
              markDirty();
              setCustomWinePath(event.target.value);
            }}
            placeholder="/opt/homebrew/bin/wine"
          />
          <Button type="button" variant="outline" onClick={() => void chooseWine()}>
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
        <Label htmlFor="prefix-dir">Pasta de prefixos</Label>
        <div className="flex gap-2">
          <Input
            id="prefix-dir"
            ref={prefixInputRef}
            value={prefixesDir}
            onChange={(event) => {
              markDirty();
              setPrefixesDir(event.target.value);
            }}
          />
          <Button type="button" variant="outline" onClick={() => void choosePrefixDir()}>
            Escolher
          </Button>
        </div>
        <p className="text-xs text-muted-foreground">
          Jogos já adicionados mantêm o prefixo antigo. A nova pasta vale para os
          próximos.
        </p>
      </div>

      <div className="flex items-center justify-between gap-4 rounded-lg border p-3">
        <div>
          <Label htmlFor="hud">HUD Metal</Label>
          <p className="text-xs text-muted-foreground">
            Define MTL_HUD_ENABLED no lançamento.
          </p>
        </div>
        <Switch
          id="hud"
          checked={metalHud}
          onCheckedChange={(checked) => {
            markDirty();
            setMetalHud(checked);
          }}
        />
      </div>

      {actionError ? <p className="text-sm text-destructive">{actionError}</p> : null}
      {saved ? (
        <Alert>
          <Check />
          <AlertTitle>Salvo</AlertTitle>
          <AlertDescription>
            Os ajustes ficam neste aparelho. No Mac, o próximo lançamento já usa
            o wine64 e o HUD que você escolheu.
          </AlertDescription>
        </Alert>
      ) : null}

      <div className="flex items-center gap-3">
        <Button type="submit">Guardar</Button>
      </div>
    </form>
  );
}
