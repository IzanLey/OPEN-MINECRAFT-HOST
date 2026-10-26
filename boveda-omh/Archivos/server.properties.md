---
tipo: archivo
tags: [omh/archivo]
aliases: [set_properties]
fuente: "omh.py · set_properties (L581), apply_network_properties (L1340)"
rust: "src/config.rs"
---
# server.properties

omh solo cambia **las claves que le interesan** y conserva el resto del archivo.

## Algoritmo `set_properties(ruta, valores)`
1. Lee el archivo (si existe) en UTF-8, sustituyendo bytes inválidos, y lo parte en líneas con `splitlines()` de Python (`\n`, `\r\n`, `\r` y otros separadores Unicode).
2. Línea a línea: si no está vacía, no empieza por `#` y contiene `=`, la clave es lo que hay antes del primer `=` sin espacios. Si esa clave está en `valores`, la línea se cambia por `clave=valor` (todas las apariciones). Las demás líneas quedan igual.
3. Las claves que no estaban se añaden al final, en el orden de `valores`.
4. Escribe en UTF-8, con `\n` entre líneas y un `\n` final.

`!` no se trata como comentario y los valores no se escapan (el motd se escribe tal cual).

## Claves por edición ([[omh network]])
| Clave | Java | Bedrock vanilla | PocketMine / Nukkit |
|---|---|---|---|
| Puerto | `server-port` | `server-port` y `server-portv6` (puerto+1) | `server-port` |
| IP | `server-ip` (vacía = todas) | — | `server-ip` (vacía → `0.0.0.0`) |
| Whitelist | `white-list` y `enforce-whitelist` (`true`/`false`) | `allow-list` (`true`/`false`) | `white-list` (`on`/`off`) |
| Cuentas oficiales | `online-mode` (`true`/`false`) | `online-mode` (`true`/`false`) | `xbox-auth` (`on`/`off`) |
| Jugadores | `max-players` | `max-players` | `max-players` |
| Nombre | `motd` | `server-name` | `motd` |

[[omh resources]] añade `max-threads=<núcleos>` en Bedrock vanilla. [[omh whitelist]] `on`/`off` vuelve a aplicar todas las claves de red.
