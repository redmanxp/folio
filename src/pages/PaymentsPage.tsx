import { useEffect, useMemo, useState } from "react";
import { Wallet, X } from "lucide-react";
import { api, errMessage } from "@/api";
import { useAppShell } from "@/app/AppShellContext";
import { PaymentStatusControls } from "@/components/payments/PaymentStatusControls";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "@/components/ui/empty-state";
import { Select } from "@/components/ui/select";
import { LoadingBlock } from "@/components/ui/skeleton";
import { SortableHead } from "@/components/ui/sortable-head";
import { DEFAULT_PAGE_SIZE, TablePagination } from "@/components/ui/table-pagination";
import {
  Table,
  TableBody,
  TableCell,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { cn, dueDateClass, formatCurrency, paymentStatusLabel } from "@/lib/utils";
import type { Payment } from "@/types";

type SortKey =
  | "providerName"
  | "number"
  | "issueDate"
  | "dueDate"
  | "amountDue"
  | "amountPaid"
  | "status";

function formatIssueDate(value?: string | null) {
  if (!value) return "—";
  const m = value.match(/^(\d{4})-(\d{2})-(\d{2})/);
  if (m) return `${m[3]}/${m[2]}/${m[1]}`;
  return value;
}

function comparePayments(a: Payment, b: Payment, key: SortKey): number {
  switch (key) {
    case "amountDue": {
      const av = a.amountDue ?? Number.NEGATIVE_INFINITY;
      const bv = b.amountDue ?? Number.NEGATIVE_INFINITY;
      return av === bv ? 0 : av < bv ? -1 : 1;
    }
    case "amountPaid":
      return a.amountPaid === b.amountPaid ? 0 : a.amountPaid < b.amountPaid ? -1 : 1;
    case "issueDate":
      return (a.issueDate || "").localeCompare(b.issueDate || "");
    case "dueDate":
      return (a.dueDate || "").localeCompare(b.dueDate || "");
    case "number":
      return (a.number || "").localeCompare(b.number || "", "es", { numeric: true });
    case "status":
      return paymentStatusLabel(a.status).localeCompare(paymentStatusLabel(b.status), "es");
    case "providerName":
    default:
      return (a.providerName || "").localeCompare(b.providerName || "", "es", {
        sensitivity: "base",
      });
  }
}

export function PaymentsPage() {
  const {
    activeEntityId,
    openDocument,
    setError,
    setSection,
    bumpRefresh,
    refreshKey,
    headerQuery,
  } = useAppShell();
  const [items, setItems] = useState<Payment[]>([]);
  const [status, setStatus] = useState("");
  const [providerFilter, setProviderFilter] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [sortKey, setSortKey] = useState<SortKey>("dueDate");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");
  const [page, setPage] = useState(0);

  async function refresh(opts?: { soft?: boolean }) {
    if (!activeEntityId) {
      setItems([]);
      setLoading(false);
      return;
    }
    if (!opts?.soft) setLoading(true);
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeEntityId, status, refreshKey]);

  useEffect(() => {
    setPage(0);
  }, [status, headerQuery, providerFilter, sortKey, sortDir]);

  const filtered = useMemo(() => {
    const q = headerQuery.trim().toLowerCase();
    return items.filter((item) => {
      if (providerFilter) {
        const name = item.providerName ?? "Sin proveedor";
        if (name !== providerFilter) return false;
      }
      if (!q) return true;
      const hay = [
        item.providerName,
        item.number,
        item.issueDate,
        item.dueDate,
        paymentStatusLabel(item.status),
        item.amountDue != null ? String(item.amountDue) : "",
        String(item.amountPaid),
      ]
        .filter(Boolean)
        .join(" ")
        .toLowerCase();
      return hay.includes(q);
    });
  }, [items, headerQuery, providerFilter]);

  const sorted = useMemo(() => {
    const rows = [...filtered];
    const dir = sortDir === "asc" ? 1 : -1;
    rows.sort((a, b) => {
      const primary = comparePayments(a, b, sortKey) * dir;
      if (primary !== 0) return primary;
      return (b.updatedAt || "").localeCompare(a.updatedAt || "");
    });
    return rows;
  }, [filtered, sortKey, sortDir]);

  const pageItems = useMemo(() => {
    const start = page * DEFAULT_PAGE_SIZE;
    return sorted.slice(start, start + DEFAULT_PAGE_SIZE);
  }, [sorted, page]);

  function toggleSort(key: SortKey) {
    if (sortKey === key) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortKey(key);
      setSortDir(
        key === "amountDue" ||
          key === "amountPaid" ||
          key === "issueDate" ||
          key === "dueDate"
          ? "desc"
          : "asc",
      );
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

  if (loading && items.length === 0) return <LoadingBlock label="Cargando pagos…" />;

  return (
    <div className="mx-auto max-w-7xl">
      <Card>
        <CardHeader>
          <div>
            <CardTitle>Control de pagos</CardTitle>
            <CardDescription>Marcá pendientes, parciales o pagados.</CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            {providerFilter ? (
              <Button
                type="button"
                size="sm"
                variant="secondary"
                className="max-w-[16rem]"
                title="Quitar filtro de proveedor"
                onClick={() => setProviderFilter(null)}
              >
                <span className="truncate">{providerFilter}</span>
                <X className="h-3.5 w-3.5 shrink-0" />
              </Button>
            ) : null}
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
          </div>
        </CardHeader>
        <CardContent>
          {sorted.length === 0 ? (
            <EmptyState
              icon={Wallet}
              title="Sin pagos para mostrar"
              description={
                providerFilter || headerQuery.trim()
                  ? "Ningún pago coincide con los filtros actuales."
                  : "Importá documentos desde Facturas para empezar el seguimiento."
              }
            />
          ) : (
            <>
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
                      label="Vence"
                      active={sortKey === "dueDate"}
                      dir={sortDir}
                      onClick={() => toggleSort("dueDate")}
                    />
                    <SortableHead
                      label="Importe"
                      active={sortKey === "amountDue"}
                      dir={sortDir}
                      onClick={() => toggleSort("amountDue")}
                      className="text-right"
                    />
                    <SortableHead
                      label="Pagado"
                      active={sortKey === "amountPaid"}
                      dir={sortDir}
                      onClick={() => toggleSort("amountPaid")}
                      className="text-right"
                    />
                    <SortableHead
                      label="Estado"
                      active={sortKey === "status"}
                      dir={sortDir}
                      onClick={() => toggleSort("status")}
                    />
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {pageItems.map((item) => {
                    const providerName = item.providerName ?? "Sin proveedor";
                    const filteredByThis = providerFilter === providerName;
                    return (
                    <TableRow key={item.id}>
                      <TableCell>
                        <button
                          type="button"
                          className={cn(
                            "font-medium text-brand-blue hover:underline",
                            filteredByThis && "underline",
                          )}
                          title={
                            filteredByThis
                              ? "Quitar filtro de proveedor"
                              : `Filtrar por ${providerName}`
                          }
                          onClick={() =>
                            setProviderFilter(filteredByThis ? null : providerName)
                          }
                        >
                          {providerName}
                        </button>
                      </TableCell>
                      <TableCell>
                        <button
                          type="button"
                          className="text-left hover:underline"
                          title="Abrir documento"
                          onClick={() => openDocument(item.documentId)}
                        >
                          {item.number ?? "—"}
                        </button>
                      </TableCell>
                      <TableCell>{formatIssueDate(item.issueDate)}</TableCell>
                      <TableCell className={cn(dueDateClass(item.dueDate, item.status))}>
                        {item.dueDate ? formatIssueDate(item.dueDate) : "—"}
                      </TableCell>
                      <TableCell className="text-right tabular-nums">
                        {item.amountDue != null ? formatCurrency(item.amountDue) : "—"}
                      </TableCell>
                      <TableCell className="text-right tabular-nums">
                        {formatCurrency(item.amountPaid)}
                      </TableCell>
                      <TableCell>
                        <PaymentStatusControls
                          entityId={activeEntityId}
                          documentId={item.documentId}
                          status={item.status}
                          amountDue={item.amountDue}
                          amountPaid={item.amountPaid}
                          compact
                          onChanged={() => {
                            bumpRefresh();
                            void refresh({ soft: true });
                          }}
                          onError={setError}
                        />
                      </TableCell>
                    </TableRow>
                    );
                  })}
                </TableBody>
              </Table>
              <TablePagination
                page={page}
                pageSize={DEFAULT_PAGE_SIZE}
                total={sorted.length}
                onPageChange={setPage}
              />
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
