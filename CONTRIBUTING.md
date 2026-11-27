# Contribuir a Folio

Gracias por interesarte en mejorar Folio.

## Antes de empezar
- Leé el [README](README.md) y el [documento maestro](docs/DOCUMENTO-MAESTRO.md).
- Folio es local-first: no propongas funciones que obliguen a subir documentos a Internet.
- Preferimos cambios chicos y revisables.

## Cómo reportar
- Errores: usá la plantilla de bug en Issues.
- Ideas: usá la plantilla de sugerencia.
- Seguridad: seguí [SECURITY.md](SECURITY.md). No adjuntes facturas ni bases de datos.

## Desarrollo local
```bash
npm install
npm run tauri:dev
```

En Windows, si falta el linker MSVC:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\dev-windows.ps1
```

Antes de abrir un PR:
```bash
npm run build
cd src-tauri && cargo test
```

## Estilo de contribución
1. Describí el problema o la mejora.
2. Explicá el impacto (usuarios, privacidad, estabilidad).
3. Incluí pasos de prueba.
4. Evitá refactors grandes no pedidos.

## Capturas de la web
```bash
npm run screenshots
```

Las capturas de marketing usan datos inventados (`scripts/demo-ui`).
