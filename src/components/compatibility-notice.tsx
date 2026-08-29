import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Info } from "lucide-react";

export function CompatibilityNotice({ compact = false }: { compact?: boolean }) {
  return (
    <Alert>
      <Info />
      <AlertTitle>Nem todo jogo Windows abre no Mac</AlertTitle>
      <AlertDescription>
        {compact
          ? "Anti-cheat de kernel, DRM e alguns DirectX ainda barram títulos. Trocar o motor pode ajudar; não é garantia."
          : "Anti-cheat de kernel (Easy Anti-Cheat, BattlEye, Vanguard), DRM e DirectX que o tradutor não cobre costumam falhar. Muitos .exe 32-bit também. O log mostra o erro; outro motor pode ou não resolver."}
      </AlertDescription>
    </Alert>
  );
}
