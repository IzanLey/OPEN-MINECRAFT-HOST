---
tipo: migracion
tags: [omh/migracion]
aliases: [instaladores nuevos, self-update]
---
# Distribución del binario

## Assets de cada release
`omh-linux-x86_64`, `omh-windows-x86_64.exe` y `SHA256SUMS` → [[CI y releases]]. Enlace fijo a la última: `https://github.com/IzanLey/OPEN-MINECRAFT-HOST/releases/latest/download/<asset>`.

## `install.sh` (Linux)
1. Comprueba que la máquina es x86_64 (si no: «solo hay binarios para Linux x86_64»).
2. Descarga el binario y `SHA256SUMS` con curl o wget; si hay `sha256sum`, comprueba el hash.
3. Lo instala en `~/.local/bin/omh` con permiso de ejecución (sustituye el lanzador antiguo de Python).
4. Avisa si `~/.local/bin` no está en el PATH y ejecuta `omh` (título y guía).
5. Variables: `OMH_REPO` como antes; `OMH_VERSION` (un tag) en lugar de `OMH_BRANCH`.

## `install.ps1` (Windows)
1. Descarga `omh.exe` y `SHA256SUMS` en `%LOCALAPPDATA%\open-mc-host\` y comprueba el hash con `Get-FileHash`.
2. Si `omh.exe` está en uso (un servidor en marcha), lo renombra antes a `omh.exe.old`.
3. Borra `omh.cmd` y `omh.py` de la instalación anterior de Python en esa carpeta.
4. Añade la carpeta al PATH del usuario (como antes) y ejecuta `omh`.

## `omh update` (Rust)
1. `GET /repos/IzanLey/OPEN-MINECRAFT-HOST/releases/latest` → asset de la plataforma y `SHA256SUMS` → [[API GitHub]].
2. `Descargando la ultima version desde <url>` y la barra de progreso de siempre → [[Descargas verificadas]].
3. Comprueba el SHA-256.
4. Sustituye el ejecutable: en Linux, archivo temporal en la misma carpeta + `rename`; en Windows, renombra el `.exe` en uso a `.old` y deja el nuevo en su lugar. El `.old` se borra en la siguiente ejecución.
5. `[OK] open-mc-host actualizado.`

## Quien tenga la versión Python
Su `omh update` descarga `main/omh.py`, que dejará de existir: tendrá que volver a ejecutar el instalador → [[Bugs y ambiguedades#B9]], [[Instalacion]].
