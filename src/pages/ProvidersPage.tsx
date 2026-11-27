import { useEffect, useState, type FormEvent } from "react";
import { ArrowLeft, Plus, Users } from "lucide-react";
import { api, errMessage } from "@/api";
import { useAppShell } from "@/app/AppShellContext";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "@/components/ui/empty-state";
import { Input } from "@/components/ui/input";
import { LoadingBlock } from "@/components/ui/skeleton";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { Provider } from "@/types";

type ProviderForm = {
  name: string;
  cuit: string;
  alias: string;
  cbu: string;
  email: string;
  phone: string;
  notes: string;
};

const emptyForm = (): ProviderForm => ({
  name: "",
  cuit: "",
  alias: "",
  cbu: "",
  email: "",
  phone: "",
  notes: "",
});

function formFromProvider(p: Provider): ProviderForm {
  return {
    name: p.name,
    cuit: p.cuit ?? "",
    alias: p.alias ?? "",
    cbu: p.cbu ?? "",
    email: p.email ?? "",
    phone: p.phone ?? "",
    notes: p.notes ?? "",
  };
}

export function ProvidersPage() {
  const {
    activeEntityId,
    setError,
    setSection,
    bumpRefresh,
    refreshKey,
    selectedProviderId,
    openProvider,
    closeProvider,
    clearSelectedProvider,
  } = useAppShell();
  const [providers, setProviders] = useState<Provider[]>([]);
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState(false);
  const [form, setForm] = useState<ProviderForm>(emptyForm);
  const [saving, setSaving] = useState(false);

  const selected = providers.find((p) => p.id === selectedProviderId) ?? null;
  const showFicha = creating || Boolean(selected);

  async function refresh() {
    if (!activeEntityId) {
      setProviders([]);
      setLoading(false);
      return;
    }
    setLoading(true);
    try {
      setProviders(await api.listProviders(activeEntityId));
    } catch (err) {
      setError(errMessage(err));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeEntityId, refreshKey]);

  useEffect(() => {
    if (selected) {
      setCreating(false);
      setForm(formFromProvider(selected));
      return;
    }
    if (creating) {
      setForm(emptyForm());
    }
  }, [selected, creating]);

  function startCreate() {
    clearSelectedProvider();
    setForm(emptyForm());
    setCreating(true);
  }

  function leaveFicha() {
    if (creating) {
      setCreating(false);
      setForm(emptyForm());
      return;
    }
    closeProvider();
  }

  async function onSave(event: FormEvent) {
    event.preventDefault();
    if (!form.name.trim()) {
      setError("El nombre del proveedor es obligatorio");
      return;
    }

    setSaving(true);
    try {
      if (creating) {
        if (!activeEntityId) return;
        const created = await api.createProvider({
          entityId: activeEntityId,
          name: form.name.trim(),
          cuit: form.cuit.trim() || null,
          alias: form.alias.trim() || null,
          cbu: form.cbu.trim() || null,
          email: form.email.trim() || null,
          phone: form.phone.trim() || null,
          notes: form.notes.trim() || null,
        });
        setCreating(false);
        bumpRefresh();
        await refresh();
        openProvider(created.id);
        return;
      }

      if (!selected || !activeEntityId) return;
      const updated = await api.updateProvider({
        id: selected.id,
        entityId: activeEntityId,
        name: form.name.trim(),
        cuit: form.cuit.trim() || null,
        alias: form.alias.trim() || null,
        cbu: form.cbu.trim() || null,
        email: form.email.trim() || null,
        phone: form.phone.trim() || null,
        notes: form.notes.trim() || null,
      });
      setProviders((prev) => prev.map((p) => (p.id === updated.id ? updated : p)));
      bumpRefresh();
    } catch (err) {
      setError(errMessage(err));
    } finally {
      setSaving(false);
    }
  }

  async function onDelete(id: string) {
    if (!confirm("¿Eliminar proveedor?")) return;
    try {
      if (!activeEntityId) return;
      await api.deleteProvider(activeEntityId, id);
      if (selectedProviderId === id) clearSelectedProvider();
      bumpRefresh();
      await refresh();
    } catch (err) {
      setError(errMessage(err));
    }
  }

  if (!activeEntityId) {
    return (
      <EmptyState
        icon={Users}
        title="Sin entidad activa"
        description="Creá un espacio de trabajo para gestionar proveedores."
        actionLabel="Ir a Configuración"
        onAction={() => setSection("settings")}
      />
    );
  }

  if (loading) return <LoadingBlock label="Cargando proveedores…" />;

  if (showFicha) {
    return (
      <div className="mx-auto flex max-w-2xl flex-col gap-4">
        <Card>
          <CardHeader>
            <div>
              <CardTitle>{creating ? "Nuevo proveedor" : "Ficha del proveedor"}</CardTitle>
              <CardDescription>
                {creating
                  ? "Completá los datos del proveedor."
                  : "Datos fiscales, contacto y pago."}
              </CardDescription>
            </div>
            <Button variant="outline" onClick={leaveFicha}>
              <ArrowLeft className="h-4 w-4" />
              Volver
            </Button>
          </CardHeader>
          <CardContent>
            <form className="grid gap-3 sm:grid-cols-2" onSubmit={(e) => void onSave(e)}>
              <label className="grid gap-1 text-sm sm:col-span-2">
                Nombre
                <Input
                  value={form.name}
                  onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
                  required
                />
              </label>
              <label className="grid gap-1 text-sm">
                CUIT
                <Input
                  value={form.cuit}
                  onChange={(e) => setForm((f) => ({ ...f, cuit: e.target.value }))}
                  placeholder="XX-XXXXXXXX-X"
                />
              </label>
              <label className="grid gap-1 text-sm">
                Alias
                <Input
                  value={form.alias}
                  onChange={(e) => setForm((f) => ({ ...f, alias: e.target.value }))}
                />
              </label>
              <label className="grid gap-1 text-sm sm:col-span-2">
                CBU / CVU
                <Input
                  value={form.cbu}
                  onChange={(e) => setForm((f) => ({ ...f, cbu: e.target.value }))}
                  placeholder="22 dígitos"
                  inputMode="numeric"
                />
              </label>
              <label className="grid gap-1 text-sm">
                Correo
                <Input
                  type="email"
                  value={form.email}
                  onChange={(e) => setForm((f) => ({ ...f, email: e.target.value }))}
                  placeholder="correo@ejemplo.com"
                />
              </label>
              <label className="grid gap-1 text-sm">
                Teléfono
                <Input
                  value={form.phone}
                  onChange={(e) => setForm((f) => ({ ...f, phone: e.target.value }))}
                  placeholder="+54 11 …"
                />
              </label>
              <label className="grid gap-1 text-sm sm:col-span-2">
                Notas
                <textarea
                  className="min-h-24 rounded-xl border border-border bg-card px-3 py-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
                  value={form.notes}
                  onChange={(e) => setForm((f) => ({ ...f, notes: e.target.value }))}
                />
              </label>
              <div className="flex flex-wrap gap-2 sm:col-span-2">
                <Button type="submit" disabled={saving}>
                  {saving ? "Guardando…" : creating ? "Crear" : "Guardar"}
                </Button>
                {!creating && selected ? (
                  <Button
                    type="button"
                    variant="danger"
                    disabled={saving}
                    onClick={() => void onDelete(selected.id)}
                  >
                    Eliminar
                  </Button>
                ) : null}
              </div>
            </form>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="mx-auto flex max-w-5xl flex-col gap-4">
      <Card>
        <CardHeader>
          <div>
            <CardTitle>Proveedores</CardTitle>
            <CardDescription>{providers.length} proveedores · clic para abrir la ficha</CardDescription>
          </div>
          <Button
            type="button"
            variant="outline"
            size="sm"
            aria-label="Nuevo proveedor"
            title="Nuevo proveedor"
            onClick={startCreate}
          >
            <Plus className="h-4 w-4" />
          </Button>
        </CardHeader>
        <CardContent>
          {providers.length === 0 ? (
            <EmptyState
              icon={Users}
              title="Sin proveedores"
              description="Agregá el primero para asociarlo a tus facturas."
              actionLabel="Nuevo proveedor"
              onAction={startCreate}
            />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Nombre</TableHead>
                  <TableHead>CUIT</TableHead>
                  <TableHead>Teléfono</TableHead>
                  <TableHead>Correo</TableHead>
                  <TableHead>CBU</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {providers.map((provider) => (
                  <TableRow
                    key={provider.id}
                    className="cursor-pointer"
                    onClick={() => openProvider(provider.id)}
                  >
                    <TableCell className="font-medium">
                      <div>{provider.name}</div>
                      {provider.alias ? (
                        <div className="text-xs text-muted-foreground">{provider.alias}</div>
                      ) : null}
                    </TableCell>
                    <TableCell>{provider.cuit ?? "—"}</TableCell>
                    <TableCell>{provider.phone ?? "—"}</TableCell>
                    <TableCell className="max-w-[12rem] truncate">{provider.email ?? "—"}</TableCell>
                    <TableCell className="max-w-[10rem] truncate font-mono text-xs">
                      {provider.cbu ?? "—"}
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
