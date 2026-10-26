---
tipo: mapa
tags: [omh/mapa]
aliases: [Archivos, Donde se guarda cada cosa]
---
# Mapa de archivos

## En la carpeta de cada servidor
| Ruta | Contenido | Nota |
|---|---|---|
| `omh.json` | Configuración de los 3 pasos y cómo arrancar | [[omh.json]] |
| `.omh/` | Consola, registros, `run.json`, descargas temporales | [[Carpeta .omh]] |
| `.omh/run.json` | Puerto y token del canal de control mientras está en marcha | [[run.json]] |
| `server.properties` | Puerto, IP, whitelist, online-mode, jugadores, motd | [[server.properties]] |
| `eula.txt` | Aceptación del EULA (solo Java) | [[EULA]] |
| `whitelist.json`, `ops.json` | Jugadores (Java) | [[Listas de jugadores]] |
| `allowlist.json`, `permissions.json` | Jugadores (Bedrock vanilla) | [[Listas de jugadores]] |
| `white-list.txt`, `ops.txt` | Jugadores (PocketMine, PowerNukkitX) | [[Listas de jugadores]] |
| `plugins/`, `mods/` | Complementos | [[Plugins y mods]] |
| `server.jar`, `fabric-server-launch.jar`, `powernukkitx.jar`, `libraries/…`, `bedrock_server[.exe]`, `PocketMine-MP.phar`, `bin/` | El servidor | [[Mapa de cores]] |

## En la carpeta del usuario: `~/.open-mc-host/`
| Ruta | Contenido | Nota |
|---|---|---|
| `state.json` | Lista de servidores y servidor activo | [[state.json]] |
| `java/<versión>/` | Java Temurin descargado | [[Java Temurin]] |
| `tools/playit-v0.17.1[.exe]` | Agente de playit.gg | [[Tunel playit.gg]] |
| `playit.toml` | Clave **secreta** del agente (la crea playit) | [[Carpeta global]] |

## Relación con git
El `.gitignore` del repositorio excluye `servers/`, `mc-*/`, `.omh/`, `omh.json`, `state.json`, `world*/`, `logs/`, `*.jar`, `*.phar`, `*.zip`, `*.tar.gz`, `*.part`, `.env`, `*.key`, `*.pem`, **`*.toml`** y **`playit*`**, para que no se suban servidores ni secretos por error.

> [!warning] Ojo en la migración
> `*.toml` ignora también `Cargo.toml` y `playit*` ignoraría un `src/network/playit.rs`. Ver [[Decisiones tecnicas#D13]].

← [[OPEN-MC-HOST]]
