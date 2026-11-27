import { BrandLogo } from "@/components/brand/BrandLogo";
import { EntitySwitcher } from "@/components/layout/EntitySwitcher";
import { useAppShell } from "@/app/AppShellContext";
import { FOOTER_NAV_ITEMS, navItemsForMode } from "@/app/navigation";
import { FOLIO_META } from "@/content/folioMeta";
import { cn } from "@/lib/utils";
import { useEffect, useState } from "react";
import { api } from "@/api";

export function Sidebar() {
  const { section, setSection, workspaceMode } = useAppShell();
  const items = navItemsForMode(workspaceMode);
  const [version, setVersion] = useState<string>(FOLIO_META.versionLabel);

  useEffect(() => {
    void api.getAppInfo().then((info) => setVersion(info.version)).catch(() => undefined);
  }, []);

  return (
    <aside className="flex h-full w-[272px] shrink-0 flex-col border-r border-white/5 bg-sidebar text-sidebar-foreground">
      <div className="flex items-center gap-3 px-5 py-5">
        <BrandLogo variant="icon" className="h-10 w-10 drop-shadow-sm" />
        <div>
          <p className="text-lg font-semibold tracking-tight text-white">
            Fol
            <span className="relative inline-block">
              i
              <span className="absolute left-1/2 top-[0.12em] h-[0.28em] w-[0.28em] -translate-x-1/2 rounded-full bg-brand-green shadow-[0_0_0_2px_#0f1c33]" />
            </span>
            o
          </p>
          <p className="mt-1 text-xs text-sidebar-muted">Gestión de documentos</p>
        </div>
      </div>

      <div className="px-4 pb-4">
        <EntitySwitcher />
      </div>

      <nav className="flex-1 space-y-1 overflow-y-auto px-3 pb-4">
        {items.map((item) => {
          const Icon = item.icon;
          const active = section === item.id;
          return (
            <button
              key={item.id}
              type="button"
              onClick={() => setSection(item.id)}
              className={cn(
                "group flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left transition",
                active
                  ? "bg-brand-blue text-white shadow-sm"
                  : "text-sidebar-foreground/85 hover:bg-sidebar-hover",
              )}
            >
              <Icon className={cn("h-4 w-4", active ? "text-white" : "text-sidebar-muted")} />
              <span className="text-sm font-medium">{item.label}</span>
            </button>
          );
        })}
      </nav>

      <div className="space-y-1 border-t border-white/5 px-3 py-3">
        {FOOTER_NAV_ITEMS.map((item) => {
          const Icon = item.icon;
          const active = section === item.id;
          return (
            <button
              key={item.id}
              type="button"
              onClick={() => setSection(item.id)}
              className={cn(
                "flex w-full items-center gap-3 rounded-xl px-3 py-2 text-left transition",
                active
                  ? "bg-white/10 text-white"
                  : "text-sidebar-foreground/75 hover:bg-sidebar-hover hover:text-sidebar-foreground",
              )}
            >
              <Icon className={cn("h-4 w-4", active ? "text-white" : "text-sidebar-muted")} />
              <span className="text-sm">{item.label}</span>
            </button>
          );
        })}
        <p className="px-3 pt-2 text-[11px] text-sidebar-muted">
          Folio v{version}
        </p>
      </div>
    </aside>
  );
}
