import { useEffect, useRef, useState, type FormEvent } from "react";
import { LoaderCircle, Monitor, Moon, Search, Sun, UserRound } from "lucide-react";
import { useAppShell } from "@/app/AppShellContext";
import { HELP_MENU_ITEMS, SECTION_TITLES } from "@/app/navigation";
import { HelpCircleIcon } from "@/components/brand/HelpCircleIcon";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Tooltip } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { useTheme, type ThemeMode } from "@/theme/ThemeProvider";

const themeCycle: ThemeMode[] = ["light", "dark", "system"];

export function Header() {
  const { section, setSection, activeEntity, error, setError, processing } = useAppShell();
  const { theme, setTheme } = useTheme();
  const [query, setQuery] = useState("");
  const [helpOpen, setHelpOpen] = useState(false);
  const helpRef = useRef<HTMLDivElement>(null);
  const meta = SECTION_TITLES[section];
  const ThemeIcon = theme === "dark" ? Moon : theme === "light" ? Sun : Monitor;

  useEffect(() => {
    if (!helpOpen) return;
    function onPointerDown(event: MouseEvent) {
      if (!helpRef.current?.contains(event.target as Node)) {
        setHelpOpen(false);
      }
    }
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") setHelpOpen(false);
    }
    document.addEventListener("mousedown", onPointerDown);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onPointerDown);
      document.removeEventListener("keydown", onKey);
    };
  }, [helpOpen]);

  function cycleTheme() {
    const index = themeCycle.indexOf(theme);
    setTheme(themeCycle[(index + 1) % themeCycle.length]);
  }

  function onSearch(event: FormEvent) {
    event.preventDefault();
    const value = query.trim();
    if (!value) {
      setSection("invoices");
      return;
    }
    sessionStorage.setItem("folio.searchQuery", value);
    setSection("invoices");
  }

  return (
    <header className="border-b border-border bg-card/70 backdrop-blur-md">
      {error ? (
        <div className="flex items-center justify-between gap-3 border-b border-danger/20 bg-danger/5 px-6 py-2 text-sm text-danger">
          <span>{error}</span>
          <button type="button" className="underline" onClick={() => setError(null)}>
            Cerrar
          </button>
        </div>
      ) : null}
      {processing.active ? (
        <div className="flex items-center gap-2 border-b border-brand-blue/20 bg-brand-blue/5 px-6 py-2 text-sm text-brand-navy dark:text-foreground">
          <LoaderCircle className="h-4 w-4 animate-spin text-brand-blue" />
          <span>{processing.label || "Procesando…"}</span>
          <span className="text-xs text-muted-foreground">Podés seguir navegando</span>
        </div>
      ) : null}
      <div className="flex items-center justify-between gap-4 px-6 py-4">
        <div className="min-w-0">
          <h1 className="truncate text-xl font-semibold tracking-tight text-foreground">
            {meta.title}
          </h1>
          <p className="truncate text-sm text-muted-foreground">
            {meta.subtitle}
            {activeEntity ? ` · ${activeEntity.name}` : ""}
          </p>
        </div>

        <div className="flex items-center gap-2">
          {processing.active ? (
            <div className="mr-1 hidden items-center gap-2 rounded-full border border-brand-blue/30 bg-brand-blue/5 px-3 py-1.5 text-xs font-medium text-brand-blue sm:flex">
              <LoaderCircle className="h-3.5 w-3.5 animate-spin" />
              Procesando
            </div>
          ) : null}
          <form onSubmit={onSearch} className="relative hidden w-64 md:block">
            <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              className="pl-9"
              placeholder="Buscar facturas…"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
            />
          </form>

          <Tooltip content={`Tema: ${theme}`}>
            <Button variant="outline" size="icon" onClick={cycleTheme} aria-label="Cambiar tema">
              <ThemeIcon className="h-4 w-4" />
            </Button>
          </Tooltip>

          <div className="relative" ref={helpRef}>
            <Tooltip content="Ayuda y manual de Folio">
              <Button
                variant="outline"
                size="icon"
                aria-label="Ayuda"
                aria-expanded={helpOpen}
                aria-haspopup="menu"
                onClick={() => setHelpOpen((open) => !open)}
              >
                <HelpCircleIcon className="h-4 w-4" />
              </Button>
            </Tooltip>
            {helpOpen ? (
              <div
                role="menu"
                className="absolute right-0 z-40 mt-2 w-64 overflow-hidden rounded-2xl border border-border bg-card p-1 shadow-[var(--shadow-soft)]"
              >
                <p className="px-3 py-2 text-xs font-medium text-muted-foreground">
                  Ayuda e información
                </p>
                {HELP_MENU_ITEMS.map((item) => {
                  const Icon = item.icon;
                  const active = section === item.id;
                  return (
                    <button
                      key={item.id}
                      type="button"
                      role="menuitem"
                      className={cn(
                        "flex w-full items-center gap-2 rounded-xl px-3 py-2 text-left text-sm transition hover:bg-muted",
                        active && "bg-muted font-medium",
                      )}
                      onClick={() => {
                        setSection(item.id);
                        setHelpOpen(false);
                      }}
                    >
                      <Icon className="h-4 w-4 text-muted-foreground" />
                      {item.label}
                    </button>
                  );
                })}
              </div>
            ) : null}
          </div>

          <div className="ml-1 flex items-center gap-2 rounded-full border border-border bg-card py-1 pl-1 pr-3">
            <div className="flex h-8 w-8 items-center justify-center rounded-full bg-brand-blue/10 text-brand-blue">
              <UserRound className="h-4 w-4" />
            </div>
            <div className="hidden sm:block">
              <p className="text-xs font-medium leading-none">Usuario local</p>
              <p className="mt-1 text-[11px] text-muted-foreground">Sin cuenta</p>
            </div>
          </div>
        </div>
      </div>
    </header>
  );
}
