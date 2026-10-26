---
tipo: migracion
tags: [omh/migracion]
aliases: [tests, pruebas]
---
# Plan de pruebas

## 1. Tests unitarios (`cargo test`, también en CI)
| Módulo | Qué se prueba |
|---|---|
| `util` | `parse_size_mb` (4G, 4096M, 4GB, 1.5G, 2GiB, 512K, errores), `fmt_mb`, `vt` y su orden, `is_release`, `valid_player`, `splitlines`/`strip` de Python, cola de líneas con la semántica de `[-N:]`, `safe_filename`, `quote`, `abspath` |
| `pyjson` | Salida idéntica a Python: sangría, separadores, `é`, emojis (pares sustitutos), controles, `\u007f`, listas y objetos vacíos |
| `config` | omh.json de Python → cargar → guardar = mismos bytes; red a medias; `set_properties` (comentarios, repetidas, CRLF, claves nuevas al final); `state.json` corrupto |
| `ui` | Título y guía idénticos a la salida de `omh.py` (texto de referencia generado con Python) |
| `extract` | zip y tar con `../`, rutas absolutas, enlaces simbólicos y duros que salen fuera → error; comprimidos normales → bien |
| `catalog` | `resolve_version` sin red (mínimos, paper 1.8.x → 1.8.8, versiones no válidas), `java_needed` de respaldo |
| `network` | `port_free`, `find_free_port` (excluidos, `need_next`), validación de IP al estilo `inet_aton`, cuerpo SOAP |
| `players` | UUID offline (`Notch` → `b50ad385-829d-3141-a216-7e7d7539ba7f`), formatos JSON/txt de cada core |
| `runner/control` | ping, cmd y token incorrecto contra un supervisor de prueba; `run.json` compacto |

## 2. Comparación con Python (local)
Con `legacy/omh.py` al lado (Python 3.15 en Windows y 3.14 en WSL), las mismas órdenes con las dos versiones y `diff` de: salida de `omh` y `omh list`, `omh.json`, `state.json`, `server.properties` y las listas de jugadores.

## 3. Prueba real (Windows y WSL Ubuntu)
Con una carpeta de usuario **aislada** (`USERPROFILE`/`HOME` apuntando a una carpeta temporal), para no tocar tu `~/.open-mc-host` ni tu servidor `mc-vanilla-1.14.1`:
1. `omh create --edition java --core paper --version latest --dir <tmp>/paper --eula` y lo mismo con `vanilla`.
2. `omh resources --ram 2G --storage 10G --cores 2`.
3. `omh network --mode local --port auto --whitelist off`.
4. `omh start --detach` → esperar «listo» → `omh status` (`EN EJECUCION`) → `omh cmd say hola` → `omh logs` → `omh stop`.
5. Compatibilidad cruzada: arrancar con `omh.py` y parar con el binario de Rust, y al revés.

> [!important] Permiso concedido (2026-10-26)
> Arrancar un servidor Java exige aceptar el EULA de Minecraft (`--eula`); se acepta solo en las carpetas de prueba.

## 4. CI
`fmt --check`, `clippy -D warnings`, `test` y `build --release` en `ubuntu-latest` y `windows-latest` → [[CI y releases]].
