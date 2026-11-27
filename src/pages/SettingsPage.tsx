import { useEffect, useState, type FormEvent } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { FolderOpen, HardDrive, Monitor, Moon, ScanText, Sun } from "lucide-react";
import { api, errMessage } from "@/api";
import { useAppShell } from "@/app/AppShellContext";
import { Alert } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { useTheme, type ThemeMode } from "@/theme/ThemeProvider";
import { cn } from "@/lib/utils";
import type { AppPaths, EntityKind } from "@/types";

const themeOptions: { id: ThemeMode; label: string; description: string; icon: typeof Sun }[] = [
  { id: "light", label: "Claro", description: "Fondos claros y acentos discretos.", icon: Sun },
  { id: "dark", label: "Oscuro", description: "Para uso prolongado.", icon: Moon },
  { id: "system", label: "Automático", description: "Sigue Windows / sistema.", icon: Monitor },
];

export function SettingsPage() {
  const {
    entities,
    activeEntityId,
    refreshEntities,
    setActiveEntityId,
    setError,
    bumpRefresh,
    workspaceMode,
    setWorkspaceMode,
    runProcessing,
    processing,
  } = useAppShell();
  const { theme, setTheme, resolvedTheme } = useTheme();
  const [name, setName] = useState("");
  const [kind, setKind] = useState<EntityKind>("personal");
  const [taxId, setTaxId] = useState("");
  const [paths, setPaths] = useState<AppPaths | null>(null);
  const [documentsDir, setDocumentsDir] = useState("");
  const [backupsDir, setBackupsDir] = useState("");
  const [backups, setBackups] = useState<string[]>([]);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void (async () => {
      try {
        const current = await api.getPaths();
        setPaths(current);
        setDocumentsDir(current.documentsDir);
        setBackupsDir(current.backupsDir);
        setBackups(await api.listBackups());
      } catch (err) {
        setError(errMessage(err));
      }
    })();
  }, [setError]);

  async function pickDocumentsDir() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected || Array.isArray(selected)) return;
    setDocumentsDir(selected);
  }

  async function pickBackupsDir() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected || Array.isArray(selected)) return;
    setBackupsDir(selected);
  }

  async function savePaths() {
    setBusy(true);
    try {
      const updated = await api.updatePaths({
        documentsDir,
        backupsDir,
      });
      setPaths(updated);
      setDocumentsDir(updated.documentsDir);
      setBackupsDir(updated.backupsDir);
      setMessage("Rutas locales actualizadas");
    } catch (err) {
      setError(errMessage(err));
    } finally {
      setBusy(false);
    }
  }

  async function reprocessInvoices() {
    if (!activeEntityId) {
      setError("Seleccioná una entidad activa para reprocesar");
      return;
    }
    if (
      !confirm(
        "¿Reprocesar las facturas de la entidad activa con QR ARCA y OCR? Se completarán datos vacíos y se crearán proveedores por CUIT.",
      )
    ) {
      return;
    }
    try {
      const report = await runProcessing("Reprocesando facturas…", async () => {
        const r = await api.reprocessDocuments(activeEntityId);
        if (r.documentIds.length > 0) {
          const { enrichDocumentsWithPdfJs } = await import("@/lib/enrichFromPdf");
          await enrichDocumentsWithPdfJs(r.documentIds);
        }
        return r;
      });
      bumpRefresh();
      setMessage(
        `Reproceso: ${report.imported} docs · ${report.qrParsed} QR · ${report.ocrUsed} OCR · ${report.providersCreated} proveedores nuevos`,
      );
      if (report.errors.length > 0) {
        setError(report.errors.slice(0, 3).join(" · "));
      }
    } catch (err) {
      setError(errMessage(err));
    }
  }

  async function createEntity(event: FormEvent) {
    event.preventDefault();
    try {
      const created = await api.createEntity({
        name,
        kind,
        taxId: taxId || null,
      });
      setName("");
      setTaxId("");
      await refreshEntities();
      setActiveEntityId(created.id);
      bumpRefresh();
      setMessage(`Entidad creada: ${created.name}`);
    } catch (err) {
      setError(errMessage(err));
    }
  }

  async function deleteEntity(id: string, label: string) {
    if (!confirm(`¿Eliminar "${label}" y sus datos asociados en Folio?`)) return;
    try {
      await api.deleteEntity(id);
      await refreshEntities();
      bumpRefresh();
    } catch (err) {
      setError(errMessage(err));
    }
  }

  async function createBackup() {
    setBusy(true);
    try {
      const path = await api.createBackup();
      setBackups(await api.listBackups());
      setMessage(`Backup creado: ${path}`);
    } catch (err) {
      setError(errMessage(err));
    } finally {
      setBusy(false);
    }
  }

  async function restoreBackup(path?: string) {
    let backupPath = path;
    if (!backupPath) {
      const selected = await open({
        multiple: false,
        filters: [{ name: "Backup Folio", extensions: ["zip"] }],
      });
      if (!selected || Array.isArray(selected)) return;
      backupPath = selected;
    }
    if (!confirm("La restauración reemplaza la base y los documentos archivados. ¿Continuar?")) {
      return;
    }
    setBusy(true);
    try {
      await api.restoreBackup(backupPath);
      await refreshEntities();
      bumpRefresh();
      setBackups(await api.listBackups());
      setMessage("Restauración completada");
    } catch (err) {
      setError(errMessage(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="mx-auto flex max-w-4xl flex-col gap-4">
      {message ? <Alert variant="success">{message}</Alert> : null}

      <Card>
        <CardHeader>
          <div>
            <CardTitle>Modo de uso</CardTitle>
            <CardDescription>
              Folio organiza facturas; no es un sistema contable. Elegí cuánta gestión mostrar.
            </CardDescription>
          </div>
        </CardHeader>
        <CardContent className="grid gap-3">
          {(
            [
              {
                id: "invoices" as const,
                label: "Solo facturas",
                description:
                  "Importar, buscar, ver PDF y proveedores. Oculta Pagos e Informes.",
              },
              {
                id: "management" as const,
                label: "Gestión completa",
                description: "Incluye pagos, vencimientos e informes CSV.",
              },
            ] as const
          ).map((option) => {
            const active = workspaceMode === option.id;
            return (
              <button
                key={option.id}
                type="button"
                onClick={() => void setWorkspaceMode(option.id)}
                className={cn(
                  "flex items-start gap-3 rounded-2xl border px-4 py-3 text-left transition",
                  active ? "border-brand-blue bg-brand-blue/5" : "border-border hover:bg-muted/50",
                )}
              >
                <div>
                  <p className="text-sm font-semibold">{option.label}</p>
                  <p className="mt-1 text-sm text-muted-foreground">{option.description}</p>
                </div>
              </button>
            );
          })}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div>
            <CardTitle>Apariencia</CardTitle>
            <CardDescription>
              Tema: {theme} · resuelto: {resolvedTheme}
            </CardDescription>
          </div>
        </CardHeader>
        <CardContent className="grid gap-3">
          {themeOptions.map((option) => {
            const Icon = option.icon;
            const active = theme === option.id;
            return (
              <button
                key={option.id}
                type="button"
                onClick={() => setTheme(option.id)}
                className={cn(
                  "flex items-start gap-3 rounded-2xl border px-4 py-3 text-left transition",
                  active ? "border-brand-blue bg-brand-blue/5" : "border-border hover:bg-muted/50",
                )}
              >
                <div className="mt-0.5 flex h-9 w-9 items-center justify-center rounded-xl bg-muted">
                  <Icon className="h-4 w-4" />
                </div>
                <div>
                  <p className="text-sm font-semibold">{option.label}</p>
                  <p className="mt-1 text-sm text-muted-foreground">{option.description}</p>
                </div>
              </button>
            );
          })}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div>
            <CardTitle>QR ARCA + OCR</CardTitle>
            <CardDescription>
              Reprocesá facturas ya cargadas: lee QR AFIP/ARCA, texto/OCR y crea proveedores por
              CUIT.
            </CardDescription>
          </div>
        </CardHeader>
        <CardContent className="space-y-3">
          <Alert>
            Al importar también se intenta leer el QR y el texto del PDF. Usá este botón para
            completar facturas ya cargadas sin CUIT/proveedor.
          </Alert>
          <Button
            className="w-full sm:w-auto"
            disabled={busy || processing.active || !activeEntityId}
            onClick={() => void reprocessInvoices()}
          >
            <ScanText className="h-4 w-4" />
            {processing.active ? "Reprocesando…" : "Reprocesar facturas de la entidad activa"}
          </Button>
          {!activeEntityId ? (
            <p className="text-xs text-muted-foreground">Activá una entidad para habilitar el botón.</p>
          ) : null}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div>
            <CardTitle>Entidades</CardTitle>
            <CardDescription>Personal y empresas con aislamiento de documentos.</CardDescription>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <form className="grid gap-3 md:grid-cols-4" onSubmit={(e) => void createEntity(e)}>
            <Input
              placeholder="Nombre"
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
            />
            <Select value={kind} onChange={(e) => setKind(e.target.value as EntityKind)}>
              <option value="personal">Personal</option>
              <option value="company">Empresa</option>
            </Select>
            <Input
              placeholder="CUIT (opcional)"
              value={taxId}
              onChange={(e) => setTaxId(e.target.value)}
            />
            <Button type="submit">Crear</Button>
          </form>

          <div className="space-y-2">
            {entities.map((entity) => (
              <div
                key={entity.id}
                className="flex items-center justify-between gap-3 rounded-2xl border border-border px-3 py-3"
              >
                <div>
                  <p className="text-sm font-medium">{entity.name}</p>
                  <p className="text-xs text-muted-foreground">
                    {entity.kind === "company" ? "Empresa" : "Personal"}
                    {entity.taxId ? ` · ${entity.taxId}` : ""}
                  </p>
                </div>
                <div className="flex gap-2">
                  <Button size="sm" variant="outline" onClick={() => setActiveEntityId(entity.id)}>
                    Activar
                  </Button>
                  <Button
                    size="sm"
                    variant="danger"
                    onClick={() => void deleteEntity(entity.id, entity.name)}
                  >
                    Eliminar
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div>
            <CardTitle>Backup</CardTitle>
            <CardDescription>Copia local de la base y documentos archivados.</CardDescription>
          </div>
          <div className="flex gap-2">
            <Button disabled={busy} onClick={() => void createBackup()}>
              <HardDrive className="h-4 w-4" />
              Crear backup
            </Button>
            <Button variant="outline" disabled={busy} onClick={() => void restoreBackup()}>
              Restaurar
            </Button>
          </div>
        </CardHeader>
        <CardContent className="space-y-2">
          {backups.length === 0 ? (
            <p className="text-sm text-muted-foreground">Todavía no hay copias.</p>
          ) : (
            backups.map((item) => (
              <div
                key={item}
                className="flex items-center justify-between gap-3 rounded-xl border border-border px-3 py-2 text-sm"
              >
                <span className="truncate">{item}</span>
                <Button size="sm" variant="outline" onClick={() => void restoreBackup(item)}>
                  Restaurar
                </Button>
              </div>
            ))
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div>
            <CardTitle>Rutas locales</CardTitle>
            <CardDescription>
              Podés cambiar dónde se guardan documentos y backups. La base SQLite permanece en la
              carpeta de datos de la app.
            </CardDescription>
          </div>
          <Button disabled={busy} onClick={() => void savePaths()}>
            Guardar rutas
          </Button>
        </CardHeader>
        <CardContent className="grid gap-4 text-sm">
          <div className="grid gap-1">
            <span className="text-muted-foreground">Datos / base (fijo)</span>
            <p className="rounded-xl border border-border bg-muted/40 px-3 py-2">
              {paths?.dataDir ?? "…"}
            </p>
            <p className="text-xs text-muted-foreground">{paths?.dbPath ?? ""}</p>
          </div>

          <label className="grid gap-2">
            <span className="font-medium">Carpeta de documentos</span>
            <div className="flex gap-2">
              <Input value={documentsDir} onChange={(e) => setDocumentsDir(e.target.value)} />
              <Button type="button" variant="outline" onClick={() => void pickDocumentsDir()}>
                <FolderOpen className="h-4 w-4" />
              </Button>
            </div>
          </label>

          <label className="grid gap-2">
            <span className="font-medium">Carpeta de backups</span>
            <div className="flex gap-2">
              <Input value={backupsDir} onChange={(e) => setBackupsDir(e.target.value)} />
              <Button type="button" variant="outline" onClick={() => void pickBackupsDir()}>
                <FolderOpen className="h-4 w-4" />
              </Button>
            </div>
          </label>
        </CardContent>
      </Card>

    </div>
  );
}
