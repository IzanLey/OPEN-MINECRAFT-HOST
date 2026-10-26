---
tipo: concepto
tags: [omh/concepto]
aliases: [platform_ids, IS_WIN, total_ram_mb, dir_size_mb, abspath]
fuente: "omh.py · IS_WIN (L46), HOME_DIR (L47), dir_size_mb (L300), total_ram_mb (L311), platform_ids (L342), make_executable (L359)"
rust: "src/platform.rs, src/util.rs"
---
# Rutas y plataforma

## Sistema
- `IS_WIN = os.name == "nt"`. Lo demás se trata como Linux (también macOS, aunque no está soportado oficialmente).
- `platform_ids()` → `(sistema, arquitectura)`: `windows`, `mac` o `linux`; `x86_64`/`amd64` → `x64`, `aarch64`/`arm64` → `aarch64` y cualquier otra tal cual. Lo usan la [[API Adoptium]], [[Core vanilla (Bedrock)]] (solo `windows`/`linux` + `x64`) y el [[Tunel playit.gg]].

## Rutas
- Carpeta de usuario `~`: en Windows `%USERPROFILE%` (si no existe, `HOMEDRIVE`+`HOMEPATH`); en Linux `$HOME` (si no existe, la del usuario en `/etc/passwd`). De ahí sale `~/.open-mc-host` → [[Carpeta global]].
- `os.path.abspath`: une con la carpeta actual y normaliza **léxicamente** (`..`, `.`, separadores dobles) sin resolver enlaces. En Windows usa `GetFullPathNameW`.
- Rutas relativas guardadas en [[omh.json]] (`start.dir`, `start.php`): con el separador del sistema.

## Recursos del equipo
| Dato | Windows | Linux | Si falla |
|---|---|---|---|
| RAM total | `GlobalMemoryStatusEx().ullTotalPhys` | `MemTotal` de `/proc/meminfo` | 0 → «desconocida» |
| Núcleos | `os.cpu_count()`: CPUs lógicas del **sistema** (no las permitidas por afinidad o cgroups) | ídem (`sysconf(_SC_NPROCESSORS_ONLN)`) | 1 |
| Disco libre | `GetDiskFreeSpaceExW` de la carpeta del servidor | `statvfs` (`f_bavail * f_frsize`) | traza |
| Tamaño del servidor | `dir_size_mb`: suma de los tamaños de todos los archivos (sigue los enlaces a archivos, no los de carpetas; ignora errores), dividida entre 1 MB | | |

## Permisos
`make_executable(ruta)`: en Linux añade `0o755` a los permisos actuales; en Windows no hace nada.
