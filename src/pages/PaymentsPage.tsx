import { useEffect, useState } from "react";
import { Wallet } from "lucide-react";
import { api, errMessage } from "@/api";
import { useAppShell } from "@/app/AppShellContext";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "@/components/ui/empty-state";
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
import { formatCurrency } from "@/lib/utils";
import type { Payment } from "@/types";

export function PaymentsPage() {
  const { activeEntityId, openDocument, setError, setSection, bumpRefresh, refreshKey } =
    useAppShell();
  const [items, setItems] = useState<Payment[]>([]);
  const [status, setStatus] = useState("");
  const [loading, setLoading] = useState(true);

  async function refresh() {
    if (!activeEntityId) {
      setItems([]);
      setLoading(false);
      return;
    }
    setLoading(true);
    try {
      setItems(await api.listPayments(activeEntityId, status || null));
    } catch (err) {
      setError(errMessage(err));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
  }, [activeEntityId, status, refreshKey]);

  async function mark(documentId: string, next: string) {
    try {
      await api.updatePayment({
        documentId,
        status: next,
        paidAt: next === "paid" ? new Date().toISOString().slice(0, 10) : null,
      });
      bumpRefresh();
      await refresh();
    } catch (err) {
      setError(errMessage(err));
    }
  }

  if (!activeEntityId) {
    return (
      <EmptyState
        icon={Wallet}
        title="Sin entidad activa"
        description="Creá un espacio de trabajo para controlar pagos."
        actionLabel="Ir a Configuración"
        onAction={() => setSection("settings")}
      />
    );
  }

  if (loading) return <LoadingBlock label="Cargando pagos…" />;

  return (
    <div className="mx-auto max-w-7xl">
      <Card>
        <CardHeader>
          <div>
            <CardTitle>Control de pagos</CardTitle>
            <CardDescription>Marcá pendientes, parciales o pagados.</CardDescription>
          </div>
          <Select
            className="w-44"
            value={status}
            onChange={(e) => setStatus(e.target.value)}
          >
            <option value="">Todos</option>
            <option value="pending">Pendientes</option>
            <option value="partial">Parciales</option>
            <option value="paid">Pagados</option>
          </Select>
        </CardHeader>
        <CardContent>
          {items.length === 0 ? (
            <EmptyState
              icon={Wallet}
              title="Sin pagos para mostrar"
              description="Importá documentos desde Facturas para empezar el seguimiento."
            />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Documento</TableHead>
                  <TableHead>Proveedor</TableHead>
                  <TableHead>Vence</TableHead>
                  <TableHead>Importe</TableHead>
                  <TableHead>Estado</TableHead>
                  <TableHead></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {items.map((item) => (
                  <TableRow key={item.id}>
                    <TableCell>
                      <button
                        type="button"
                        className="font-medium text-brand-blue hover:underline"
                        onClick={() => openDocument(item.documentId)}
                      >
                        {item.fileName}
                      </button>
                    </TableCell>
                    <TableCell>{item.providerName ?? "—"}</TableCell>
                    <TableCell>{item.dueDate ?? "—"}</TableCell>
                    <TableCell>
                      {item.amountDue != null ? formatCurrency(item.amountDue) : "—"}
                    </TableCell>
                    <TableCell>
                      <Badge
                        variant={
                          item.status === "paid"
                            ? "success"
                            : item.status === "partial"
                              ? "default"
                              : "warning"
                        }
                      >
                        {item.status}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <div className="flex flex-wrap gap-1">
                        <Button size="sm" variant="outline" onClick={() => void mark(item.documentId, "pending")}>
                          Pendiente
                        </Button>
                        <Button size="sm" variant="outline" onClick={() => void mark(item.documentId, "partial")}>
                          Parcial
                        </Button>
                        <Button size="sm" variant="outline" onClick={() => void mark(item.documentId, "paid")}>
                          Pagado
                        </Button>
                      </div>
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
