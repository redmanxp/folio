# Arquitectura de Folio

## Stack
- **Frontend:** React + TypeScript (Vite)
- **Shell:** Tauri 2
- **Backend:** Rust
- **Datos:** SQLite (`rusqlite`, bundled)

## Capas
1. **UI (React):** pantallas por dominio (entidades, importación, búsqueda, pagos…).
2. **Bridge:** comandos Tauri (`invoke`) tipados en `src/api.ts`.
3. **Dominio Rust:** módulos en `src-tauri/src/commands/*`.
4. **Persistencia:** SQLite + archivos en `%APPDATA%/com.folio.app/`.

## Datos en disco
```
app_data_dir/
  folio.db
  documents/<entity_id>/...
  backups/folio_backup_YYYYMMDD_HHMMSS.zip
```

## Aislamiento
Toda consulta de documentos/proveedores/pagos exige `entity_id`.
Los archivos se guardan en subcarpetas por entidad.
El índice único `(entity_id, file_hash)` evita duplicados **dentro** de la misma entidad.

## Comandos principales
- Entidades: `list_entities`, `create_entity`, `update_entity`, `delete_entity`
- Importación: `import_files`
- Documentos: `get_document`, `update_document_meta`, `delete_document`, `list_recent_documents`
- Búsqueda/export: `search_documents`, `export_search_csv`
- Proveedores / pagos / agenda
- Backup: `create_backup`, `restore_backup`, `list_backups`
- Config: `get_paths`, `get_setting`, `set_setting`, `get_dashboard_stats`

## Decisiones
- Sin servidor propio.
- Sin telemetría.
- CSV como primera exportación tabular (Excel compatible); XLSX nativo puede llegar después.
- QR/OCR: etapa posterior sobre la importación actual.
