import { useEffect, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import * as pdfjs from "pdfjs-dist";
import pdfWorker from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import { LoadingBlock } from "@/components/ui/skeleton";

pdfjs.GlobalWorkerOptions.workerSrc = pdfWorker;

type Props = {
  filePath: string;
  title: string;
};

/**
 * Visor PDF vía PDF.js (canvas). Más fiable que iframe/asset en WebView2.
 */
export function PdfViewer({ filePath, title }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    const container = containerRef.current;
    if (!container) return;

    container.replaceChildren();
    setLoading(true);
    setError(null);

    void (async () => {
      try {
        const url = convertFileSrc(filePath);
        const pdf = await pdfjs.getDocument({ url }).promise;
        if (cancelled) return;

        for (let pageNum = 1; pageNum <= pdf.numPages; pageNum++) {
          const page = await pdf.getPage(pageNum);
          if (cancelled) return;

          const base = page.getViewport({ scale: 1 });
          const width = container.clientWidth || 720;
          const scale = Math.min(2, Math.max(1, width / base.width));
          const viewport = page.getViewport({ scale });

          const canvas = document.createElement("canvas");
          canvas.width = viewport.width;
          canvas.height = viewport.height;
          canvas.className = "mb-3 w-full rounded-lg bg-white shadow-sm";
          canvas.setAttribute("aria-label", `${title} — página ${pageNum}`);

          const ctx = canvas.getContext("2d");
          if (!ctx) continue;
          await page.render({ canvasContext: ctx, viewport }).promise;
          if (cancelled) return;
          container.appendChild(canvas);
        }
        setLoading(false);
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : String(err));
          setLoading(false);
        }
      }
    })();

    return () => {
      cancelled = true;
      container.replaceChildren();
    };
  }, [filePath, title]);

  return (
    <div className="relative min-h-[70vh] overflow-auto rounded-xl bg-muted/40 p-2">
      {loading ? (
        <div className="absolute inset-0 z-10 flex items-center justify-center bg-background/60">
          <LoadingBlock label="Cargando PDF…" />
        </div>
      ) : null}
      {error ? (
        <p className="p-4 text-sm text-destructive">No se pudo mostrar el PDF: {error}</p>
      ) : null}
      <div ref={containerRef} className="mx-auto max-w-4xl" />
    </div>
  );
}
