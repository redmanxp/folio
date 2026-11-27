import { useEffect, useMemo, useState, type FormEvent } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { ArrowDown, ArrowUp, ArrowUpDown, FileUp, ScanText, Search } from "lucide-react";
import { api, errMessage } from "@/api";
import { useAppShell } from "@/app/AppShellContext";
import { Alert } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "@/components/ui/empty-state";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { cn, formatCurrency } from "@/lib/utils";
import type { Document, ImportReport, SearchResult } from "@/types";

type SortKey = "providerName" | "number" | "issueDate" | "period" | "amount" | "paymentStatus";

export function InvoicesPage() {
  const {
    activeEntityId,
    openDocument,
    openProvider,
    setError,
    bumpRefresh,
    setSection,
    processing,
    runProcessing,
    workspaceMode,
  } = useAppShell();
  const [query, setQuery] = useState(() => sessionStorage.getItem("folio.searchQuery") ?? "");
  const [provider, setProvider] = useState("");
  const [period, setPeriod] = useState("");
  const [paymentStatus, setPaymentStatus] = useState("");
  const [result, setResult] = useState<SearchResult | null>(null);
  const [report, setReport] = useState<ImportReport | null>(null);
  const [busy, setBusy] = useState(false);
  const [sortKey, setSortKey] = useState<SortKey>("issueDate");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("desc");
  const busyUi = busy || processing.active;

  const sortedDocs = useMemo(() => {
    if (!result) return [];
    const docs = [...result.documents];
    const dir = sortDir === "asc" ? 1 : -1;
    docs.sort((a, b) => {
      const primary = compareDocs(a, b, sortKey) * dir;
      if (primary !== 0) return primary;
      // desempate: más recientes primero
      return (b.importedAt || "").localeCompare(a.importedAt || "");
    });
    return docs;
  }, [result, sortKey, sortDir]);

  function toggleSort(key: SortKey) {
    if (sortKey === key) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortKey(key);
      setSortDir(
        key === "amount" || key === "period" || key === "issueDate" ? "desc" : "asc",
      );
    }
  }

  function formatIssueDate(value?: string | null) {
    if (!value) return "—";
    const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(value);
    if (m) return `${m[3]}/${m[2]}/${m[1]}`;
    return value;
  }

  async function runSearch(event?: FormEvent) {
    event?.preventDefault();
    if (!activeEntityId) {
      setError("Seleccioná o creá una entidad");
      return;
    }
    setBusy(true);
    try {
      sessionStorage.removeItem("folio.searchQuery");
      setResult(
        await api.searchDocuments({
          entityId: activeEntityId,
          query: query || null,
          provider: provider || null,
          period: period || null,
          paymentStatus: paymentStatus || null,
        }),
      );
    } catch (err) {
      setError(errMessage(err));
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    if (activeEntityId) void runSearch();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeEntityId]);

  async function importFiles() {
    if (!activeEntityId) {
      setError("Seleccioná o creá una entidad");
      return;
    }
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: "Documentos", extensions: ["pdf", "png", "jpg", "jpeg"] }],
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      const imported = await runProcessing(
        `Importando ${paths.length} archivo${paths.length === 1 ? "" : "s"}…`,
        async () => {
          const r = await api.importFiles(activeEntityId, paths);
          if (r.documentIds.length > 0) {
            const { enrichDocumentsWithPdfJs } = await import("@/lib/enrichFromPdf");
            await enrichDocumentsWithPdfJs(r.documentIds);
          }
          return r;
        },
      );
      setReport(imported);
      if (imported.errors.length > 0) {
        setError(imported.errors.slice(0, 3).join(" · "));
      }
      bumpRefresh();
      await runSearch();
    } catch (err) {
      setError(errMessage(err));
    }
  }

  async function reprocessExisting() {
    if (!activeEntityId) {
      setError("Seleccioná o creá una entidad");
      return;
    }
    if (
      !confirm(
        "¿Reprocesar las facturas ya cargadas? Se leerá QR/texto/OCR y se crearán proveedores por CUIT.",
      )
    ) {
      return;
    }
    try {
      const imported = await runProcessing("Reprocesando facturas…", async () => {
        const r = await api.reprocessDocuments(activeEntityId);
        if (r.documentIds.length > 0) {
          const { enrichDocumentsWithPdfJs } = await import("@/lib/enrichFromPdf");
          await enrichDocumentsWithPdfJs(r.documentIds);
        }
        return r;
      });
      setReport(imported);
      if (imported.errors.length > 0) {
        setError(imported.errors.slice(0, 3).join(" · "));
      }
      bumpRefresh();
      await runSearch();
    } catch (err) {
      setError(errMessage(err));
    }
  }

  if (!activeEntityId) {
    return (
      <EmptyState
        icon={FileUp}
        title="Necesitás una entidad activa"
        description="Creá Personal o una empresa en Configuración antes de importar."
        actionLabel="Ir a Configuración"
        onAction={() => setSection("settings")}
      />
    );
  }

  return (
    <div className="mx-auto flex max-w-7xl flex-col gap-4">
      <Card>
        <CardHeader className="flex-col items-stretch gap-3 sm:flex-row sm:items-center sm:justify-between">
          <div className="min-w-0">
            <CardTitle>Importar</CardTitle>
            <CardDescription>
              Copia PDFs, lee QR ARCA/AFIP, aplica OCR si hace falta y crea proveedores por CUIT.
            </CardDescription>
          </div>
          <div className="flex shrink-0 flex-row flex-nowrap items-center gap-2">
            <Button onClick={() => void importFiles()} disabled={busyUi}>
              <FileUp className="h-4 w-4" />
              {busyUi ? "Procesando…" : "Elegir archivos"}
            </Button>
            <Button variant="outline" onClick={() => void reprocessExisting()} disabled={busyUi}>
              <ScanText className="h-4 w-4" />
              Reprocesar cargadas
            </Button>
          </div>
        </CardHeader>
        {report ? (
          <CardContent className="space-y-2">
            <Alert title="Proceso completado">
              {report.imported} ok · {report.duplicates} duplicados · {report.qrParsed} QR ARCA ·{" "}
              {report.ocrUsed} OCR · {report.providersCreated} proveedores nuevos ·{" "}
              {report.errors.length} errores
            </Alert>
            {report.errors.length > 0 ? (
              <Alert variant="danger" title="Detalle de errores">
                {report.errors.slice(0, 5).join(" · ")}
                {report.errors.length > 5 ? ` · +${report.errors.length - 5} más` : ""}
              </Alert>
            ) : null}
          </CardContent>
        ) : null}
      </Card>

      <Card>
        <CardHeader>
          <div>
            <CardTitle>Buscar facturas</CardTitle>
            <CardDescription>Filtrá por texto, proveedor, período o estado de pago.</CardDescription>
          </div>
        </CardHeader>
        <CardContent>
          <form className="grid gap-3 md:grid-cols-4" onSubmit={(e) => void runSearch(e)}>
            <Input
              placeholder="Texto"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
            <Input
              placeholder="Proveedor"
              value={provider}
              onChange={(e) => setProvider(e.target.value)}
            />
            <Input
              placeholder={workspaceMode === "invoices" ? "Mes YYYY-MM" : "Período YYYY-MM"}
              value={period}
              onChange={(e) => setPeriod(e.target.value)}
            />
            <Select value={paymentStatus} onChange={(e) => setPaymentStatus(e.target.value)}>
              <option value="">Todos los estados</option>
              <option value="pending">Pendiente</option>
              <option value="partial">Parcial</option>
              <option value="paid">Pagado</option>
            </Select>
            <Button type="submit" disabled={busyUi} className="md:col-span-4 md:w-fit">
              <Search className="h-4 w-4" />
              Buscar
            </Button>
          </form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div>
            <CardTitle>
              {result ? `${result.count} resultado${result.count === 1 ? "" : "s"}` : "Resultados"}
            </CardTitle>
            <CardDescription>
              {result
                ? `Total importes: ${formatCurrency(result.totalAmount)}`
                : "Ejecutá una búsqueda para listar documentos."}
            </CardDescription>
          </div>
        </CardHeader>
        <CardContent>
          {!result || result.documents.length === 0 ? (
            <EmptyState
              icon={Search}
              title="Sin resultados"
              description="Importá archivos o ajustá los filtros de búsqueda."
            />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <SortableHead
                    label="Proveedor"
                    active={sortKey === "providerName"}
                    dir={sortDir}
                    onClick={() => toggleSort("providerName")}
                  />
                  <SortableHead
                    label="Número"
                    active={sortKey === "number"}
                    dir={sortDir}
                    onClick={() => toggleSort("number")}
                  />
                  <SortableHead
                    label="Emisión"
                    active={sortKey === "issueDate"}
                    dir={sortDir}
                    onClick={() => toggleSort("issueDate")}
                  />
                  <SortableHead
                    label={workspaceMode === "invoices" ? "Mes" : "Período"}
                    active={sortKey === "period"}
                    dir={sortDir}
                    onClick={() => toggleSort("period")}
                  />
                  <SortableHead
                    label="Importe"
                    active={sortKey === "amount"}
                    dir={sortDir}
                    onClick={() => toggleSort("amount")}
                  />
                  {workspaceMode === "management" ? (
                    <SortableHead
                      label="Pago"
                      active={sortKey === "paymentStatus"}
                      dir={sortDir}
                      onClick={() => toggleSort("paymentStatus")}
                    />
                  ) : null}
                  <TableHead className="w-[1%]" />
                </TableRow>
              </TableHeader>
              <TableBody>
                {sortedDocs.map((doc) => (
                  <TableRow
                    key={doc.id}
                    className="cursor-pointer"
                    onClick={() => openDocument(doc.id)}
                  >
                    <TableCell>
                      {doc.providerId && doc.providerName ? (
                        <button
                          type="button"
                          className="text-left font-medium text-primary underline-offset-2 hover:underline"
                          onClick={(e) => {
                            e.stopPropagation();
                            openProvider(doc.providerId!, "invoices");
                          }}
                        >
                          {doc.providerName}
                        </button>
                      ) : (
                        <span className="font-medium">{doc.providerName ?? "—"}</span>
                      )}
                    </TableCell>
                    <TableCell>{doc.number ?? "—"}</TableCell>
                    <TableCell>{formatIssueDate(doc.issueDate)}</TableCell>
                    <TableCell>{doc.period ?? "—"}</TableCell>
                    <TableCell>
                      {doc.amount != null ? formatCurrency(doc.amount) : "—"}
                    </TableCell>
                    {workspaceMode === "management" ? (
                      <TableCell>
                        <Badge
                          variant={
                            doc.paymentStatus === "paid"
                              ? "success"
                              : doc.paymentStatus === "partial"
                                ? "default"
                                : "warning"
                          }
                        >
                          {doc.paymentStatus ?? "—"}
                        </Badge>
                      </TableCell>
                    ) : null}
                    <TableCell>
                      <Button
                        size="sm"
                        variant="outline"
                        disabled={busyUi}
                        title="Reprocesar esta factura"
                        onClick={(e) => {
                          e.stopPropagation();
                          void (async () => {
                            try {
                              const imported = await runProcessing(
                                `Reprocesando ${doc.fileName}…`,
                                async () => {
                                  const r = await api.reprocessDocument(doc.id);
                                  const { enrichDocumentWithPdfJs } = await import(
                                    "@/lib/enrichFromPdf"
                                  );
                                  const current = await api.getDocument(doc.id);
                                  await enrichDocumentWithPdfJs(current);
                                  return r;
                                },
                              );
                              setReport(imported);
                              bumpRefresh();
                              await runSearch();
                            } catch (err) {
                              setError(errMessage(err));
                            }
                          })();
                        }}
                      >
                        <ScanText className="h-4 w-4" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function SortableHead({
  label,
  active,
  dir,
  onClick,
}: {
  label: string;
  active: boolean;
  dir: "asc" | "desc";
  onClick: () => void;
}) {
  const Icon = !active ? ArrowUpDown : dir === "asc" ? ArrowUp : ArrowDown;
  return (
    <TableHead>
      <button
        type="button"
        className={cn(
          "inline-flex items-center gap-1 text-left font-medium hover:text-foreground",
          active ? "text-foreground" : "text-muted-foreground",
        )}
        onClick={onClick}
      >
        {label}
        <Icon className="h-3.5 w-3.5 opacity-70" />
      </button>
    </TableHead>
  );
}

function compareDocs(a: Document, b: Document, key: SortKey): number {
  switch (key) {
    case "amount": {
      const av = a.amount ?? Number.NEGATIVE_INFINITY;
      const bv = b.amount ?? Number.NEGATIVE_INFINITY;
      return av === bv ? 0 : av < bv ? -1 : 1;
    }
    case "issueDate":
      return (a.issueDate || "").localeCompare(b.issueDate || "");
    case "providerName":
      return (a.providerName || "").localeCompare(b.providerName || "", "es", {
        sensitivity: "base",
      });
    case "number":
      return (a.number || "").localeCompare(b.number || "", "es", {
        numeric: true,
        sensitivity: "base",
      });
    case "period":
      return (a.period || "").localeCompare(b.period || "", "es");
    case "paymentStatus":
      return (a.paymentStatus || "").localeCompare(b.paymentStatus || "", "es");
    default:
      return 0;
  }
}
