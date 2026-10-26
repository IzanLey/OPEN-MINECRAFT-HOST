---
tipo: concepto
tags: [omh/concepto]
aliases: [build_command, start, JVM, argsfile]
fuente: "omh.py · build_command (L1534), Runner.run (L1742-1756)"
rust: "src/runner/mod.rs"
---
# Comando de arranque

`build_command(cfg)` construye la línea de comandos según `start.type` de [[omh.json]]:

| Tipo | Comando |
|---|---|
| `jar` | `<java> <JVM> -jar <jar>`, y `nogui` en la edición Java (no en Nukkit) |
| `argsfile` | `<java> <JVM> @<start.dir>/win_args.txt` (Windows) o `unix_args.txt` (Linux) `nogui` |
| `native` | `<carpeta>/<exe>`; en Linux con `LD_LIBRARY_PATH=<carpeta>` (sustituye el que hubiera) |
| `php` | `<carpeta>/<php> <carpeta>/<phar> --no-wizard` |

**JVM**: `-Xms<ram>M -Xmx<ram>M -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:ActiveProcessorCount=<núcleos>`.

`<java>` sale de `ensure_java(cfg.java)`, que lo descarga si falta → [[Java Temurin]]. El entorno es una copia del de omh.

## Cómo se lanza (en el [[Supervisor (Runner)]])
- Carpeta de trabajo: la del servidor (por eso `@libraries/…` y `-jar server.jar` son relativos).
- stdin y stdout con tuberías; stderr va a la misma tubería que stdout.
- Windows: `CREATE_NEW_PROCESS_GROUP` (0x200), para que Ctrl+C en la terminal no mate a Java sin guardar.
- Linux: nueva sesión (`setsid`) por el mismo motivo, más la afinidad → [[Limite de nucleos]].
