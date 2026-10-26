---
tipo: archivo
tags: [omh/archivo]
aliases: [whitelist.json, ops.json, allowlist.json, permissions.json, white-list.txt, ops.txt]
fuente: "omh.py · list_files (L1988), read/write_json_list (L1961), read/write_txt_list (L1975)"
rust: "src/players.rs"
---
# Listas de jugadores

| Core | Whitelist | Administradores |
|---|---|---|
| Java (todos) | `whitelist.json` | `ops.json` |
| Bedrock vanilla | `allowlist.json` | `permissions.json` |
| PocketMine, Nukkit | `white-list.txt` | `ops.txt` |

## Formatos
- **`whitelist.json`**: lista de `{"uuid": "…", "name": "…"}`.
- **`ops.json`**: lista de `{"uuid": "…", "name": "…", "level": 4, "bypassesPlayerLimit": false}`.
- **`allowlist.json`**: omh añade `{"ignoresPlayerLimit": false, "name": "…"}`.
- **`permissions.json`**: lo escribe el propio BDS (`{"permission": "operator", "xuid": "…"}`); omh no lo edita porque no conoce el XUID.
- **`.txt`**: un nombre por línea, en minúsculas al añadirlo. Al leer se ignoran líneas vacías y las que empiezan por `#` en la primera columna.

## Lectura y escritura
- JSON que no se puede leer o que no es una lista cuenta como lista vacía.
- Al escribir: `json.dump(indent=2)`, conservando las entradas y el orden de claves que puso el servidor → [[Compatibilidad de formatos]].
- `.txt`: líneas unidas con `\n` y `\n` final si hay alguna.
- Al añadir, primero se quita la entrada del mismo nombre (sin distinguir mayúsculas) y después se añade al final.
- Mostrar: en los JSON se usa `name`, o `xuid`, o `?`.

→ [[Whitelist y administradores]], [[UUID de jugadores]]
