# Instalador de open-mc-host para Windows (PowerShell).
# Uso: irm https://raw.githubusercontent.com/IzanLey/OPEN-MINECRAFT-HOST/main/install.ps1 | iex
$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$Repo = if ($env:OMH_REPO) { $env:OMH_REPO } else { 'IzanLey/OPEN-MINECRAFT-HOST' }
$Branch = if ($env:OMH_BRANCH) { $env:OMH_BRANCH } else { 'main' }
$InstallDir = Join-Path $env:LOCALAPPDATA 'open-mc-host'
$Url = "https://raw.githubusercontent.com/$Repo/$Branch/omh.py"

# Busca un Python 3.8+ real (no el acceso directo de Microsoft Store).
$Python = $null
# Sin comillas dentro: PowerShell 5.1 las pasa mal a los programas externos.
$Check = 'import sys; sys.version_info >= (3, 8) and print(sys.executable)'
foreach ($candidate in 'py -3', 'python', 'python3') {
    $parts = $candidate.Split(' ')
    $exe = $parts[0]
    $extra = @($parts | Select-Object -Skip 1)
    if (-not (Get-Command $exe -ErrorAction SilentlyContinue)) { continue }
    try {
        $path = (& $exe @extra -c $Check | Out-String).Trim()
        if ($LASTEXITCODE -eq 0 -and $path) { $Python = $path; break }
    } catch { }
}

if (-not $Python) {
    Write-Host 'Se necesita Python 3.8 o superior.'
    Write-Host 'Instalalo con:  winget install Python.Python.3.12'
    Write-Host 'o desde https://www.python.org/downloads/ y vuelve a ejecutar este instalador.'
    return
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Write-Host "Descargando open-mc-host desde $Url"
Invoke-WebRequest -UseBasicParsing -Uri $Url -OutFile (Join-Path $InstallDir 'omh.py')

$Launcher = "@echo off`r`n`"$Python`" `"%~dp0omh.py`" %*`r`n"
Set-Content -Path (Join-Path $InstallDir 'omh.cmd') -Value $Launcher -Encoding ASCII -NoNewline

$UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (-not $UserPath) { $UserPath = '' }
if (($UserPath -split ';') -notcontains $InstallDir) {
    [Environment]::SetEnvironmentVariable('Path', ($UserPath.TrimEnd(';') + ';' + $InstallDir).TrimStart(';'), 'User')
    Write-Host "Se anadio $InstallDir a tu PATH de usuario. Abre una terminal nueva para usar 'omh'."
}
$env:Path = "$env:Path;$InstallDir"

Write-Host "open-mc-host instalado en $InstallDir"
& $Python (Join-Path $InstallDir 'omh.py')
