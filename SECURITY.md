# Seguridad

## Cómo reportar
Si encontrás un problema de seguridad en Folio:

1. Escribinos a `folioapp.dev@gmail.com` con el asunto `Security: Folio`.
2. Indicá versión de Folio, versión de Windows y pasos para reproducir.
3. No publiques exploits ni datos personales de terceros.
4. No adjuntes facturas, bases SQLite ni backups reales.

Si preferís GitHub, abrí un issue **sin detalle explotable** y pedí contacto privado.

## Principios (lo que buscamos demostrar)
- Folio opera **localmente** para el uso habitual.
- No hay cuentas ni telemetría obligatoria.
- No hay publicidad dentro de la aplicación.
- Los instaladores públicos publican **SHA-256**.
- El acceso a archivos desde el webview está acotado.

Estas afirmaciones se validan con el checklist de abajo. Mientras un ítem esté pendiente, no lo presentamos como garantía absoluta.

## Alcance actual (0.1.3+)
- Base SQLite local con foreign keys
- Hash SHA-256 de archivos en importación
- Mutaciones de proveedores/documentos/pagos validadas por `entity_id`
- Export CSV con sanitización anti-fórmulas
- Logs locales rotados; sin CUIT/importes/CAE ni rutas absolutas
- Backup ZIP local con restauración validada + rollback + límites anti zip-bomb
- Importación con tope de tamaño por archivo (50 MiB)
- Instalador NSIS en modo usuario actual
- **Sin firma de código** (SmartScreen puede avisar)
- Capabilities mínimas: `core`, `dialog`, `opener` (sin `fs` ni `shell`)
- CSP habilitada (compatible con PDF.js / workers)
- Protocolo `asset:` limitado a `$APPDATA/**` y `$APPLOCALDATA/**` (ya resuelven a la carpeta de Folio)

### Permisos Tauri (revisión)
| Plugin / permiso | Estado | Motivo |
|---|---|---|
| `dialog` | Sí | Elegir archivos/carpetas para importar, exportar y backups |
| `opener` | Sí | Abrir enlaces externos (GitHub, mail, Mercado Pago) |
| `fs` | **Removido** | El frontend no lo usaba; I/O de archivos va por comandos Rust |
| `shell` | **Removido** | No se ejecutan comandos del sistema desde el frontend |
| `asset:` | Acotado | Solo carpetas de datos de Folio para previsualizar documentos |

> Si configurás una carpeta de documentos fuera del directorio de datos de Folio, la previsualización vía `asset:` puede no funcionar. El almacenamiento sigue operando por backend.

### Restauración de backups
Orden seguro:
1. Extraer ZIP a staging (path traversal bloqueado + límites de entradas/tamaño/ratio)
2. Validar estructura + `PRAGMA integrity_check` + tablas requeridas
3. Crear snapshot de seguridad en `_restore_safety` (fuera de `_restore_work`)
4. Reemplazar datos solo después de validar
5. Reabrir SQLite; si falla, rollback al snapshot

## Aviso de Windows / SmartScreen
En 0.1.3 el instalador **no está firmado** con certificado de código. Windows suele mostrar:

> “Windows protegió su PC” / “SmartScreen…”

Eso **no significa** que Folio esté infectado: significa que el editor aún no tiene reputación ni certificado de firma (tiene costo y se evalúa para 1.0).

### Cómo instalar con cuidado
1. Descargá solo desde [GitHub Releases](https://github.com/redmanxp/folio/releases).
2. Verificá el SHA-256 publicado en las notas del release.
3. En el aviso: **Más información** → **Ejecutar de todas formas**.
4. Si el hash no coincide, no instales y avisanos.

### Verificar hash en PowerShell
```powershell
Get-FileHash .\Folio_0.1.3_x64-setup.exe -Algorithm SHA256
```

### VirusTotal (0.1.3)
- Informe: https://www.virustotal.com/gui/file/2fcd498513b7b7851ef2a5ef3476cf09c14cded063146475307a72f3f2547c45
- Resultado: **3/70** (Cylance, APEX, Sophos). Es típico ML/PUA en instaladores NSIS **sin firma de código**; no implica malware confirmado.

### Firma de código (cómo llegar a instalador firmado)
Para que SmartScreen confíe más (y bajar falsos positivos):

1. **Certificado Authenticode** a nombre de una persona/empresa verificada:
   - **OV Code Signing** (Organization Validation): más barato; desde ~2023 suele ir en **token/HSM** (no solo archivo `.pfx` suelto).
   - **EV Code Signing** (Extended Validation): más caro; reputación SmartScreen más rápida.
2. Comprar en una CA reconocida (DigiCert, Sectigo, SSL.com, GlobalSign, etc.). En Argentina suele hacer falta CUIT/razón social o datos de persona según el producto.
3. Firmar el `.exe`/instalador con `signtool` (Windows SDK) o el pipeline de Tauri (`bundle > windows > certificateThumbprint` / variables de entorno de firma).
4. Timestamp (`/tr` RFC3161) para que la firma siga válida al vencer el cert.
5. Publicar el instalador ya firmado + SHA-256 + VirusTotal.

Hasta tener certificado, Folio documenta SmartScreen + hash + VT como mitigación.

## Checklist antes del release 1.0
- [x] Revisar capabilities y permisos de Tauri (mínimo: dialog + opener)
- [x] Confirmar que el frontend no usa plugins fs/shell para I/O arbitrario
- [x] Validar restauración: no borra datos vivos antes de validar; rollback si falla
- [x] Verificar que los ZIP de backup no permitan path traversal al restaurarse
- [ ] Revisar consultas SQL / parámetros (auditoría completa)
- [ ] Auditar dependencias npm y crates (`npm audit`, `cargo audit`)
- [ ] Confirmar que ningún documento o metadato se envía a servidores propios
- [ ] Comprobar que los archivos originales del usuario no se modifican
- [x] Probar restauraciones de backup reales
- [ ] Probar desinstalación sin pérdida de archivos originales
- [ ] Probar en Windows limpio 10/11
- [x] Publicar hash del instalador + VirusTotal
- [x] Documentar ausencia de firma de código (SmartScreen esperado)
- [x] Configurar CSP razonable compatible con PDF.js
- [x] ~~Certificado de firma de código (OV/EV)~~ — **no aplica** por ahora (producto gratis; hash + VT + SmartScreen documentado)

## Instaladores
Cada release debe indicar el nombre del instalador, su SHA-256, link VirusTotal y si está firmado o no.
