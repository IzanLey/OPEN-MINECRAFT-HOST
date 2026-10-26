---
tipo: concepto
tags: [omh/concepto]
aliases: [install.sh, install.ps1, instaladores, requisitos]
fuente: "install.sh, install.ps1, README"
---
# Instalación

## Requisitos (0.0.1)
- Windows 10/11 o Linux de 64 bits.
- Python 3.8 o superior (`winget install Python.Python.3.12`, `sudo apt install python3`…).
- Java **no**: lo descarga omh → [[Java Temurin]].

## Comandos
- Linux: `curl -fsSL https://raw.githubusercontent.com/IzanLey/OPEN-MINECRAFT-HOST/main/install.sh | bash`
- Windows (PowerShell): `irm https://raw.githubusercontent.com/IzanLey/OPEN-MINECRAFT-HOST/main/install.ps1 | iex`

## Qué hacen hoy
**install.sh**: comprueba `python3` ≥ 3.8; descarga `omh.py` de `raw…/<OMH_REPO>/<OMH_BRANCH>/omh.py` (por defecto `IzanLey/OPEN-MINECRAFT-HOST` y `main`) con curl o wget en `~/.local/share/open-mc-host/`; crea el lanzador `~/.local/bin/omh` (`exec python3 …/omh.py "$@"`); avisa si `~/.local/bin` no está en el PATH; ejecuta `omh` (título y guía).

**install.ps1**: busca un Python 3.8+ real (`py -3`, `python`, `python3`, evitando el acceso directo de la Microsoft Store); descarga `omh.py` en `%LOCALAPPDATA%\open-mc-host\`; crea `omh.cmd` (que llama al Python encontrado con `%~dp0omh.py`); añade la carpeta al PATH del usuario; ejecuta `omh`. Fuerza TLS 1.2 para PowerShell 5.1.

También se puede usar sin instalar: `git clone …` y `python omh.py`.

## En la 0.1.0
Los instaladores descargan el **binario** de la última release y ya no hace falta Python → [[Distribucion del binario]].
