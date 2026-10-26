---
tipo: migracion
tags: [omh/migracion]
aliases: [dependencias, Cargo.toml]
---
# Crates

## Sugeridos en el encargo
| Crate | Uso | Cómo |
|---|---|---|
| `clap` 4 (derive) | CLI | cabeceras de ayuda en español → [[Decisiones tecnicas#D8]] |
| `reqwest` 0.13 (`blocking`, `json`, `rustls`) | HTTP | sin OpenSSL. `rustls` usa el verificador de certificados del sistema, como Python. Sin el timeout total de 30 s que trae el cliente bloqueante → [[Decisiones tecnicas#D10]] |
| `serde` + `serde_json` (`preserve_order`) | JSON | orden de claves conservado al reescribir listas de jugadores |
| `sha1`, `sha2` | verificación de descargas | |
| `zip` (solo deflate) | Temurin en Windows, BDS, PHP, PowerNukkitX | |
| `tar` + `flate2` | Temurin y PHP en Linux | |
| `anyhow` + `thiserror` | errores con contexto + errores tipados (HTTP, red, fallo de omh) | |
| `dirs` | carpeta de usuario de respaldo | primero `USERPROFILE`/`HOME`, como Python |

### Sugeridos que no hacen falta
| Crate | Por qué no |
|---|---|
| `tokio` (directo) | Solo entra como dependencia interna de reqwest. El programa es secuencial salvo el supervisor, que con hilos queda igual que en Python. Con async habría que marcar como async casi todo y complicar las preguntas por stdin → [[Decisiones tecnicas#D1]] |
| `toml` | omh nunca lee `playit.toml`: solo pasa su ruta al agente |
| `sysinfo` | Python usa RAM total, CPUs lógicas del sistema y espacio libre **de la carpeta**. sysinfo no da lo último y puede contar RAM y CPUs de otra forma (cgroups). Con `libc`/`windows-sys`, que ya hacen falta para la afinidad, son unas pocas llamadas y el resultado es idéntico |

## Extra (justificados)
| Crate | Para qué |
|---|---|
| `regex` | Portar tal cual las ~12 expresiones de Python (versiones, tamaños, jugadores, nombres de archivo, SSDP/SOAP, «Done», enlace de playit, versión de BDS) sin reescribirlas a mano |
| `chrono` (solo `clock`) | Fecha local `AAAA-MM-DD HH:MM:SS` de `created` y `started`: la biblioteca estándar no tiene zona horaria |
| `md-5` | UUID offline `MD5("OfflinePlayer:<nombre>")` → [[UUID de jugadores]]. Misma familia RustCrypto que `sha1`/`sha2` |
| `getrandom` | Token de 128 bits del canal de control con el generador seguro del sistema (como `secrets.token_hex`), sin traer `rand` |
| `ctrlc` | Ctrl+C en Windows y Linux sin tokio → [[Decisiones tecnicas#D7]] |
| `libc` (solo Linux) | `sched_setaffinity`, `setsid`, `kill`, `statvfs`, `sysconf` |
| `windows-sys` (solo Windows) | `SetProcessAffinityMask`, `GlobalMemoryStatusEx`, `GetDiskFreeSpaceExW`, `GetActiveProcessorCount` |
| `tempfile` (solo tests) | Carpetas temporales en los tests de extracción y configuración |
