import { useState, type FormEvent } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { BarChart3, Download } from "lucide-react";
import { api, errMessage } from "@/api";
import { useAppShell } from "@/app/AppShellContext";
import { Alert } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "@/components/ui/empty-state";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";

export function ReportsPage() {
  const { activeEntityId, setError, setSection } = useAppShell();
  const [query, setQuery] = useState("");
  const [period, setPeriod] = useState("");
  const [paymentStatus, setPaymentStatus] = useState("");
  const [exportedPath, setExportedPath] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function onExport(event: FormEvent) {
    event.preventDefault();
    if (!activeEntityId) {
      setError("Seleccioná una entidad");
      return;
    }
    try {
      const target = await save({
        defaultPath: `folio-${period || "export"}.csv`,
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });
      if (!target) return;
      setBusy(true);
      const path = await api.exportSearchCsv(
        {
          entityId: activeEntityId,
          query: query || null,
          period: period || null,
          paymentStatus: paymentStatus || null,
        },
        target,
      );
      setExportedPath(path);
    } catch (err) {
      setError(errMessage(err));
    } finally {
      setBusy(false);
    }
  }

  if (!activeEntityId) {
    return (
      <EmptyState
        icon={BarChart3}
        title="Sin entidad activa"
        description="Creá un espacio de trabajo para exportar informes."
        actionLabel="Ir a Configuración"
        onAction={() => setSection("settings")}
      />
    );
  }

  return (
    <div className="mx-auto max-w-3xl">
      <Card>
        <CardHeader>
          <div>
            <CardTitle>Exportar a CSV</CardTitle>
            <CardDescription>
              Compatible con Excel. Usa los filtros de la entidad activa.
            </CardDescription>
          </div>
        </CardHeader>
        <CardContent>
          <form className="grid gap-3" onSubmit={(e) => void onExport(e)}>
            <Input
              placeholder="Texto opcional"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
            <Input
              placeholder="Período YYYY-MM"
              value={period}
              onChange={(e) => setPeriod(e.target.value)}
            />
            <Select value={paymentStatus} onChange={(e) => setPaymentStatus(e.target.value)}>
              <option value="">Todos los estados de pago</option>
              <option value="pending">Pendiente</option>
              <option value="partial">Parcial</option>
              <option value="paid">Pagado</option>
            </Select>
            <Button type="submit" disabled={busy}>
              <Download className="h-4 w-4" />
              {busy ? "Exportando…" : "Exportar CSV"}
            </Button>
          </form>
          {exportedPath ? (
            <div className="mt-4">
              <Alert variant="success" title="Exportación lista">
                Archivo guardado en {exportedPath}
              </Alert>
            </div>
          ) : null}
        </CardContent>
      </Card>
    </div>
  );
}
