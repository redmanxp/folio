import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { api, errMessage } from "@/api";
import type { AppSection } from "@/app/navigation";
import type { Entity } from "@/types";

const ACTIVE_KEY = "folio.activeEntityId";
const WORKSPACE_MODE_KEY = "ui.workspaceMode";

export type WorkspaceMode = "invoices" | "management";

type ProcessingState = {
  active: boolean;
  label: string;
};

type AppShellContextValue = {
  section: AppSection;
  setSection: (section: AppSection) => void;
  entities: Entity[];
  activeEntityId: string | null;
  setActiveEntityId: (id: string | null) => void;
  activeEntity: Entity | null;
  selectedDocumentId: string | null;
  openDocument: (id: string) => void;
  clearSelectedDocument: () => void;
  selectedProviderId: string | null;
  openProvider: (id: string, returnTo?: AppSection) => void;
  closeProvider: () => void;
  clearSelectedProvider: () => void;
  loading: boolean;
  error: string | null;
  setError: (message: string | null) => void;
  refreshEntities: () => Promise<void>;
  refreshKey: number;
  bumpRefresh: () => void;
  processing: ProcessingState;
  runProcessing: <T>(label: string, task: () => Promise<T>) => Promise<T>;
  workspaceMode: WorkspaceMode;
  setWorkspaceMode: (mode: WorkspaceMode) => Promise<void>;
};

const AppShellContext = createContext<AppShellContextValue | null>(null);

export function AppShellProvider({ children }: { children: ReactNode }) {
  const [section, setSectionState] = useState<AppSection>("home");
  const [entities, setEntities] = useState<Entity[]>([]);
  const [activeEntityId, setActiveEntityIdState] = useState<string | null>(null);
  const [selectedDocumentId, setSelectedDocumentId] = useState<string | null>(null);
  const [selectedProviderId, setSelectedProviderId] = useState<string | null>(null);
  const [providerReturnSection, setProviderReturnSection] = useState<AppSection | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  const [processing, setProcessing] = useState<ProcessingState>({
    active: false,
    label: "",
  });
  const [workspaceMode, setWorkspaceModeState] = useState<WorkspaceMode>("invoices");

  const bumpRefresh = useCallback(() => setRefreshKey((value) => value + 1), []);

  const setSection = useCallback(
    (next: AppSection) => {
      if (
        workspaceMode === "invoices" &&
        (next === "payments" || next === "reports")
      ) {
        setSelectedProviderId(null);
        setProviderReturnSection(null);
        setSectionState("invoices");
        return;
      }
      // Navegación del menú: vuelve al listado (openDocument/openProvider usan setSectionState)
      setSelectedProviderId(null);
      setProviderReturnSection(null);
      if (next !== "documents") setSelectedDocumentId(null);
      setSectionState(next);
    },
    [workspaceMode],
  );

  const refreshEntities = useCallback(async () => {
    try {
      const list = await api.listEntities();
      setEntities(list);
      setActiveEntityIdState((current) => {
        if (current && list.some((e) => e.id === current)) return current;
        const saved = localStorage.getItem(ACTIVE_KEY);
        if (saved && list.some((e) => e.id === saved)) return saved;
        return list[0]?.id ?? null;
      });
      setError(null);
    } catch (err) {
      setError(errMessage(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refreshEntities();
  }, [refreshEntities]);

  useEffect(() => {
    void (async () => {
      try {
        const saved = await api.getSetting(WORKSPACE_MODE_KEY);
        if (saved === "management" || saved === "invoices") {
          setWorkspaceModeState(saved);
        }
      } catch {
        // preferencia local por defecto
      }
    })();
  }, []);

  useEffect(() => {
    if (
      workspaceMode === "invoices" &&
      (section === "payments" || section === "reports")
    ) {
      setSectionState("invoices");
    }
  }, [workspaceMode, section]);

  const setActiveEntityId = useCallback((id: string | null) => {
    setActiveEntityIdState(id);
    if (id) localStorage.setItem(ACTIVE_KEY, id);
    else localStorage.removeItem(ACTIVE_KEY);
  }, []);

  const openDocument = useCallback((id: string) => {
    setSelectedDocumentId(id);
    setSelectedProviderId(null);
    setProviderReturnSection(null);
    setSectionState("documents");
  }, []);

  const clearSelectedDocument = useCallback(() => setSelectedDocumentId(null), []);

  const openProvider = useCallback((id: string, returnTo: AppSection = "providers") => {
    setSelectedProviderId(id);
    setProviderReturnSection(returnTo);
    setSelectedDocumentId(null);
    setSectionState("providers");
  }, []);

  const closeProvider = useCallback(() => {
    const back = providerReturnSection ?? "providers";
    setSelectedProviderId(null);
    setProviderReturnSection(null);
    setSectionState(back);
  }, [providerReturnSection]);

  const clearSelectedProvider = useCallback(() => {
    setSelectedProviderId(null);
    setProviderReturnSection(null);
  }, []);

  const runProcessing = useCallback(async <T,>(label: string, task: () => Promise<T>) => {
    setProcessing({ active: true, label });
    try {
      return await task();
    } finally {
      setProcessing({ active: false, label: "" });
    }
  }, []);

  const setWorkspaceMode = useCallback(async (mode: WorkspaceMode) => {
    setWorkspaceModeState(mode);
    try {
      await api.setSetting(WORKSPACE_MODE_KEY, mode);
    } catch (err) {
      setError(errMessage(err));
    }
  }, []);

  const activeEntity = useMemo(
    () => entities.find((entity) => entity.id === activeEntityId) ?? null,
    [entities, activeEntityId],
  );

  const value = useMemo(
    () => ({
      section,
      setSection,
      entities,
      activeEntityId,
      setActiveEntityId,
      activeEntity,
      selectedDocumentId,
      openDocument,
      clearSelectedDocument,
      selectedProviderId,
      openProvider,
      closeProvider,
      clearSelectedProvider,
      loading,
      error,
      setError,
      refreshEntities,
      refreshKey,
      bumpRefresh,
      processing,
      runProcessing,
      workspaceMode,
      setWorkspaceMode,
    }),
    [
      section,
      setSection,
      entities,
      activeEntityId,
      setActiveEntityId,
      activeEntity,
      selectedDocumentId,
      openDocument,
      clearSelectedDocument,
      selectedProviderId,
      openProvider,
      closeProvider,
      clearSelectedProvider,
      loading,
      error,
      refreshEntities,
      refreshKey,
      bumpRefresh,
      processing,
      runProcessing,
      workspaceMode,
      setWorkspaceMode,
    ],
  );

  return <AppShellContext.Provider value={value}>{children}</AppShellContext.Provider>;
}

export function useAppShell() {
  const ctx = useContext(AppShellContext);
  if (!ctx) throw new Error("useAppShell debe usarse dentro de AppShellProvider");
  return ctx;
}
