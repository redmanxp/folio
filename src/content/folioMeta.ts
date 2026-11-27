/** Metadatos institucionales y enlaces públicos del proyecto. */
export const FOLIO_META = {
  name: "Folio",
  tagline: "Organizá tus documentos. Simplificá tu día.",
  slogan: "Tus documentos, bajo control.",
  versionLabel: "0.1.1",
  license: "MIT",
  platform: "Windows",
} as const;

/**
 * Enlaces externos oficiales.
 * Si un campo está vacío, la UI muestra “próximamente”.
 */
export const FOLIO_LINKS = {
  website: "https://redmanxp.github.io/folio/",
  github: "https://github.com/redmanxp/folio",
  githubIssues: "https://github.com/redmanxp/folio/issues/new/choose",
  githubDiscussions: "https://github.com/redmanxp/folio/discussions",
  email: "folioapp.dev@gmail.com",
  mercadopago: "https://link.mercadopago.com.ar/folioapp",
  privacy: "https://redmanxp.github.io/folio/#privacidad",
  license: "https://github.com/redmanxp/folio/blob/main/LICENSE",
  downloads: "https://github.com/redmanxp/folio/releases",
} as const;

export type FolioLinkKey = keyof typeof FOLIO_LINKS;

export function hasLink(key: FolioLinkKey): boolean {
  return Boolean(FOLIO_LINKS[key]?.trim());
}
