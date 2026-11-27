import { useEffect, useState, type FormEvent } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { FileText, ScanText } from "lucide-react";
import { api, errMessage } from "@/api";
import { useAppShell } from "@/app/AppShellContext";
import { PdfViewer } from "@/components/documents/PdfViewer";
import { Alert } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "@/components/ui/empty-state";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { LoadingBlock } from "@/components/ui/skeleton";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { Document, Provider } from "@/types";

export function DocumentsPage() {
  const {
    activeEntityId,
    selectedDocumentId,
    openDocument,
    closeDocument,
    setError,
    bumpRefresh,
    setSection,
    refreshKey,
    runProcessing,
    processing,
    workspaceMode,
  } = useAppShell();
  const [docs, setDocs] = useState<Document[]>([]);
  const [doc, setDoc] = useState<Document | null>(null);
  const [providers, setProviders] = useState<Provider[]>([]);
  const [loading, setLoading] = useState(true);
  const [number, setNumber] = useState("");
  const [period, setPeriod] = useState("");
  const [issueDate, setIssueDate] = useState("");
  const [dueDate, setDueDate] = useState("");
  const [amount, setAmount] = useState("");
  const [providerId, setProviderId] = useState("");
  const [notes, setNotes] = useState("");
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    void (async () => {
      if (!activeEntityId) {
        setDocs([]);
        setLoading(false);
        return;
      }
      setLoading(true);
      try {
        setDocs(await api.listRecentDocuments(activeEntityId, 100));
        setProviders(await api.listProviders(activeEntityId));
      } catch (err) {
        setError(errMessage(err));
      } finally {
        setLoading(false);
      }
    })();
  }, [activeEntityId, refreshKey, setError]);

  useEffect(() => {
    if (!selectedDocumentId) {
      setDoc(null);
      return;
    }
    if (!activeEntityId) {
      setDoc(null);
      return;
    }
    void (async () => {
      try {
        const current = await api.getDocument(activeEntityId, selectedDocumentId);
        setDoc(current);
        setNumber(current.number ?? "");
        setPeriod(current.period ?? "");
        setIssueDate(current.issueDate ?? "");
        setDueDate(current.dueDate ?? "");
        setAmount(current.amount?.toString() ?? "");
        setProviderId(current.providerId ?? "");
        setNotes(current.notes ?? "");
      } catch (err) {
        setError(errMessage(err));
      }
    })();
  }, [activeEntityId, selectedDocumentId, setError]);

  async function onSave(event: FormEvent) {
    event.preventDefault();
    if (!doc || !activeEntityId) return;
    try {
      const updated = await api.updateDocumentMeta({
        entityId: activeEntityId,
        id: doc.id,
        number: number || null,
        period: period || null,
        issueDate: issueDate || null,
        dueDate: dueDate || null,
        amount: amount ? Number(amount) : null,
        providerId: providerId || null,
        notes: notes || null,
        docType: doc.docType,
      });
      setDoc(updated);
      bumpRefresh();
    } catch (err) {
      setError(errMessage(err));
    }
  }

  async function onDelete() {
    if (!doc || !activeEntityId) return;
    if (!confirm("¿Quitar el documento del índice de Folio?")) return;
    const deleteFile = confirm("¿También eliminar el archivo copiado en Folio?");
    try {
      await api.deleteDocument(activeEntityId, doc.id, deleteFile);
      closeDocument();
      bumpRefresh();
    } catch (err) {
      setError(errMessage(err));
    }
  }

  async function onReprocessOne() {
    if (!doc || !activeEntityId) return;
    setMessage(null);
    try {
      const report = await runProcessing(`Reprocesando ${doc.fileName}…`, async () => {
        const r = await api.reprocessDocument(doc.id);
        let current = await api.getDocument(activeEntityId, doc.id);
        const { enrichDocumentWithPdfJs } = await import("@/lib/enrichFromPdf");
        current = await enrichDocumentWithPdfJs(current);
        return { report: r, current };
      });
      const current = report.current;
      setDoc(current);
      setNumber(current.number ?? "");
      setPeriod(current.period ?? "");
      setIssueDate(current.issueDate ?? "");
      setDueDate(current.dueDate ?? "");
      setAmount(current.amount?.toString() ?? "");
      setProviderId(current.providerId ?? "");
      setNotes(current.notes ?? "");
      if (activeEntityId) {
        setProviders(await api.listProviders(activeEntityId));
      }
      bumpRefresh();
      setMessage(
        `Reprocesado: ${report.report.qrParsed} QR · ${report.report.ocrUsed} OCR · ${report.report.providersCreated} proveedores · ${report.report.errors.length} errores`,
      );
      if (report.report.errors.length > 0) {
        setError(report.report.errors.slice(0, 3).join(" · "));
      }
    } catch (err) {
      setError(errMessage(err));
    }
  }

  if (!activeEntityId) {
    return (
      <EmptyState
        icon={FileText}
        title="Sin entidad activa"
        description="Creá un espacio de trabajo para ver documentos."
        actionLabel="Ir a Configuración"
        onAction={() => setSection("settings")}
      />
    );
  }

  if (loading) return <LoadingBlock label="Cargando documentos…" />;

  if (selectedDocumentId && doc) {
    const src = convertFileSrc(doc.storedPath);
    const isPdf =
      doc.mimeType?.includes("pdf") || doc.fileName.toLowerCase().endsWith(".pdf");

    return (
      <div className="mx-auto grid max-w-7xl gap-4 xl:grid-cols-[1.4fr_0.8fr]">
        <Card className="overflow-hidden">
          <CardHeader>
            <div>
              <CardTitle>{doc.fileName}</CardTitle>
              <CardDescription>Hash {doc.fileHash.slice(0, 16)}…</CardDescription>
            </div>
            <Button variant="outline" onClick={closeDocument}>
              Volver al listado
            </Button>
          </CardHeader>
          <CardContent className="p-3">
            {isPdf ? (
              <PdfViewer filePath={doc.storedPath} title={doc.fileName} />
            ) : (
              <img src={src} alt={doc.fileName} className="max-h-[70vh] w-full rounded-xl object-contain" />
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <div>
              <CardTitle>Metadatos</CardTitle>
              <CardDescription>
                {workspaceMode === "invoices"
                  ? "Mes de la factura (YYYY-MM), según la fecha de emisión."
                  : "Período = mes (YYYY-MM), derivado de la fecha del QR/factura."}
              </CardDescription>
            </div>
          </CardHeader>
          <CardContent className="space-y-3">
            {message ? <Alert variant="success">{message}</Alert> : null}
            <Button
              type="button"
              variant="outline"
              disabled={processing.active}
              onClick={() => void onReprocessOne()}
            >
              <ScanText className="h-4 w-4" />
              {processing.active ? "Reprocesando…" : "Reprocesar esta factura"}
            </Button>
            <form className="grid gap-3" onSubmit={(e) => void onSave(e)}>
              <label className="grid gap-1 text-sm">
                Número
                <Input value={number} onChange={(e) => setNumber(e.target.value)} />
              </label>
              <label className="grid gap-1 text-sm">
                {workspaceMode === "invoices" ? "Mes (YYYY-MM)" : "Período (YYYY-MM)"}
                <Input value={period} onChange={(e) => setPeriod(e.target.value)} placeholder="2026-03" />
              </label>
              <label className="grid gap-1 text-sm">
                Fecha emisión
                <Input type="date" value={issueDate} onChange={(e) => setIssueDate(e.target.value)} />
              </label>
              <label className="grid gap-1 text-sm">
                Vencimiento
                <Input type="date" value={dueDate} onChange={(e) => setDueDate(e.target.value)} />
              </label>
              <label className="grid gap-1 text-sm">
                Importe
                <Input value={amount} onChange={(e) => setAmount(e.target.value)} inputMode="decimal" />
              </label>
              <label className="grid gap-1 text-sm">
                Proveedor
                <Select value={providerId} onChange={(e) => setProviderId(e.target.value)}>
                  <option value="">Sin proveedor</option>
                  {providers.map((provider) => (
                    <option key={provider.id} value={provider.id}>
                      {provider.name}
                    </option>
                  ))}
                </Select>
              </label>

              {(doc.pointOfSale != null ||
                doc.authCode ||
                doc.fxRate != null ||
                doc.receptorDocNumber) && (
                <div className="rounded-xl border border-border bg-muted/40 p-3 text-sm">
                  <p className="mb-2 font-medium text-foreground">Datos del QR ARCA</p>
                  <dl className="grid gap-1.5 text-muted-foreground">
                    {doc.docType ? (
                      <div className="flex justify-between gap-2">
                        <dt>Tipo</dt>
                        <dd className="text-right text-foreground">{doc.docType}</dd>
                      </div>
                    ) : null}
                    {doc.pointOfSale != null ? (
                      <div className="flex justify-between gap-2">
                        <dt>Punto de venta</dt>
                        <dd className="text-right text-foreground">{doc.pointOfSale}</dd>
                      </div>
                    ) : null}
                    {doc.authType || doc.authCode ? (
                      <div className="flex justify-between gap-2">
                        <dt>{doc.authType ?? "Autorización"}</dt>
                        <dd className="text-right font-mono text-foreground">{doc.authCode ?? "—"}</dd>
                      </div>
                    ) : null}
                    {doc.currency && doc.currency !== "ARS" ? (
                      <div className="flex justify-between gap-2">
                        <dt>Moneda</dt>
                        <dd className="text-right text-foreground">{doc.currency}</dd>
                      </div>
                    ) : null}
                    {doc.fxRate != null ? (
                      <div className="flex justify-between gap-2">
                        <dt>Cotización</dt>
                        <dd className="text-right text-foreground">{doc.fxRate}</dd>
                      </div>
                    ) : null}
                    {doc.receptorDocNumber ? (
                      <div className="flex justify-between gap-2">
                        <dt>Receptor</dt>
                        <dd className="text-right font-mono text-foreground">
                          {doc.receptorDocType != null ? `${doc.receptorDocType} · ` : ""}
                          {doc.receptorDocNumber}
                        </dd>
                      </div>
                    ) : null}
                  </dl>
                </div>
              )}

              <label className="grid gap-1 text-sm">
                Notas
                <textarea
                  className="min-h-24 rounded-xl border border-border bg-card px-3 py-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
                  value={notes}
                  onChange={(e) => setNotes(e.target.value)}
                />
              </label>
              <div className="flex flex-wrap gap-2">
                <Button type="submit">Guardar</Button>
                <Button type="button" variant="danger" onClick={() => void onDelete()}>
                  Eliminar
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-7xl">
      <Card>
        <CardHeader>
          <div>
            <CardTitle>Archivo de documentos</CardTitle>
            <CardDescription>Abrí un documento para visualizarlo y editar metadatos.</CardDescription>
          </div>
          <Button onClick={() => setSection("invoices")}>Importar más</Button>
        </CardHeader>
        <CardContent>
          {docs.length === 0 ? (
            <EmptyState
              icon={FileText}
              title="Todavía no hay documentos"
              description="Importá PDF desde Facturas."
              actionLabel="Ir a Facturas"
              onAction={() => setSection("invoices")}
            />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Archivo</TableHead>
                  <TableHead>Proveedor</TableHead>
                  <TableHead>Período</TableHead>
                  <TableHead>Importado</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {docs.map((item) => (
                  <TableRow
                    key={item.id}
                    className="cursor-pointer"
                    onClick={() => openDocument(item.id)}
                  >
                    <TableCell className="font-medium">{item.fileName}</TableCell>
                    <TableCell>{item.providerName ?? "—"}</TableCell>
                    <TableCell>{item.period ?? "—"}</TableCell>
                    <TableCell>{item.importedAt.slice(0, 10)}</TableCell>
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
