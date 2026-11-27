export type EntityKind = "personal" | "company";

export interface Entity {
  id: string;
  name: string;
  kind: EntityKind;
  taxId?: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface NewEntity {
  name: string;
  kind: EntityKind;
  taxId?: string | null;
}

export interface Provider {
  id: string;
  entityId: string;
  name: string;
  cuit?: string | null;
  alias?: string | null;
  cbu?: string | null;
  email?: string | null;
  phone?: string | null;
  notes?: string | null;
  createdAt: string;
}

export interface NewProvider {
  entityId: string;
  name: string;
  cuit?: string | null;
  alias?: string | null;
  cbu?: string | null;
  email?: string | null;
  phone?: string | null;
  notes?: string | null;
}

export interface UpdateProvider {
  id: string;
  name: string;
  cuit?: string | null;
  alias?: string | null;
  cbu?: string | null;
  email?: string | null;
  phone?: string | null;
  notes?: string | null;
}

export interface Document {
  id: string;
  entityId: string;
  providerId?: string | null;
  fileName: string;
  originalPath?: string | null;
  storedPath: string;
  fileHash: string;
  mimeType?: string | null;
  docType?: string | null;
  number?: string | null;
  period?: string | null;
  issueDate?: string | null;
  dueDate?: string | null;
  amount?: number | null;
  currency: string;
  pointOfSale?: number | null;
  fxRate?: number | null;
  authType?: string | null;
  authCode?: string | null;
  receptorDocType?: number | null;
  receptorDocNumber?: string | null;
  notes?: string | null;
  importedAt: string;
  providerName?: string | null;
  paymentStatus?: string | null;
}

export interface SearchFilters {
  entityId: string;
  query?: string | null;
  provider?: string | null;
  cuit?: string | null;
  number?: string | null;
  period?: string | null;
  paymentStatus?: string | null;
  fromDate?: string | null;
  toDate?: string | null;
}

export interface SearchResult {
  documents: Document[];
  totalAmount: number;
  count: number;
}

export interface Payment {
  id: string;
  documentId: string;
  entityId: string;
  status: string;
  amountDue?: number | null;
  amountPaid: number;
  paidAt?: string | null;
  notes?: string | null;
  updatedAt: string;
  fileName?: string | null;
  providerName?: string | null;
  dueDate?: string | null;
}

export interface UpdatePayment {
  documentId: string;
  status: string;
  amountDue?: number | null;
  amountPaid?: number | null;
  paidAt?: string | null;
  notes?: string | null;
}

export interface ImportReport {
  imported: number;
  duplicates: number;
  errors: string[];
  documentIds: string[];
  qrParsed: number;
  ocrUsed: number;
  providersCreated: number;
}

export interface AppPaths {
  dataDir: string;
  dbPath: string;
  documentsDir: string;
  backupsDir: string;
}

export interface AppInfo {
  name: string;
  version: string;
  description: string;
}

export interface DashboardStats {
  entities: number;
  documents: number;
  providers: number;
  pendingPayments: number;
  dueSoon: number;
}
