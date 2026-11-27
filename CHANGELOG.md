# Changelog

## 0.1.2 — 2026-11-26

### Corregido
- Visor PDF con PDF.js (canvas) en lugar de iframe en blanco
- Al cerrar un documento se vuelve a la sección de origen (Facturas ↔ Documentos)
- Scope `asset:`: `$APPDATA` ya incluye `com.folio.app` (el path duplicado daba 403 al cargar PDFs)
- Importes alineados a la derecha en listados

### Añadido
- Instalador NSIS `Folio_0.1.2_x64-setup.exe`

## 0.1.1 — 2026-11-26

### Seguridad
- Aislamiento estricto por `entity_id` en update/delete de proveedores, documentos y pagos
- Sanitización anti-fórmulas en exportación CSV
- Logs sin CUIT/importes/CAE ni rutas absolutas; rotación de `folio.log` (5 MiB)
- Restore: límites anti zip-bomb + snapshot de seguridad fuera de `_restore_work`
- Importación: tope de 50 MiB por archivo
- Tests negativos de restore, aislamiento e import oversized

### Añadido
- Instalador NSIS `Folio_0.1.1_x64-setup.exe`

## 0.1.0 — 2026-11-26

### Añadido
- Scaffold Tauri 2 + React + TypeScript + Rust + SQLite
- Entidades (personal / empresa) con aislamiento
- Importación de PDF/imágenes con hash SHA-256 y detección de duplicados
- Extracción asistida (QR ARCA/AFIP, texto PDF, OCR local)
- Archivo local de documentos por entidad
- Buscador con filtros y exportación CSV
- Visualizador de PDF/imagen integrado
- Proveedores con ficha (CUIT, alias, CBU, correo, teléfono)
- Pagos y agenda de vencimientos
- Backup / restauración ZIP local
- Secciones institucionales: Acerca, Apoyar, Ayuda, Contacto, Privacidad
- Apoyo voluntario por Mercado Pago
- Sitio en GitHub Pages + capturas de demostración
- Instalador NSIS `Folio_0.1.0_x64-setup.exe`
- Documentación de arquitectura, roadmap, lanzamiento y seguridad
