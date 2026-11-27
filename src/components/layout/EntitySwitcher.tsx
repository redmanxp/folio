import { Building2, ChevronDown, User } from "lucide-react";
import { useAppShell } from "@/app/AppShellContext";
import { cn } from "@/lib/utils";

export function EntitySwitcher() {
  const { entities, activeEntityId, setActiveEntityId, activeEntity, loading, setSection } =
    useAppShell();

  if (loading) {
    return (
      <div className="rounded-xl border border-white/10 bg-white/5 px-3 py-3 text-xs text-sidebar-muted">
        Cargando entidades…
      </div>
    );
  }

  if (entities.length === 0) {
    return (
      <div className="space-y-2">
        <p className="px-1 text-[11px] font-semibold uppercase tracking-[0.14em] text-sidebar-muted">
          Espacio de trabajo
        </p>
        <button
          type="button"
          onClick={() => setSection("settings")}
          className="w-full rounded-xl border border-dashed border-white/20 bg-white/5 px-3 py-3 text-left text-sm text-sidebar-foreground hover:bg-white/10"
        >
          Creá tu primera entidad
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <p className="px-1 text-[11px] font-semibold uppercase tracking-[0.14em] text-sidebar-muted">
        Espacio de trabajo
      </p>
      <div className="relative">
        <div className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-sidebar-muted">
          {activeEntity?.kind === "personal" ? (
            <User className="h-4 w-4" />
          ) : (
            <Building2 className="h-4 w-4" />
          )}
        </div>
        <select
          aria-label="Selector de entidad"
          className={cn(
            "h-11 w-full appearance-none rounded-xl border border-white/10 bg-white/5 pl-10 pr-9 text-sm text-sidebar-foreground outline-none transition",
            "hover:bg-white/8 focus-visible:ring-2 focus-visible:ring-brand-blue-light/40",
          )}
          value={activeEntityId ?? ""}
          onChange={(event) => setActiveEntityId(event.target.value || null)}
        >
          {entities.map((entity) => (
            <option key={entity.id} value={entity.id} className="bg-sidebar text-foreground">
              {entity.name}
            </option>
          ))}
        </select>
        <ChevronDown className="pointer-events-none absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2 text-sidebar-muted" />
      </div>
    </div>
  );
}
