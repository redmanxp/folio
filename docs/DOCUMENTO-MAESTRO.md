# FOLIO — Documento maestro del proyecto

**Gestión simple, inteligente y local de facturas y documentos de compra para Windows.**

> Folio — Tus documentos. Tus empresas. Todo en un solo lugar.  
> Software gratuito · Windows · Offline · Privacidad local  
> Versión conceptual 1.0 — 25 de noviembre de 2026

Documento integral: producto, funcionalidades, tecnología, diseño, desarrollo con Cursor, pruebas, seguridad, distribución y crecimiento.

## 1. Visión

Aplicación gratuita para Windows que organiza, administra, busca y controla facturas y documentos de compras, gastos, proveedores y pagos.

**No es un sistema contable.** Es una herramienta:
- Simple, visual, rápida, privada, gratuita, offline, multiempresa.

## 2. Qué es Folio

Administrador local de documentación de compras. Centraliza y simplifica el seguimiento cotidiano. No reemplaza ERP, contabilidad ni facturador.

**Público:** personas, profesionales, emprendedores, comercios chicos, multiempresa, quienes guardan PDF y evitan nube/suscripciones.

## 3–12. Funcionalidades (resumen de alcance)

- Dashboard con métricas, vencimientos, últimos docs, filtros entidad/período
- Multiempresa (entidades con color/logo opcional)
- Documentos: facturas, NC/ND, recibos, tickets, etc.; carga drag&drop; extracción asistida (revisión manual)
- QR ARCA/AFIP + OCR local; visor PDF
- Proveedores con CUIT (sin duplicar por CUIT)
- Pagos parciales, estados, vencimientos
- Categorías y etiquetas
- Búsqueda/filtros combinables
- Export Excel/CSV e informes
- Backup/restauración ZIP transparente
- Sin telemetría, sin cuenta, sin publicidad

## 13. Stack

Tauri 2 · Rust · React · TypeScript · SQLite · Tailwind · shadcn/ui · Lucide · (TanStack Table, PDF.js, ZXing/rqrr, XLSX según etapa)

## 14–16. Arquitectura y etapas

IPC Tauri controlado; archivos separados de SQLite; desarrollo por etapas con checkpoint. No avanzar con errores críticos.

## 17–20. QA, distribución y crecimiento

Pruebas con PDFs reales; GitHub Releases; SHA-256; VirusTotal; difusión orgánica; Folio gratuito; apoyo voluntario opcional.

## 21–25. Costos, roadmap, marca, reglas Cursor

Paleta oficial #2563EB / #4F8DF7 / #20B486 / #172B4D / #CBD5E1.  
Assets en `src/assets/brand/`.  
Usuario no programa a mano; Cursor implementa por etapa autorizada.

## Extracción QR ARCA (referencia técnica)

URL: `https://www.arca.gob.ar/fe/qr/?p={base64}` (también afip.gob.ar).

JSON v1: `ver`, `fecha`, `cuit` (emisor), `ptoVta`, `tipoCmp`, `nroCmp`, `importe`, `moneda`, `ctz`, `tipoDocRec`, `nroDocRec`, `tipoCodAut` (E=CAE / A=CAEA), `codAut`.

El QR **no** trae razón social, neto/IVA ni vencimiento. Leer offline; constatar CAE en web es opcional y futuro.
