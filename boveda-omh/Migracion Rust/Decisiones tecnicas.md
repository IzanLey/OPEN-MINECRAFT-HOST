---
tipo: migracion
tags: [omh/migracion]
aliases: [decisiones]
---
# Decisiones técnicas

## D1
**Síncrono con hilos, sin tokio directo.** `reqwest::blocking` para HTTP. El supervisor usa hilos como en Python (salida del servidor, canal de control, vigilancia de disco, stdin) → [[Supervisor (Runner)]]. Así el código se parece al original y es más fácil comprobar que se comporta igual.

## D2
**Binarios sin dependencias.** Linux: `x86_64-unknown-linux-musl`, enlazado estático, funciona en cualquier distribución (un binario glibc compilado en Ubuntu 24.04 no arranca en Debian 11 ni en Ubuntu 20.04). Windows: `x86_64-pc-windows-msvc` con CRT estático (`+crt-static`), para no depender de `vcruntime140.dll`.

## D3
**JSON idéntico al de Python.** Un `Formatter` propio de serde_json reproduce `json.dump`: `indent=2`, separadores `", "`/`": "` en compacto, `\uXXXX` en minúsculas para todo lo que no es ASCII imprimible (incluidos pares sustitutos y `\u007f`) y orden de claves → [[Compatibilidad de formatos]].

## D4
**Rutas como Python.** `abspath` léxico (en Windows con `GetFullPathNameW`, es decir, `std::path::absolute`); en Linux, normalización de `..` propia, porque `std::path::absolute` no la hace. Carpeta de usuario: `%USERPROFILE%` en Windows y `$HOME` en Linux, y `dirs` como respaldo. Además, cambiando esa variable se pueden hacer pruebas aisladas sin tocar tu `~/.open-mc-host` real.

## D5
**Afinidad nativa.** Linux: `setsid` + `sched_setaffinity` en `pre_exec`. Windows: `SetProcessAffinityMask` en cuanto se lanza el proceso, que es lo mismo que hacía `(Get-Process).ProcessorAffinity`. No uso un Job Object con «matar al cerrar»: si muriera el supervisor, mataría a Java sin guardar, y Python no lo hace → [[Limite de nucleos]].

## D6
**Cierre forzado igual que en Python**: `taskkill /PID <pid> /T /F` en Windows (mata también los hijos; es un ejecutable del sistema, no PowerShell) y `SIGTERM` en Linux → [[omh stop]].

## D7
**Ctrl+C** con un manejador global (`ctrlc`) que actúa según el modo: normal → `Cancelado.` y 130; supervisor en primer plano → envía `stop` una vez; consola → sale con 0; túnel → cierra el agente y sale con 0 → [[Ctrl+C y cancelacion]].

## D8
**Ayuda.** clap genera `--help` con un formato distinto del de argparse. Mismos textos de cada comando y opción, cabeceras en español («Uso», «Opciones», «Comandos»). Los errores de sintaxis de clap siguen en inglés (como los de argparse) y salen con código 2. Solo `--version` (sin `-V`), que imprime `open-mc-host 0.1.0`.

## D9
**Errores.** Mismos textos que Python para `fail`, HTTP (`El servidor remoto respondio N (url).`) y red (`Error de red: <motivo>. Comprueba tu conexion a internet.`). El `<motivo>` lo da el sistema o la biblioteca y no será idéntico. Donde Python da una traza, Rust da `[ERROR]` con un mensaje claro → [[Errores y codigos de salida]].

## D10
**Timeouts HTTP por operación** (conexión y lectura), como urllib: 30 s en JSON y 60 s en descargas. Nunca un timeout total, que cortaría descargas grandes en conexiones lentas.

## D11
**Protocolo de control idéntico** (JSON por línea con token): un `omh` en Rust puede parar o mandar comandos a un servidor lanzado por el `omh.py` antiguo, y al revés → [[Canal de control]].

## D12
**`omh update`** descarga el binario de la última release, lo comprueba con `SHA256SUMS` y lo sustituye. En Windows renombra antes el `.exe` en uso a `.old` (el supervisor en segundo plano lo mantiene abierto) → [[Distribucion del binario]].

## D13
**`.gitignore`**: se añaden `/target/` y `!Cargo.toml`, porque `*.toml` ignoraría el manifiesto. Se mantiene `playit*`, que protege la clave del agente, y por eso el módulo se llama `tunnel.rs`.

## D14
**Todo por stdout**, incluido `[ERROR]`, como Python. Las preguntas se escriben sin salto de línea y se vacía el buffer antes de leer.

## D15
**TLS** con rustls y el verificador de la plataforma, que usa los certificados del sistema como Python (también los de proxies corporativos instalados en el sistema).
