---
tipo: api
tags: [omh/api]
aliases: [Modrinth]
fuente: "omh.py · MODRINTH_LOADERS (L639), modrinth_install (L2111), cmd_plugin search (L2154)"
rust: "src/plugins.rs"
---
# API Modrinth

## Versiones de un proyecto
`GET https://api.modrinth.com/v2/project/<id>/version?loaders=<json>&game_versions=<json>`
- `<id>` codificado con `quote` (no codifica `/`); `loaders` y `game_versions` son listas JSON con el formato de Python (`["paper", "spigot"]`, con espacio tras la coma) codificadas como formulario.
- 404: `No existe el proyecto '%s' en Modrinth. Prueba: omh plugin search %s`
- Lista vacía: `'%s' no tiene version para %s %s.`
- Usa `versions[0]` (la más reciente) y su archivo con `primary`, o el primero. Verifica **SHA-1** (`hashes.sha1`).
- Instala las `dependencies` con `dependency_type == "required"` y `project_id`, recursivamente y sin repetir.

## Búsqueda
`GET https://api.modrinth.com/v2/search?query=<texto>&facets=<json>&limit=15`
- facets: `[["project_type:plugin|mod"], ["categories:<cargador>", …], ["versions:<mc>"]]`
- Se muestran `slug`, `downloads` y `description`.

Cargadores por core: paper → `paper, spigot, bukkit, purpur`; purpur → `purpur, paper, spigot, bukkit`; fabric, forge y neoforge → el suyo. → [[Plugins y mods]], [[omh plugin]]
