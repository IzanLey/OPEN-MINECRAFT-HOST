---
tipo: core
tags: [omh/core, omh/bug]
aliases: [nukkit, PowerNukkitX, powernukkitx]
fuente: "omh.py · install_nukkit (L1011)"
rust: "src/cores/nukkit.rs"
---
# Core nukkit (PowerNukkitX)

`PowerNukkitX, fork mantenido de Nukkit (Java). Plugins .jar.`

| | |
|---|---|
| Edición | bedrock |
| Sinónimo | `--core powernukkitx` |
| Versión | siempre la última release (`tag_name`, o `latest`) |
| Java | 21 → [[Java Temurin]] |
| Complementos | plugins `.jar` por URL o archivo local (sin buscador) |
| Puerto | 19132/UDP |

## Instalación → [[API GitHub]]
1. Última release de `PowerNukkitX/PowerNukkitX`.
2. Si hay `.jar`: prefiere los que contienen `all` o `shaded` y, entre ellos, el más grande. Lo descarga como `powernukkitx.jar` → `{"type": "jar", "jar": "powernukkitx.jar"}`.
3. Si no hay jar y sí `.zip`: descarga el primero a `.omh/`, lo extrae en la carpeta del servidor y busca todos los `*.jar` recursivamente. Descarta los que están en `plugins/` o `libs/`, se queda el más grande y guarda su ruta relativa.
4. Si no hay nada: `No se encontro ningun archivo descargable de PowerNukkitX.` o `No se encontro el jar de PowerNukkitX.`

Jugadores en `white-list.txt` y `ops.txt` (como PocketMine) → [[Listas de jugadores]].

> [!bug] Heredado de Python (decidido)
> El filtro de `plugins`/`libs` mira la ruta **absoluta** → [[Bugs y ambiguedades#B6]].
