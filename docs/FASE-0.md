# Fase 0 — Identidad visual y fundación UI

## Alcance
Shell visual de Folio sin lógica de negocio.

## Incluye
- Logos oficiales SVG
- Tokens de marca
- Tailwind + kit UI (estilo shadcn)
- Sidebar, header, selector de entidad (demo)
- Dashboard con datos ficticios
- Temas claro / oscuro / system
- Favicon + generación de iconos Windows

## No incluye
DB, importación, QR, PDF, pagos reales, backup, red, licencias.

## Cómo ejecutar
```bash
npm install
npm run dev          # solo frontend Vite
npm run tauri:dev    # app Tauri (requiere MSVC)
```

Helper Windows:
```powershell
powershell -ExecutionPolicy Bypass -File scripts\dev-windows.ps1
```

Iconos:
```bash
npm run icons:generate
npm run icons:tauri
```
