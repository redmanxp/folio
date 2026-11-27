use crate::error::{AppError, AppResult};
use rusqlite::{params, Connection};
use std::path::Path;

pub fn open_db(db_path: &Path) -> AppResult<Connection> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        ",
    )?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );
        ",
    )?;

    let current: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current < 1 {
        conn.execute_batch(
            "
            CREATE TABLE entities (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                kind TEXT NOT NULL CHECK (kind IN ('personal', 'company')),
                tax_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE providers (
                id TEXT PRIMARY KEY,
                entity_id TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                cuit TEXT,
                alias TEXT,
                notes TEXT,
                created_at TEXT NOT NULL
            );

            CREATE UNIQUE INDEX IF NOT EXISTS idx_providers_entity_cuit
                ON providers(entity_id, cuit) WHERE cuit IS NOT NULL AND cuit != '';

            CREATE TABLE documents (
                id TEXT PRIMARY KEY,
                entity_id TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
                provider_id TEXT REFERENCES providers(id) ON DELETE SET NULL,
                file_name TEXT NOT NULL,
                original_path TEXT,
                stored_path TEXT NOT NULL,
                file_hash TEXT NOT NULL,
                mime_type TEXT,
                doc_type TEXT,
                number TEXT,
                period TEXT,
                issue_date TEXT,
                due_date TEXT,
                amount REAL,
                currency TEXT NOT NULL DEFAULT 'ARS',
                notes TEXT,
                imported_at TEXT NOT NULL,
                UNIQUE(entity_id, file_hash)
            );

            CREATE INDEX IF NOT EXISTS idx_documents_entity ON documents(entity_id);
            CREATE INDEX IF NOT EXISTS idx_documents_period ON documents(entity_id, period);
            CREATE INDEX IF NOT EXISTS idx_documents_number ON documents(entity_id, number);

            CREATE TABLE payments (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL UNIQUE REFERENCES documents(id) ON DELETE CASCADE,
                entity_id TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
                status TEXT NOT NULL CHECK (status IN ('pending', 'partial', 'paid')),
                amount_due REAL,
                amount_paid REAL NOT NULL DEFAULT 0,
                paid_at TEXT,
                notes TEXT,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_payments_status ON payments(entity_id, status);

            CREATE TABLE settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            ",
        )?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, datetime('now'))",
            params![1],
        )?;
        log::info!("Migración schema v1 aplicada");
    }

    let current: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current < 2 {
        conn.execute_batch(
            "
            ALTER TABLE documents ADD COLUMN point_of_sale INTEGER;
            ALTER TABLE documents ADD COLUMN fx_rate REAL;
            ALTER TABLE documents ADD COLUMN auth_type TEXT;
            ALTER TABLE documents ADD COLUMN auth_code TEXT;
            ALTER TABLE documents ADD COLUMN receptor_doc_type INTEGER;
            ALTER TABLE documents ADD COLUMN receptor_doc_number TEXT;
            ",
        )?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, datetime('now'))",
            params![2],
        )?;
        log::info!("Migración schema v2 aplicada (campos QR ARCA)");
    }

    let current: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current < 3 {
        conn.execute_batch(
            "
            ALTER TABLE providers ADD COLUMN cbu TEXT;
            ALTER TABLE providers ADD COLUMN email TEXT;
            ALTER TABLE providers ADD COLUMN phone TEXT;
            ",
        )?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, datetime('now'))",
            params![3],
        )?;
        log::info!("Migración schema v3 aplicada (contacto proveedor)");
    }

    Ok(())
}

pub fn get_setting(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn ensure_entity_exists(conn: &Connection, entity_id: &str) -> AppResult<()> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM entities WHERE id = ?1",
        params![entity_id],
        |row| row.get(0),
    )?;
    if count == 0 {
        return Err(AppError::msg("La entidad no existe o no está seleccionada"));
    }
    Ok(())
}

pub fn ensure_provider_in_entity(
    conn: &Connection,
    entity_id: &str,
    provider_id: &str,
) -> AppResult<()> {
    ensure_entity_exists(conn, entity_id)?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM providers WHERE id = ?1 AND entity_id = ?2",
        params![provider_id, entity_id],
        |row| row.get(0),
    )?;
    if count == 0 {
        return Err(AppError::msg(
            "El proveedor no existe en la entidad seleccionada",
        ));
    }
    Ok(())
}

pub fn ensure_document_in_entity(
    conn: &Connection,
    entity_id: &str,
    document_id: &str,
) -> AppResult<()> {
    ensure_entity_exists(conn, entity_id)?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM documents WHERE id = ?1 AND entity_id = ?2",
        params![document_id, entity_id],
        |row| row.get(0),
    )?;
    if count == 0 {
        return Err(AppError::msg(
            "El documento no existe en la entidad seleccionada",
        ));
    }
    Ok(())
}

/// Si se informa provider_id, debe pertenecer a la misma entidad del documento.
pub fn ensure_optional_provider_same_entity(
    conn: &Connection,
    entity_id: &str,
    provider_id: &Option<String>,
) -> AppResult<()> {
    if let Some(pid) = provider_id
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        ensure_provider_in_entity(conn, entity_id, pid)?;
    }
    Ok(())
}
