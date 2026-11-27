# Seguridad

## Reportes
Si encontrás un problema de seguridad en Folio, abrí un issue privado o escribinos a `folioapp.dev@gmail.com` indicando versión, sistema operativo y pasos reproducibles. No publiques exploits.

## Principios
- Folio opera **localmente**. No debe enviar documentos ni metadatos a Internet.
- No hay cuentas ni telemetría.
- Los instaladores públicos deben publicar **SHA-256** y, idealmente, análisis en VirusTotal.
- El alcance del protocolo `asset:` se limita a carpetas de datos/documentos del usuario.

## Alcance actual (0.1.0)
- Base SQLite local con foreign keys
- Hash de archivos en importación
- Backup ZIP local
- Instalador NSIS en modo usuario actual (sin privilegios elevados innecesarios)

## Pendiente antes del release público 1.0
- Revisión de permisos Tauri/capabilities
- Prueba en Windows limpio 10/11
- Dependabot / audit de crates y npm
- Firma de código (si se consigue sin costo inicial, o documentar ausencia)
