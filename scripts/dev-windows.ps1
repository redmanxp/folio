# Activa MSVC Build Tools + Windows SDK y lanza Folio en modo desarrollo.
# Uso: powershell -ExecutionPolicy Bypass -File scripts\dev-windows.ps1

$ErrorActionPreference = "Stop"

$msvcRoot = "C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Tools\MSVC\14.29.30133"
$sdk = "C:\Program Files (x86)\Windows Kits\10"
$sdkVer = "10.0.22621.0"

if (-not (Test-Path "$msvcRoot\bin\Hostx64\x64\link.exe")) {
  Write-Error "No se encontró link.exe de MSVC. Instalá 'Build Tools for Visual Studio' con workload C++."
}

$env:Path = "$msvcRoot\bin\Hostx64\x64;$sdk\bin\x64;$sdk\bin\$sdkVer\x64;$env:Path"
$env:LIB = "$msvcRoot\lib\x64;$sdk\Lib\$sdkVer\um\x64;$sdk\Lib\$sdkVer\ucrt\x64"
$env:INCLUDE = "$msvcRoot\include;$sdk\Include\$sdkVer\ucrt;$sdk\Include\$sdkVer\um;$sdk\Include\$sdkVer\shared"

Set-Location (Split-Path $PSScriptRoot -Parent)
Write-Host "Entorno MSVC listo. Iniciando Folio..."
npm run tauri:dev
