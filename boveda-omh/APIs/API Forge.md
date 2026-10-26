---
tipo: api
tags: [omh/api]
aliases: [Forge promotions, maven.minecraftforge.net]
fuente: "omh.py · FORGE_PROMOS (L651), forge_promos (L703), install_forge (L885)"
rust: "src/cores/forge.rs"
---
# API Forge

## Promociones
`GET https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json` (se guarda en memoria durante la ejecución)
- `promos`: `{"<mc>-recommended": "<forge>", "<mc>-latest": "<forge>", …}`.
- Se usa la `recommended` y, si no hay, la `latest`.
- `latest` como versión de Minecraft: la más alta de las claves cuyo prefijo es una release.

## Instalador
`https://maven.minecraftforge.net/net/minecraftforge/forge/<nombre>/forge-<nombre>-installer.jar`, probando dos nombres:
1. `<mc>-<forge>`
2. `<mc>-<forge>-<mc>` (formato de algunas versiones antiguas)
Un 404 pasa al siguiente; cualquier otro error HTTP corta. Si fallan los dos: `No se encontro el instalador de Forge para %s.`

Sin hash publicado. Lo usa [[Core forge]].
