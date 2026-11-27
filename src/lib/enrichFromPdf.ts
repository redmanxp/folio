import { api } from "@/api";
import { extractPdfText, needsTextEnrichment } from "@/lib/pdfText";
import type { Document } from "@/types";

/**
 * Si el backend no completó monto/vencimiento (p.ej. AySA con fuentes custom),
 * lee el texto con PDF.js (mismo motor que el visor) y enriquece la DB.
 */
export async function enrichDocumentWithPdfJs(doc: Document): Promise<Document> {
  if (!needsTextEnrichment(doc)) return doc;
  const path = doc.storedPath;
  if (!path.toLowerCase().endsWith(".pdf")) return doc;

  try {
    const text = await extractPdfText(path);
    if (text.length < 40) return doc;
    return await api.enrichDocumentFromText(doc.id, text);
  } catch (err) {
    console.warn("enrichDocumentWithPdfJs falló", doc.fileName, err);
    return doc;
  }
}

export async function enrichDocumentsWithPdfJs(ids: string[]): Promise<number> {
  let enriched = 0;
  for (const id of ids) {
    try {
      const doc = await api.getDocument(id);
      if (!needsTextEnrichment(doc)) continue;
      const before = `${doc.amount}|${doc.dueDate}|${doc.number}`;
      const updated = await enrichDocumentWithPdfJs(doc);
      const after = `${updated.amount}|${updated.dueDate}|${updated.number}`;
      if (after !== before) enriched += 1;
    } catch {
      // seguir con el resto
    }
  }
  return enriched;
}
