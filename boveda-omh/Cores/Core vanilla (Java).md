---
tipo: core
tags: [omh/core]
aliases: [vanilla java, servidor oficial java]
fuente: "omh.py · install_vanilla_java (L827)"
rust: "src/cores/vanilla.rs"
---
# Core vanilla (Java)

`Servidor oficial de Mojang. Sin plugins ni mods.`

| | |
|---|---|
| Edición | java |
| Versión mínima | 1.7.10 |
| `latest` | `latest.release` de la [[API Mojang]] |
| Java | `javaVersion.majorVersion` de Mojang (mínimo 8) → [[Java Temurin]] |
| Complementos | ninguno → [[Plugins y mods]] |
| Puerto por defecto | 25565/TCP |

## Instalación
1. Busca la versión en el manifiesto y lee `downloads.server.{url, sha1}` → [[API Mojang]].
2. Descarga `server.jar` comprobando el **SHA-1** (etiqueta `vanilla <v>`) → [[Descargas verificadas]].
3. `start = {"type": "jar", "jar": "server.jar"}` → [[Comando de arranque]].

Archivos del servidor: `server.jar`, más los que genere el propio servidor (`world/`, `logs/`, `server.properties`, `whitelist.json`, `ops.json`, `usercache.json`…).
