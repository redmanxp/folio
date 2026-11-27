# Folio

**Tus documentos, bajo control.**

Aplicación de escritorio para Windows: gratuita, local, sin cuenta y de código abierto.

- Sitio: https://redmanxp.github.io/folio/
- Código: https://github.com/redmanxp/folio
- Contacto: folioapp.dev@gmail.com
- Apoyar: [Mercado Pago](https://link.mercadopago.com.ar/folioapp)

## Estado actual
**MVP funcional local** — UI + SQLite + importación/búsqueda/pagos/backup.

Iconos oficiales: `src/assets/brand/folio_icon.svg` (favicon, sidebar y `.exe`).

## Qué hace
- Entidades (personal / empresa)
- Importar PDF/imagen con extracción asistida (QR ARCA/AFIP, texto PDF, OCR local)
- Buscar, visualizar y editar metadatos
- Proveedores, pagos y exportación CSV
- Backup / restauración local

## Stack
Tauri 2 · React · TypeScript · Tailwind CSS · Lucide · Rust · SQLite

## Desarrollo

### Requisitos
- Node.js 20+
- Rust (stable, target `x86_64-pc-windows-msvc`)
- Windows 10/11
- **Build Tools for Visual Studio** con workload C++ (para `link.exe` y Windows SDK)

### Comandos
```bash
npm install
npm run build
npm run tauri:dev
```

En Windows, si `cargo` no encuentra el linker:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\dev-windows.ps1
```

Tests Rust:

```powershell
cd src-tauri
cargo test
```

Build instalador:
```bash
npm run tauri:build
```

## Datos
Folio guarda la base y una copia de los documentos en el directorio de datos de la aplicación (`%APPDATA%/com.folio.app`). La desinstalación no debería usarse como forma de borrar tus archivos originales: Folio trabaja sobre copias archivadas.

## Documentación
- [Arquitectura](docs/ARQUITECTURA.md)
- [Roadmap](docs/ROADMAP.md)
- [Lanzamiento](docs/LANZAMIENTO.md)
- [Seguridad](SECURITY.md)

## Licencia
MIT
