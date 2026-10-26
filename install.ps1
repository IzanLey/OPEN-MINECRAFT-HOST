# Instalador de open-mc-host para Windows (PowerShell).
# Uso: irm https://raw.githubusercontent.com/IzanLey/OPEN-MINECRAFT-HOST/main/install.ps1 | iex
# Version concreta: $env:OMH_VERSION = 'v0.1.0'; irm ... | iex
$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$Repo = if ($env:OMH_REPO) { $env:OMH_REPO } else { 'IzanLey/OPEN-MINECRAFT-HOST' }
if ($env:OMH_VERSION) {
    $BaseUrl = "https://github.com/$Repo/releases/download/$($env:OMH_VERSION)"
} else {
    $BaseUrl = "https://github.com/$Repo/releases/latest/download"
}
$Asset = 'omh-windows-x86_64.exe'
$AssetUrl = "$BaseUrl/$Asset"
$SumsUrl = "$BaseUrl/SHA256SUMS"

$InstallDir = Join-Path $env:LOCALAPPDATA 'open-mc-host'
$Exe = Join-Path $InstallDir 'omh.exe'
$NewExe = Join-Path $InstallDir 'omh.exe.new'
$OldExe = Join-Path $InstallDir 'omh.exe.old'
$SumsFile = Join-Path $env:TEMP ('omh-SHA256SUMS-' + [Guid]::NewGuid().ToString('N'))

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

Write-Host "Descargando open-mc-host desde $AssetUrl"
try {
    # Descarga a un temporal: el omh.exe bueno no se toca hasta comprobar el hash.
    Invoke-WebRequest -UseBasicParsing -Uri $AssetUrl -OutFile $NewExe | Out-Null
    Invoke-WebRequest -UseBasicParsing -Uri $SumsUrl -OutFile $SumsFile | Out-Null
} catch {
    Remove-Item -Force -ErrorAction SilentlyContinue -Path $NewExe, $SumsFile
    Write-Host "No se pudo descargar open-mc-host: $($_.Exception.Message)"
    Write-Host 'Comprueba tu conexion y vuelve a ejecutar este instalador.'
    return
}

$Expected = $null
foreach ($line in (Get-Content -Path $SumsFile)) {
    $parts = $line -split '\s+'
    if ($parts.Length -ge 2) {
        $name = $parts[1].TrimStart('*')
        if ($name -eq $Asset) { $Expected = $parts[0]; break }
    }
}

if (-not $Expected) {
    Remove-Item -Force -ErrorAction SilentlyContinue -Path $NewExe, $SumsFile
    Write-Host "SHA256SUMS no contiene $Asset. Instalacion cancelada."
    return
}

# -ne no distingue mayusculas al comparar cadenas.
$Actual = (Get-FileHash -Algorithm SHA256 -Path $NewExe).Hash
if ($Actual -ne $Expected) {
    Remove-Item -Force -ErrorAction SilentlyContinue -Path $NewExe, $SumsFile
    Write-Host 'El hash de la descarga no coincide.'
    Write-Host "  esperado: $Expected"
    Write-Host "  obtenido: $Actual"
    Write-Host 'Instalacion cancelada.'
    return
}
Remove-Item -Force -ErrorAction SilentlyContinue -Path $SumsFile
Write-Host 'Hash SHA256 verificado.'

if (Test-Path $Exe) {
    if (Test-Path $OldExe) {
        Remove-Item -Force -ErrorAction SilentlyContinue -Path $OldExe
    }
    try {
        # Si hay un servidor en marcha, el exe esta en uso: se aparta antes de sustituirlo.
        Rename-Item -Path $Exe -NewName 'omh.exe.old'
    } catch {
        Write-Host 'No se pudo sustituir omh.exe: parece que hay un servidor en marcha.'
        Write-Host 'Para el servidor (Ctrl+C en su terminal) y vuelve a ejecutar este instalador.'
        Write-Host "La descarga queda en $NewExe."
        return
    }
}
try {
    Move-Item -Force -Path $NewExe -Destination $Exe -ErrorAction Stop
} catch {
    if ((Test-Path $OldExe) -and -not (Test-Path $Exe)) {
        Move-Item -Force -Path $OldExe -Destination $Exe -ErrorAction SilentlyContinue
    }
    Write-Host 'No se pudo instalar omh.exe (antivirus o archivo bloqueado). Se mantiene la version anterior.'
    Write-Host "La descarga queda en $NewExe."
    return
}

# Restos de la instalacion antigua de Python en esta misma carpeta.
Remove-Item -Force -ErrorAction SilentlyContinue -Path (Join-Path $InstallDir 'omh.cmd'), (Join-Path $InstallDir 'omh.py')

$UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (-not $UserPath) { $UserPath = '' }
if (($UserPath -split ';') -notcontains $InstallDir) {
    [Environment]::SetEnvironmentVariable('Path', ($UserPath.TrimEnd(';') + ';' + $InstallDir).TrimStart(';'), 'User')
    Write-Host "Se anadio $InstallDir a tu PATH de usuario. Abre una terminal nueva para usar 'omh'."
}
$env:Path = "$env:Path;$InstallDir"

Write-Host "open-mc-host instalado en $InstallDir"
& (Join-Path $InstallDir 'omh.exe')
