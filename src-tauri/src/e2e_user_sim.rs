//! Simulación de recorrido de usuario (sin UI) sobre un data dir temporal.
#[cfg(test)]
mod tests {
    use crate::commands::backup::{create_backup_for_test, restore_backup_for_test};
    use crate::commands::import::import_files_sync;
    use crate::models::NewProvider;
    use crate::state::AppState;
    use crate::util::{new_id, now_iso};
    use rusqlite::params;
    use std::fs;
    use std::path::PathBuf;

    fn temp_data_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("folio-e2e-{}", new_id()));
        fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    fn write_demo_pdf(path: &std::path::Path) {
        let content = b"%PDF-1.4
1 0 obj<< /Type /Catalog /Pages 2 0 R >>endobj
2 0 obj<< /Type /Pages /Kids [3 0 R] /Count 1 >>endobj
3 0 obj<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] /Contents 4 0 R /Resources<< /Font<< /F1 5 0 R >> >> >>endobj
4 0 obj<< /Length 160 >>stream
BT /F1 12 Tf 40 150 Td (FACTURA DEMO FOLIO) Tj 0 -20 Td (Proveedor: Norte Energia SA) Tj 0 -20 Td (CUIT: 30-71234567-8) Tj 0 -20 Td (Total a Pagar: $12.345,67) Tj 0 -20 Td (Vencimiento: 15/12/2026) Tj ET
endstream
endobj
5 0 obj<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>endobj
xref
0 6
0000000000 65535 f 
0000000010 00000 n 
0000000060 00000 n 
0000000115 00000 n 
0000000270 00000 n 
0000000480 00000 n 
trailer<< /Size 6 /Root 1 0 R >>
startxref
553
%%EOF
";
        fs::write(path, content).expect("write pdf");
    }

    fn create_entity(state: &AppState, name: &str, kind: &str) -> String {
        let id = new_id();
        let now = now_iso();
        state
            .with_db(|conn| {
                conn.execute(
                    "INSERT INTO entities (id, name, kind, tax_id, created_at, updated_at)
                     VALUES (?1, ?2, ?3, NULL, ?4, ?5)",
                    params![id, name, kind, now, now],
                )?;
                Ok(())
            })
            .unwrap();
        fs::create_dir_all(state.entity_docs_dir(&id).unwrap()).unwrap();
        id
    }

    fn create_provider(state: &AppState, entity_id: &str, payload: NewProvider) -> String {
        let id = new_id();
        let now = now_iso();
        state
            .with_db(|conn| {
                conn.execute(
                    "INSERT INTO providers
                     (id, entity_id, name, cuit, alias, cbu, email, phone, notes, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        id,
                        entity_id,
                        payload.name,
                        payload.cuit,
                        payload.alias,
                        payload.cbu,
                        payload.email,
                        payload.phone,
                        payload.notes,
                        now
                    ],
                )?;
                Ok(())
            })
            .unwrap();
        id
    }

    #[test]
    fn user_journey_mvp_simulado() {
        let data_dir = temp_data_dir();
        let state = AppState::init(data_dir.clone()).expect("init state");

        // 1) Usuario crea espacio de trabajo
        let entity_id = create_entity(&state, "Espacio Demo E2E", "personal");
        let entities: i64 = state
            .with_db(|conn| {
                Ok(conn.query_row("SELECT COUNT(*) FROM entities", [], |r| r.get(0))?)
            })
            .unwrap();
        assert_eq!(entities, 1);

        // 2) Alta de proveedor inventado
        let provider_id = create_provider(
            &state,
            &entity_id,
            NewProvider {
                entity_id: entity_id.clone(),
                name: "Norte Energia SA".into(),
                cuit: Some("30-71234567-8".into()),
                alias: Some("luz-demo".into()),
                cbu: Some("0170099120000001234567".into()),
                email: Some("facturacion@norte-energia.demo".into()),
                phone: Some("0800-555-1000".into()),
                notes: Some("Proveedor inventado para E2E".into()),
            },
        );
        assert!(!provider_id.is_empty());

        // 3) Importa PDF inventado
        let pdf = data_dir.join("factura-demo.pdf");
        write_demo_pdf(&pdf);
        let report = import_files_sync(
            state.clone(),
            entity_id.clone(),
            vec![pdf.to_string_lossy().into_owned()],
        )
        .expect("import");
        assert_eq!(report.imported, 1, "errors={:?}", report.errors);
        assert!(report.errors.is_empty());
        assert_eq!(report.document_ids.len(), 1);

        // 4) Reimporta el mismo archivo => duplicado
        let dup = import_files_sync(
            state.clone(),
            entity_id.clone(),
            vec![pdf.to_string_lossy().into_owned()],
        )
        .expect("reimport");
        assert_eq!(dup.duplicates, 1);
        assert_eq!(dup.imported, 0);

        // 5) Búsqueda como en Facturas
        let found: i64 = state
            .with_db(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM documents d
                     LEFT JOIN providers p ON p.id = d.provider_id
                     WHERE d.entity_id = ?1
                       AND (d.file_name LIKE '%demo%' OR IFNULL(p.name,'') LIKE '%Norte%')",
                    params![entity_id],
                    |r| r.get(0),
                )?)
            })
            .unwrap();
        assert!(found >= 1);

        // 6) Stats de inicio
        let docs: i64 = state
            .with_db(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM documents WHERE entity_id = ?1",
                    params![entity_id],
                    |r| r.get(0),
                )?)
            })
            .unwrap();
        assert!(docs >= 1);

        // 7) Backup + restore
        let backup = create_backup_for_test(&state).expect("backup");
        assert!(PathBuf::from(&backup).is_file());

        let _ = create_entity(&state, "Debe Desaparecer", "company");
        restore_backup_for_test(&state, &backup).expect("restore");
        let after: i64 = state
            .with_db(|conn| {
                Ok(conn.query_row("SELECT COUNT(*) FROM entities", [], |r| r.get(0))?)
            })
            .unwrap();
        assert_eq!(after, 1, "restore debe volver al estado del backup");

        // 8) ZIP corrupto no debe tocar datos vivos
        let bad_zip = data_dir.join("backups").join("folio_backup_corrupt.zip");
        fs::create_dir_all(bad_zip.parent().unwrap()).unwrap();
        fs::write(&bad_zip, b"esto-no-es-un-zip").unwrap();
        let before_docs: i64 = state
            .with_db(|conn| {
                Ok(conn.query_row("SELECT COUNT(*) FROM documents", [], |r| {
                    r.get::<_, i64>(0)
                })?)
            })
            .unwrap();
        let err = restore_backup_for_test(&state, &bad_zip.to_string_lossy())
            .expect_err("restore corrupto debe fallar");
        let _ = err;
        let after_fail: i64 = state
            .with_db(|conn| {
                Ok(conn.query_row("SELECT COUNT(*) FROM documents", [], |r| {
                    r.get::<_, i64>(0)
                })?)
            })
            .unwrap();
        assert_eq!(after_fail, before_docs, "datos vivos intactos tras ZIP inválido");

        // 9) Eliminar backup
        crate::commands::backup::delete_backup_for_test(&state, &backup).expect("delete backup");
        assert!(!PathBuf::from(&backup).exists());

        // 10) Paths
        let paths = state.paths_snapshot().expect("paths");
        assert!(paths.db_path.exists());
        assert!(paths.documents_dir.exists());
        assert!(paths.backups_dir.exists());

        drop(state);
        let _ = fs::remove_dir_all(data_dir);
    }

    #[test]
    fn restore_rejects_zip_without_db() {
        use std::io::Write;
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        let data_dir = temp_data_dir();
        let state = AppState::init(data_dir.clone()).expect("init");
        let _ = create_entity(&state, "A", "personal");
        let before: i64 = state
            .with_db(|conn| Ok(conn.query_row("SELECT COUNT(*) FROM entities", [], |r| r.get(0))?))
            .unwrap();

        let bad = data_dir.join("backups").join("folio_backup_nodb.zip");
        fs::create_dir_all(bad.parent().unwrap()).unwrap();
        let file = fs::File::create(&bad).unwrap();
        let mut zip = ZipWriter::new(file);
        zip.start_file("readme.txt", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"sin folio.db").unwrap();
        zip.finish().unwrap();

        let err = restore_backup_for_test(&state, &bad.to_string_lossy()).expect_err("sin db");
        let _ = err;
        let after: i64 = state
            .with_db(|conn| Ok(conn.query_row("SELECT COUNT(*) FROM entities", [], |r| r.get(0))?))
            .unwrap();
        assert_eq!(after, before);

        drop(state);
        let _ = fs::remove_dir_all(data_dir);
    }

    #[test]
    fn restore_rejects_path_traversal() {
        use std::io::Write;
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        let data_dir = temp_data_dir();
        let state = AppState::init(data_dir.clone()).expect("init");
        let _ = create_entity(&state, "A", "personal");

        let bad = data_dir.join("backups").join("folio_backup_traverse.zip");
        fs::create_dir_all(bad.parent().unwrap()).unwrap();
        let file = fs::File::create(&bad).unwrap();
        let mut zip = ZipWriter::new(file);
        zip.start_file("../evil.txt", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"x").unwrap();
        zip.finish().unwrap();

        let err = restore_backup_for_test(&state, &bad.to_string_lossy())
            .expect_err("traversal debe fallar");
        assert!(
            format!("{err}").to_lowercase().contains("traversal")
                || format!("{err}").to_lowercase().contains("path"),
            "mensaje={err}"
        );

        drop(state);
        let _ = fs::remove_dir_all(data_dir);
    }

    #[test]
    fn isolation_blocks_cross_entity_provider_delete() {
        let data_dir = temp_data_dir();
        let state = AppState::init(data_dir.clone()).expect("init");
        let a = create_entity(&state, "Ent A", "personal");
        let b = create_entity(&state, "Ent B", "personal");
        let provider_a = create_provider(
            &state,
            &a,
            NewProvider {
                entity_id: a.clone(),
                name: "Solo A".into(),
                cuit: None,
                alias: None,
                cbu: None,
                email: None,
                phone: None,
                notes: None,
            },
        );

        // delete con entity B no debe borrar proveedor de A
        let err = state
            .with_db(|conn| {
                crate::db::ensure_provider_in_entity(conn, &b, &provider_a)?;
                Ok(())
            })
            .expect_err("cross-entity");
        let _ = err;

        let still: i64 = state
            .with_db(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM providers WHERE id = ?1 AND entity_id = ?2",
                    params![provider_a, a],
                    |r| r.get(0),
                )?)
            })
            .unwrap();
        assert_eq!(still, 1);

        drop(state);
        let _ = fs::remove_dir_all(data_dir);
    }

    #[test]
    fn import_rejects_oversized_file() {
        let data_dir = temp_data_dir();
        let state = AppState::init(data_dir.clone()).expect("init");
        let entity_id = create_entity(&state, "Big", "personal");

        let big = data_dir.join("enorme.bin");
        // 50 MiB + 1 byte (tope de import)
        let oversized = (50 * 1024 * 1024) + 1;
        let file = fs::File::create(&big).unwrap();
        file.set_len(oversized).unwrap();

        let report = import_files_sync(
            state.clone(),
            entity_id,
            vec![big.to_string_lossy().into_owned()],
        )
        .expect("import sync no aborta el lote");
        assert_eq!(report.imported, 0);
        assert!(!report.errors.is_empty());
        assert!(
            report.errors[0].to_lowercase().contains("grande")
                || report.errors[0].to_lowercase().contains("máx"),
            "errors={:?}",
            report.errors
        );

        drop(state);
        let _ = fs::remove_dir_all(data_dir);
    }
}
