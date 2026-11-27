import {
  BarChart3,
  CircleHelp,
  FolderOpen,
  HeartHandshake,
  Home,
  Info,
  Mail,
  Receipt,
  Settings,
  Shield,
  Users,
  Wallet,
  type LucideIcon,
} from "lucide-react";

export type AppSection =
  | "home"
  | "invoices"
  | "providers"
  | "payments"
  | "documents"
  | "reports"
  | "settings"
  | "about"
  | "support"
  | "help"
  | "contact"
  | "privacy";

export type NavItem = {
  id: AppSection;
  label: string;
  icon: LucideIcon;
  description: string;
};

/** Navegación principal (sin Configuración: va al pie de la sidebar). */
export const NAV_ITEMS: NavItem[] = [
  { id: "home", label: "Inicio", icon: Home, description: "Resumen general" },
  { id: "invoices", label: "Facturas", icon: Receipt, description: "Importar y buscar" },
  { id: "providers", label: "Proveedores", icon: Users, description: "CUIT y alias" },
  { id: "payments", label: "Pagos", icon: Wallet, description: "Pendientes y vencimientos" },
  { id: "documents", label: "Documentos", icon: FolderOpen, description: "Archivo y visor" },
  { id: "reports", label: "Informes", icon: BarChart3, description: "Exportar CSV" },
];

export const FOOTER_NAV_ITEMS: NavItem[] = [
  { id: "settings", label: "Configuración", icon: Settings, description: "Tema, entidades y backup" },
  { id: "about", label: "Acerca de Folio", icon: Info, description: "Filosofía y versión" },
];

export const HELP_MENU_ITEMS: { id: AppSection; label: string; icon: LucideIcon }[] = [
  { id: "help", label: "Centro de ayuda", icon: CircleHelp },
  { id: "contact", label: "Contacto y sugerencias", icon: Mail },
  { id: "about", label: "Acerca de Folio", icon: Info },
  { id: "support", label: "Apoyar Folio", icon: HeartHandshake },
  { id: "privacy", label: "Privacidad", icon: Shield },
];

/** Modo "solo facturas": oculta módulos más contables (pagos/informes). */
export const INVOICES_ONLY_SECTIONS: AppSection[] = [
  "home",
  "invoices",
  "providers",
  "documents",
  "settings",
  "about",
  "support",
  "help",
  "contact",
  "privacy",
];

export function navItemsForMode(mode: "invoices" | "management"): NavItem[] {
  if (mode === "management") return NAV_ITEMS;
  return NAV_ITEMS.filter((item) => INVOICES_ONLY_SECTIONS.includes(item.id));
}

export const SECTION_TITLES: Record<AppSection, { title: string; subtitle: string }> = {
  home: {
    title: "Inicio",
    subtitle: "Tus documentos, bajo control.",
  },
  invoices: {
    title: "Facturas",
    subtitle: "Importá, buscá y organizá comprobantes.",
  },
  providers: {
    title: "Proveedores",
    subtitle: "Directorio por entidad con CUIT y alias.",
  },
  payments: {
    title: "Pagos",
    subtitle: "Pendientes, parciales y vencimientos.",
  },
  documents: {
    title: "Documentos",
    subtitle: "Archivo local y visualización de PDF.",
  },
  reports: {
    title: "Informes",
    subtitle: "Exportá resultados a CSV para Excel.",
  },
  settings: {
    title: "Configuración",
    subtitle: "Tema, entidades, rutas y copias de seguridad.",
  },
  about: {
    title: "Acerca de Folio",
    subtitle: "Filosofía, versión y enlaces del proyecto.",
  },
  support: {
    title: "Apoyar Folio",
    subtitle: "Colaboración voluntaria para sostener el proyecto.",
  },
  help: {
    title: "Centro de ayuda",
    subtitle: "Guías rápidas para usar Folio.",
  },
  contact: {
    title: "Contacto y sugerencias",
    subtitle: "Errores, ideas y consultas.",
  },
  privacy: {
    title: "Privacidad",
    subtitle: "Cómo Folio trata tu información.",
  },
};
