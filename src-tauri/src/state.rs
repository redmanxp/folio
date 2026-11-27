use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::AppPaths;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    paths: Arc<Mutex<AppPathsInternal>>,
}

#[derive(Clone)]
pub struct AppPathsInternal {
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
    pub documents_dir: PathBuf,
    pub backups_dir: PathBuf,
}

impl AppState {
    pub fn init(data_dir: PathBuf) -> AppResult<Self> {
        let db_path = data_dir.join("folio.db");
        std::fs::create_dir_all(&data_dir)?;

        log::info!("Inicializando Folio en {:?}", data_dir);
        let conn = db::open_db(&db_path)?;

        let documents_dir = db::get_setting(&conn, "path.documents_dir")?
            .map(PathBuf::from)
            .unwrap_or_else(|| data_dir.join("documents"));
        let backups_dir = db::get_setting(&conn, "path.backups_dir")?
            .map(PathBuf::from)
            .unwrap_or_else(|| data_dir.join("backups"));

        std::fs::create_dir_all(&documents_dir)?;
        std::fs::create_dir_all(&backups_dir)?;

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            paths: Arc::new(Mutex::new(AppPathsInternal {
                data_dir,
                db_path,
                documents_dir,
                backups_dir,
            })),
        })
    }

    pub fn with_db<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&Connection) -> AppResult<T>,
    {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::msg("No se pudo acceder a la base de datos"))?;
        f(&conn)
    }

    pub fn paths_snapshot(&self) -> AppResult<AppPathsInternal> {
        self.paths
            .lock()
            .map(|p| p.clone())
            .map_err(|_| AppError::msg("No se pudieron leer las rutas"))
    }

    pub fn paths_public(&self) -> AppResult<AppPaths> {
        let p = self.paths_snapshot()?;
        Ok(AppPaths {
            data_dir: p.data_dir.to_string_lossy().into_owned(),
            db_path: p.db_path.to_string_lossy().into_owned(),
            documents_dir: p.documents_dir.to_string_lossy().into_owned(),
            backups_dir: p.backups_dir.to_string_lossy().into_owned(),
        })
    }

    pub fn entity_docs_dir(&self, entity_id: &str) -> AppResult<PathBuf> {
        Ok(self.paths_snapshot()?.documents_dir.join(entity_id))
    }

    pub fn update_local_paths(
        &self,
        documents_dir: Option<String>,
        backups_dir: Option<String>,
    ) -> AppResult<AppPaths> {
        if let Some(docs) = documents_dir {
            let path = PathBuf::from(docs.trim());
            if path.as_os_str().is_empty() {
                return Err(AppError::msg("La ruta de documentos no puede estar vacía"));
            }
            std::fs::create_dir_all(&path)?;
            {
                let mut paths = self
                    .paths
                    .lock()
                    .map_err(|_| AppError::msg("No se pudieron actualizar las rutas"))?;
                paths.documents_dir = path.clone();
            }
            self.with_db(|conn| {
                db::set_setting(conn, "path.documents_dir", &path.to_string_lossy())
            })?;
            log::info!("Ruta de documentos actualizada: {:?}", path);
        }

        if let Some(backs) = backups_dir {
            let path = PathBuf::from(backs.trim());
            if path.as_os_str().is_empty() {
                return Err(AppError::msg("La ruta de backups no puede estar vacía"));
            }
            std::fs::create_dir_all(&path)?;
            {
                let mut paths = self
                    .paths
                    .lock()
                    .map_err(|_| AppError::msg("No se pudieron actualizar las rutas"))?;
                paths.backups_dir = path.clone();
            }
            self.with_db(|conn| {
                db::set_setting(conn, "path.backups_dir", &path.to_string_lossy())
            })?;
            log::info!("Ruta de backups actualizada: {:?}", path);
        }

        self.paths_public()
    }
}
