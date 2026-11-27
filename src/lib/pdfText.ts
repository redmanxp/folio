import { convertFileSrc } from "@tauri-apps/api/core";
import * as pdfjs from "pdfjs-dist";
// Worker empaquetado por Vite
import pdfWorker from "pdfjs-dist/build/pdf.worker.min.mjs?url";

pdfjs.GlobalWorkerOptions.workerSrc = pdfWorker;

/** Extrae texto de las primeras páginas de un PDF local (ruta de archivo). */
export async function extractPdfText(filePath: string, maxPages = 3): Promise<string> {
  const url = convertFileSrc(filePath);
  const loadingTask = pdfjs.getDocument({ url });
  const pdf = await loadingTask.promise;
  const pages = Math.min(pdf.numPages, maxPages);
  const parts: string[] = [];
  for (let i = 1; i <= pages; i++) {
    const page = await pdf.getPage(i);
    const content = await page.getTextContent();
    const strings = content.items
      .map((item) => ("str" in item ? String(item.str) : ""))
      .filter(Boolean);
    parts.push(strings.join(" "));
  }
  return parts.join("\n").trim();
}

export function needsTextEnrichment(doc: {
  amount?: number | null;
  dueDate?: string | null;
  issueDate?: string | null;
  number?: string | null;
}): boolean {
  return (
    doc.amount == null ||
    !doc.dueDate ||
    !doc.issueDate ||
    !doc.number
  );
}
