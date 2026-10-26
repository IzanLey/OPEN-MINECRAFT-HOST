---
tipo: migracion
tags: [omh/migracion]
aliases: [compatibilidad, formato de archivos]
---
# Compatibilidad de formatos

Lo que tiene que quedar **igual** para que los servidores creados con la 0.0.1 sigan funcionando y para que las dos versiones puedan convivir.

## Archivos (igual byte a byte en el caso normal)
| Archivo | Formato | Nota |
|---|---|---|
| [[omh.json]], [[state.json]] | `json.dump(indent=2)`, `ensure_ascii`, orden de claves, sin `\n` final | Rust lee también las variantes «a medias» → [[Bugs y ambiguedades#B1]] |
| [[run.json]] | `json.dump` compacto (`", "`, `": "`) | permisos `600` en Linux |
| `whitelist.json`, `ops.json`, `allowlist.json` | `indent=2`, conservando entradas y orden de claves del servidor | → [[Listas de jugadores]] |
| `white-list.txt`, `ops.txt` | `\n`, con `\n` final si hay nombres | |
| [[server.properties]] | `set_properties` con `splitlines` de Python, `\n` y UTF-8 | |
| `eula.txt` | texto exacto → [[EULA]] | |
| `php.ini` (PocketMine, Linux) | sin líneas `extension_dir…` y con la nueva al final | |
| `.omh/console.log` | UTF-8, añadiendo, `[omh] ` delante de los mensajes propios | |

En **Windows**, Python escribe todos esos archivos de texto con `\r\n` (modo texto); Rust hace lo mismo (`util::write_text`), comprobado con los archivos reales de `mc-vanilla-1.14.1`.

## Comportamiento
- Mismos comandos, opciones, opciones cortas, valores por defecto, preguntas y validaciones.
- Mismos textos (sin tildes), mismas sangrías y mismo título ASCII → [[Mensajes y titulo]].
- Mismos códigos de salida → [[Errores y codigos de salida]].
- Mismo protocolo del [[Canal de control]].
- Mismas rutas: `~/.open-mc-host/java/<n>`, `tools/playit-v0.17.1[.exe]`, `playit.toml` → [[Carpeta global]].
- Rutas relativas en `start` con el separador del sistema.

## Lo que no puede ser idéntico
- El formato de `--help` y los errores de sintaxis (clap frente a argparse) → [[Decisiones tecnicas#D8]].
- El «motivo» de los errores de red y de E/S (lo da el sistema o la biblioteca).
- Las trazas de Python, que pasan a `[ERROR]` claros.
- `User-Agent` y `--version` llevan `0.1.0`.
- `omh update` descarga un binario en vez de `omh.py` → [[Distribucion del binario]].
- La salida por pantalla redirigida a un archivo en Windows: Python escribia `\r\n` y Rust escribe `\n` (en la consola se ve igual).
