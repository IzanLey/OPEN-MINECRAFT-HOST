---
tipo: core
tags: [omh/core]
aliases: [paper, PaperMC]
fuente: "omh.py · install_paper (L833), resolve_version (L765)"
rust: "src/cores/paper.rs"
---
# Core paper

`Alto rendimiento. Plugins Bukkit/Spigot/Paper.`

| | |
|---|---|
| Edición | java |
| Versión mínima | 1.8.8 |
| `latest` | la última versión de la [[API PaperMC]] |
| Java | según Mojang → [[Java Temurin]] |
| Complementos | plugins en `plugins/`; Modrinth con `paper, spigot, bukkit, purpur` |

## Instalación
1. Versión 1.8.x distinta de 1.8.8: `[AVISO] Paper no tiene 1.8.9. Se usa 1.8.8, compatible con todos los clientes 1.8.x.` y usa 1.8.8.
2. Pide la URL y el SHA-256 a Fill v3 y, si falla, a la API v2 → [[API PaperMC]].
3. Descarga `server.jar` comprobando el **SHA-256** (etiqueta `paper <v>`).
4. `start = {"type": "jar", "jar": "server.jar"}`.

Para que entren jugadores de Bedrock se puede instalar Geyser: `omh plugin add geyser` (usa el puerto 19132/UDP) → [[Plugins y mods]].
