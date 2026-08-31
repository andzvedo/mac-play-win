import { useState } from "react";
import { TriangleAlert } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { usePonte } from "@/context/ponte-context";
import { isTauri } from "@/lib/ponte";
import { catalogEntry } from "@/lib/runtime-catalog";

export function RuntimeBanner({ foundCount }: { foundCount: number }) {
  const { installRuntime } = usePonte();
  const [busy, setBusy] = useState(false);

  if (foundCount > 0) {
    return null;
  }

  const gptk = catalogEntry("gptk");
  const gcenx = catalogEntry("gcenx_staging");
  const winehq = catalogEntry("winehq");

  async function installWine() {
    setBusy(true);
    try {
      await installRuntime("gcenx_staging");
    } catch {
      /* mensagem já vai em actionError */
    } finally {
      setBusy(false);
    }
  }

  return (
    <Alert variant="destructive">
      <TriangleAlert />
      <AlertTitle>Nenhum Wine encontrado</AlertTitle>
      <AlertDescription className="grid gap-3">
        <p>
          O Ponte pode instalar o Wine Staging gerenciado. Também pode
          apontar o{" "}
          <a href={gptk.installUrl ?? "#"} target="_blank" rel="noreferrer">
            Game Porting Toolkit
          </a>
          , o{" "}
          <a href={gcenx.installUrl ?? "#"} target="_blank" rel="noreferrer">
            Wine Staging Gcenx
          </a>{" "}
          ou o{" "}
          <a href={winehq.installUrl ?? "#"} target="_blank" rel="noreferrer">
            WineHQ Stable
          </a>
          {isTauri()
            ? "."
            : ". Neste navegador a instalação e a detecção são simuladas."}
        </p>
        <Button
          type="button"
          variant="secondary"
          size="sm"
          className="w-fit"
          disabled={busy}
          onClick={() => void installWine()}
        >
          {busy ? "A instalar…" : "Instalar Wine Staging"}
        </Button>
      </AlertDescription>
    </Alert>
  );
}
