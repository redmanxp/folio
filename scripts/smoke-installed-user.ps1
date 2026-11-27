# Smoke de usuario sobre Folio instalado (sin Computer Use).
$ErrorActionPreference = "Stop"

$exe = Join-Path $env:LOCALAPPDATA "Folio\folio.exe"
$results = New-Object System.Collections.Generic.List[string]

function Ok($msg) { $script:results.Add("PASS  $msg"); Write-Host "PASS  $msg" -ForegroundColor Green }
function Fail($msg) { $script:results.Add("FAIL  $msg"); Write-Host "FAIL  $msg" -ForegroundColor Red }

Write-Host "=== Folio smoke usuario instalado ==="

# 1) Instalador / binario
if (Test-Path $exe) { Ok "folio.exe instalado en $exe" } else { Fail "No esta folio.exe instalado"; exit 1 }

# 2) Hash del setup descargado (si existe)
$setup = Join-Path $env:USERPROFILE "Downloads\Folio_0.1.0_x64-setup.exe"
if (Test-Path $setup) {
  $hash = (Get-FileHash $setup -Algorithm SHA256).Hash
  if ($hash -eq "BB546B0C3F4AC0E2833CD221A79B1E037EB77BDC30AFF337ED100E9FE74550F0") {
    Ok "SHA-256 del instalador coincide"
  } else {
    Fail "SHA-256 no coincide: $hash"
  }
} else {
  $results.Add("SKIP  setup no esta en Downloads")
}

# 3) Arranque
Get-Process Folio -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 400
Start-Process $exe
Start-Sleep -Seconds 3
$proc = Get-Process Folio -ErrorAction SilentlyContinue
if ($proc -and $proc.MainWindowTitle -match "Folio") {
  Ok "App abre ventana 'Folio' (PID $($proc.Id))"
} elseif ($proc) {
  Ok "Proceso Folio corriendo (PID $($proc.Id)) sin titulo aun"
} else {
  Fail "Folio no arranco"
}

# 4) Datos locales
$data = Join-Path $env:APPDATA "com.folio.app"
if (Test-Path (Join-Path $data "folio.db")) { Ok "Existe folio.db en AppData" } else { Fail "Falta folio.db" }
if (Test-Path (Join-Path $data "documents")) { Ok "Existe carpeta documents" } else { Fail "Falta documents" }
if (Test-Path (Join-Path $data "backups")) { Ok "Existe carpeta backups" } else { Fail "Falta backups" }

# 5) Log reciente
$log = Join-Path $data "folio.log"
if (Test-Path $log) {
  $tail = Get-Content $log -Tail 30 -ErrorAction SilentlyContinue
  if ($tail -match "Folio listo|Inicializando Folio") {
    Ok "Log muestra inicializacion OK"
  } else {
    $results.Add("WARN  log sin linea de listo reciente")
    Write-Host "WARN  log sin linea de listo reciente"
  }
} else {
  Fail "No hay folio.log"
}

# 6) Uninstall presente
$un = Join-Path $env:LOCALAPPDATA "Folio\uninstall.exe"
if (Test-Path $un) { Ok "uninstall.exe presente" } else { Fail "Falta uninstall.exe" }

Write-Host ""
Write-Host "=== Resumen ==="
$results | ForEach-Object { Write-Host $_ }
$failed = @($results | Where-Object { $_ -like "FAIL*" }).Count
if ($failed -gt 0) { exit 1 } else { exit 0 }
