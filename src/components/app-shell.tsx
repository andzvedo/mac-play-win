import { useState, type ReactNode } from "react";
import { NavLink, Outlet } from "react-router-dom";
import { Library, Menu, Settings, Wine } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { cn } from "@/lib/utils";

function NavItems({ onNavigate }: { onNavigate?: () => void }) {
  const linkClass = ({ isActive }: { isActive: boolean }) =>
    cn(
      "flex items-center gap-2 rounded-lg px-3 py-2 text-sm transition-colors",
      isActive
        ? "bg-sidebar-accent text-sidebar-accent-foreground"
        : "text-sidebar-foreground/80 hover:bg-sidebar-accent/70",
    );

  return (
    <nav className="grid gap-1">
      <NavLink to="/" end className={linkClass} onClick={onNavigate}>
        <Library className="size-4" />
        Biblioteca
      </NavLink>
      <NavLink to="/settings" className={linkClass} onClick={onNavigate}>
        <Settings className="size-4" />
        Ajustes
      </NavLink>
    </nav>
  );
}

function Brand() {
  return (
    <div className="flex items-center gap-2 px-3 py-2">
      <div className="flex size-9 items-center justify-center rounded-lg bg-primary text-primary-foreground">
        <Wine className="size-4" />
      </div>
      <div>
        <p className="font-heading text-sm font-semibold">Ponte</p>
        <p className="text-xs text-muted-foreground">Jogos Windows no Mac</p>
      </div>
    </div>
  );
}

export function AppShell({ children }: { children?: ReactNode }) {
  const [open, setOpen] = useState(false);

  return (
    <div className="flex h-full min-h-0 overflow-hidden bg-background">
      <aside className="hidden h-full w-60 shrink-0 border-r border-sidebar-border bg-sidebar md:flex md:flex-col">
        <Brand />
        <Separator />
        <div className="min-h-0 flex-1 overflow-y-auto p-3">
          <NavItems />
        </div>
      </aside>
      <div className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
        <header className="flex shrink-0 items-center gap-2 border-b px-4 py-3 md:hidden">
          <Sheet open={open} onOpenChange={setOpen}>
            <SheetTrigger asChild>
              <Button variant="outline" size="icon-sm" aria-label="Abrir menu">
                <Menu />
              </Button>
            </SheetTrigger>
            <SheetContent side="left" className="w-64 bg-sidebar p-0">
              <SheetHeader>
                <SheetTitle className="sr-only">Navegação</SheetTitle>
              </SheetHeader>
              <Brand />
              <div className="p-3">
                <NavItems onNavigate={() => setOpen(false)} />
              </div>
            </SheetContent>
          </Sheet>
          <span className="font-heading text-sm font-semibold">Ponte</span>
        </header>
        <main className="min-h-0 min-w-0 flex-1 overflow-x-hidden overflow-y-auto overscroll-y-contain [overflow-anchor:none] p-4 md:p-8">
          {children ?? <Outlet />}
        </main>
      </div>
    </div>
  );
}
