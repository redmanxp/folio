import { useEffect, useState } from "react";
import {
  CalendarClock,
  FileCheck2,
  FolderOpen,
  Receipt,
  Users,
} from "lucide-react";
import { api, errMessage } from "@/api";
import { useAppShell } from "@/app/AppShellContext";
import { BrandLogo } from "@/components/brand/BrandLogo";
import { Alert } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "@/components/ui/empty-state";
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
import type { DashboardStats, Document, Payment } from "@/types";

export function DashboardPage() {
  const {
    activeEntity,
    activeEntityId,
    setSection,
    openDocument,
    setError,
    refreshKey,
    workspaceMode,
  } = useAppShell();
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [recent, setRecent] = useState<Document[]>([]);
  const [agenda, setAgenda] = useState<Payment[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    void (async () => {
      setLoading(true);
      try {
        setStats(await api.getDashboardStats(activeEntityId));
        if (activeEntityId) {
          setRecent(await api.listRecentDocuments(activeEntityId, 8));
          setAgenda((await api.listAgenda(activeEntityId)).slice(0, 5));
        } else {
          setRecent([]);
          setAgenda([]);
        }
      } catch (err) {
        setError(errMessage(err));
      } finally {
        setLoading(false);
      }
    })();
  }, [activeEntityId, refreshKey, setError]);

  if (loading) return <LoadingBlock label="Cargando inicio…" />;

  if (!activeEntityId) {
    return (
      <EmptyState
        icon={FolderOpen}
        title="Creá una entidad para empezar"
        description="Personal o empresa: tus documentos quedan aislados por espacio de trabajo."
        actionLabel="Ir a Configuración"
        onAction={() => setSection("settings")}
      />
    );
  }

  const cards = [
    {
      label: "Documentos",
      value: String(stats?.documents ?? 0),
      icon: Receipt,
    },
    {
      label: "Proveedores",
      value: String(stats?.providers ?? 0),
      icon: Users,
    },
    ...(workspaceMode === "management"
      ? [
          {
            label: "Pagos pendientes",
            value: String(stats?.pendingPayments ?? 0),
            icon: CalendarClock,
          },
          {
            label: "Vencen en 7 días",
            value: String(stats?.dueSoon ?? 0),
            icon: FileCheck2,
          },
        ]
      : []),
  ];

  return (
    <div className="mx-auto flex max-w-7xl flex-col gap-6">
      <section className="overflow-hidden rounded-3xl border border-border bg-card shadow-[var(--shadow-soft)]">
        <div className="grid gap-6 p-6 md:grid-cols-[1.2fr_0.8fr] md:items-center">
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.16em] text-brand-blue">
              Folio
            </p>
            <h2 className="mt-2 text-3xl font-semibold tracking-tight text-foreground">
              Tus documentos, bajo control.
            </h2>
            <p className="mt-3 max-w-xl text-sm leading-relaxed text-muted-foreground">
              Trabajando en <strong>{activeEntity?.name}</strong>. Importá PDFs, buscá por
              proveedor o mes y tené tus facturas ordenadas.
            </p>
            <div className="mt-5 flex flex-wrap gap-2">
              <Button onClick={() => setSection("invoices")}>Importar / buscar</Button>
              {workspaceMode === "management" ? (
                <Button variant="outline" onClick={() => setSection("payments")}>
                  Ver pagos
                </Button>
              ) : (
                <Button variant="outline" onClick={() => setSection("providers")}>
                  Proveedores
                </Button>
              )}
            </div>
          </div>
          <div className="flex justify-start md:justify-end">
            <div className="rounded-3xl bg-white px-6 py-5">
              <BrandLogo variant="full" className="h-16 w-auto max-w-full" />
            </div>
          </div>
        </div>
      </section>

      <section className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
        {cards.map((item) => {
          const Icon = item.icon;
          return (
            <Card key={item.label}>
              <CardHeader>
                <div className="flex h-10 w-10 items-center justify-center rounded-2xl bg-brand-blue/10 text-brand-blue">
                  <Icon className="h-5 w-5" />
                </div>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground">{item.label}</p>
                <p className="mt-2 text-2xl font-semibold tracking-tight">{item.value}</p>
              </CardContent>
            </Card>
          );
        })}
      </section>

      <section className="grid gap-4 xl:grid-cols-[1.4fr_0.8fr]">
        <Card>
          <CardHeader>
            <div>
              <CardTitle>Recientes</CardTitle>
              <CardDescription>Últimos documentos importados.</CardDescription>
            </div>
            <Button variant="ghost" size="sm" onClick={() => setSection("documents")}>
              Ver todos
            </Button>
          </CardHeader>
          <CardContent>
            {recent.length === 0 ? (
              <Alert title="Sin documentos todavía">
                Andá a Facturas e importá tus primeros PDF.
              </Alert>
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Archivo</TableHead>
                    <TableHead>Proveedor</TableHead>
                    <TableHead>{workspaceMode === "invoices" ? "Mes" : "Período"}</TableHead>
                    {workspaceMode === "management" ? <TableHead>Estado</TableHead> : null}
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {recent.map((doc) => (
                    <TableRow
                      key={doc.id}
                      className="cursor-pointer"
                      onClick={() => openDocument(doc.id)}
                    >
                      <TableCell className="font-medium">{doc.fileName}</TableCell>
                      <TableCell>{doc.providerName ?? "—"}</TableCell>
                      <TableCell>{doc.period ?? "—"}</TableCell>
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
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </CardContent>
        </Card>

        {workspaceMode === "management" ? (
          <Card>
            <CardHeader>
              <div>
                <CardTitle>Próximos vencimientos</CardTitle>
                <CardDescription>Pendientes y parciales.</CardDescription>
              </div>
            </CardHeader>
            <CardContent className="space-y-3">
              {agenda.length === 0 ? (
                <p className="text-sm text-muted-foreground">No hay vencimientos pendientes.</p>
              ) : (
                agenda.map((item) => (
                  <button
                    key={item.id}
                    type="button"
                    className="flex w-full items-center justify-between rounded-2xl border border-border bg-muted/40 px-3 py-3 text-left hover:bg-muted/70"
                    onClick={() => openDocument(item.documentId)}
                  >
                    <div>
                      <p className="text-sm font-medium">{item.fileName}</p>
                      <p className="text-xs text-muted-foreground">
                        {item.providerName ?? "Sin proveedor"} · {item.dueDate ?? "Sin fecha"}
                      </p>
                    </div>
                    <p className="text-sm font-semibold">
                      {item.amountDue != null ? formatCurrency(item.amountDue) : "—"}
                    </p>
                  </button>
                ))
              )}
            </CardContent>
          </Card>
        ) : (
          <Card>
            <CardHeader>
              <div>
                <CardTitle>Atajos</CardTitle>
                <CardDescription>Modo solo facturas.</CardDescription>
              </div>
            </CardHeader>
            <CardContent className="space-y-2">
              <Button className="w-full" onClick={() => setSection("invoices")}>
                Ir a Facturas
              </Button>
              <Button className="w-full" variant="outline" onClick={() => setSection("documents")}>
                Ver documentos
              </Button>
            </CardContent>
          </Card>
        )}
      </section>
    </div>
  );
}
