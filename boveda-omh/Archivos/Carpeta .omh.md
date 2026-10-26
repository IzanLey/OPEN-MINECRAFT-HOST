---
tipo: archivo
tags: [omh/archivo]
aliases: [.omh, RUN_DIR, console.log, wrapper.log, installer.log]
fuente: "omh.py · RUN_DIR (L50)"
rust: "src/runner/mod.rs"
---
# Carpeta .omh

`<servidor>/.omh/`: datos de ejecución del servidor. La crea [[omh create]].

| Archivo | Quién lo escribe | Contenido |
|---|---|---|
| `console.log` | [[Supervisor (Runner)]] | Todo lo que imprime el servidor y los mensajes `[omh] …`, en UTF-8 y añadiendo al final. Al arrancar, si pasa de 5 MB se renombra a `console.old.log`. |
| `console.old.log` | Supervisor | El registro anterior |
| `wrapper.log` | [[omh start]] `--detach` | Salida del supervisor en segundo plano (añadiendo) |
| `installer.log` | Forge/NeoForge | Salida del instalador → [[Core forge]], [[Core neoforge]] |
| `playit.log` | Supervisor | Salida del agente (se reescribe en cada arranque) → [[Tunel playit.gg]] |
| `run.json` | Supervisor | [[run.json]] |
| `forge-installer.jar`, `neoforge-installer.jar`, `bedrock-server.zip`, archivo de PHP, zip de PowerNukkitX | Instaladores | Descargas temporales que se borran al terminar |

- Lo leen [[omh logs]] y [[omh console]] (`console.log`) y [[Direcciones del servidor]] (`playit.log`, para detectar el enlace `https://playit.gg/claim/…`).
- Está en el `.gitignore` del repositorio.
