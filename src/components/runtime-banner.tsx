import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { TriangleAlert } from "lucide-react";
import { catalogEntry } from "@/lib/runtime-catalog";

export function RuntimeBanner({ foundCount }: { foundCount: number }) {
  if (foundCount > 0) {
    return null;
  }

  const gptk = catalogEntry("gptk");
  const gcenx = catalogEntry("gcenx_staging");

  return (
    <Alert variant="destructive">
      <TriangleAlert />
      <AlertTitle>Nenhum wine64 encontrado</AlertTitle>
      <AlertDescription>
        O Ponte não instala o Wine. No Mac, instale o{" "}
        <a href={gptk.installUrl ?? "#"} target="_blank" rel="noreferrer">
          Game Porting Toolkit
        </a>{" "}
        ou o{" "}
        <a href={gcenx.installUrl ?? "#"} target="_blank" rel="noreferrer">
          Wine Staging Gcenx
        </a>
        , depois aponte o binário em Ajustes. Neste navegador a detecção é simulada.
      </AlertDescription>
    </Alert>
  );
}
