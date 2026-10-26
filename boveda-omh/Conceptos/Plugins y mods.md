---
tipo: concepto
tags: [omh/concepto]
aliases: [addons, addons_dir, safe_filename, plugins/, mods/]
fuente: "omh.py · addons_dir (L2097), safe_filename (L2106)"
rust: "src/plugins.rs"
---
# Plugins y mods

| Core | Carpeta | Fuente para `add <nombre>` |
|---|---|---|
| paper, purpur | `plugins/` | [[API Modrinth]] |
| fabric, forge, neoforge | `mods/` | [[API Modrinth]] |
| pocketmine | `plugins/` | [[API Poggit]] |
| nukkit | `plugins/` | solo URL o archivo `.jar` |
| vanilla (Java y Bedrock) | — | no admite complementos |

En todos se puede usar una URL `http(s)://` o un archivo local.

## Nombre de archivo seguro (`safe_filename`)
`basename(unquote(texto sin la parte ?…))` y cambia por `_` todo lo que no sea `[A-Za-z0-9_.+-]`. Si queda vacío: `addon.jar`.

## Geyser
Para que entren jugadores de Bedrock en un servidor Java (paper/purpur): `omh plugin add geyser`. Usa el puerto 19132/UDP.

Hay que reiniciar el servidor para cargar o quitar complementos. → [[omh plugin]]
