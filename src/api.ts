import { invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  AppPaths,
  DashboardStats,
  Document,
  Entity,
  ImportReport,
  NewEntity,
  NewProvider,
  Payment,
  Provider,
  UpdateProvider,
  SearchFilters,
  SearchResult,
  UpdatePayment,
} from "@/types";

export const api = {
  listEntities: () => invoke<Entity[]>("list_entities"),
  createEntity: (payload: NewEntity) => invoke<Entity>("create_entity", { payload }),
  updateEntity: (id: string, payload: NewEntity) =>
    invoke<Entity>("update_entity", { id, payload }),
  deleteEntity: (id: string) => invoke<void>("delete_entity", { id }),

  listProviders: (entityId: string) => invoke<Provider[]>("list_providers", { entityId }),
  createProvider: (payload: NewProvider) => invoke<Provider>("create_provider", { payload }),
  updateProvider: (payload: UpdateProvider) => invoke<Provider>("update_provider", { payload }),
  deleteProvider: (entityId: string, id: string) =>
    invoke<void>("delete_provider", { entityId, id }),

  getDocument: (entityId: string, id: string) =>
    invoke<Document>("get_document", { entityId, id }),
  updateDocumentMeta: (args: {
    entityId: string;
    id: string;
    number?: string | null;
    period?: string | null;
    issueDate?: string | null;
    dueDate?: string | null;
    amount?: number | null;
    providerId?: string | null;
    notes?: string | null;
    docType?: string | null;
  }) => invoke<Document>("update_document_meta", args),
  deleteDocument: (entityId: string, id: string, deleteFile: boolean) =>
    invoke<void>("delete_document", { entityId, id, deleteFile }),
  listRecentDocuments: (entityId: string, limit = 20) =>
    invoke<Document[]>("list_recent_documents", { entityId, limit }),

  importFiles: (entityId: string, paths: string[]) =>
    invoke<ImportReport>("import_files", { entityId, paths }),
  reprocessDocuments: (entityId: string) =>
    invoke<ImportReport>("reprocess_documents", { entityId }),
  reprocessDocument: (documentId: string) =>
    invoke<ImportReport>("reprocess_document", { documentId }),
  enrichDocumentFromText: (documentId: string, text: string) =>
    invoke<Document>("enrich_document_from_text", { documentId, text }),

  searchDocuments: (filters: SearchFilters) =>
    invoke<SearchResult>("search_documents", { filters }),
  exportSearchCsv: (filters: SearchFilters, targetPath: string) =>
    invoke<string>("export_search_csv", { filters, targetPath }),

  listPayments: (entityId: string, status?: string | null) =>
    invoke<Payment[]>("list_payments", { entityId, status: status ?? null }),
  updatePayment: (payload: UpdatePayment) => invoke<Payment>("update_payment", { payload }),
  listAgenda: (entityId: string) => invoke<Payment[]>("list_agenda", { entityId }),

  createBackup: () => invoke<string>("create_backup"),
  restoreBackup: (backupPath: string) => invoke<void>("restore_backup", { backupPath }),
  listBackups: () => invoke<string[]>("list_backups"),
  deleteBackup: (backupPath: string) => invoke<void>("delete_backup", { backupPath }),

  getAppInfo: () => invoke<AppInfo>("get_app_info"),
  getPaths: () => invoke<AppPaths>("get_paths"),
  updatePaths: (payload: { documentsDir?: string | null; backupsDir?: string | null }) =>
    invoke<AppPaths>("update_paths", { payload }),
  getSetting: (key: string) => invoke<string | null>("get_setting", { key }),
  setSetting: (key: string, value: string) => invoke<void>("set_setting", { key, value }),
  getDashboardStats: (entityId?: string | null) =>
    invoke<DashboardStats>("get_dashboard_stats", { entityId: entityId ?? null }),
};

export function errMessage(err: unknown) {
  return err instanceof Error ? err.message : String(err);
}
