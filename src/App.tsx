import { Route, Routes } from "react-router-dom";
import { TooltipProvider } from "@/components/ui/tooltip";
import { AppShell } from "@/components/app-shell";
import { PonteProvider } from "@/context/ponte-context";
import { GameDetailPage } from "@/pages/game-detail-page";
import { LibraryPage } from "@/pages/library-page";
import { SettingsPage } from "@/pages/settings-page";

export default function App() {
  return (
    <TooltipProvider>
      <PonteProvider>
        <Routes>
          <Route element={<AppShell />}>
            <Route path="/" element={<LibraryPage />} />
            <Route path="/games/:id" element={<GameDetailPage />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Route>
        </Routes>
      </PonteProvider>
    </TooltipProvider>
  );
}
