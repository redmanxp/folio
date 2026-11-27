import { openUrl } from "@tauri-apps/plugin-opener";
import { FOLIO_LINKS, type FolioLinkKey, hasLink } from "@/content/folioMeta";

export async function openFolioLink(key: FolioLinkKey): Promise<boolean> {
  if (!hasLink(key)) return false;
  const url = FOLIO_LINKS[key];
  try {
    await openUrl(url);
    return true;
  } catch (err) {
    console.warn("No se pudo abrir enlace externo", key, err);
    return false;
  }
}

export async function openExternalUrl(url: string): Promise<boolean> {
  if (!url.trim()) return false;
  try {
    await openUrl(url);
    return true;
  } catch (err) {
    console.warn("No se pudo abrir URL", err);
    return false;
  }
}
