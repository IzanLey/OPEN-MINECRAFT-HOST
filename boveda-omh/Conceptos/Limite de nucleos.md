---
tipo: concepto
tags: [omh/concepto, omh/bug]
aliases: [afinidad, cores, ActiveProcessorCount, sched_setaffinity]
fuente: "omh.py · build_command (L1541), Runner.run (L1740-1760)"
rust: "src/platform.rs, src/runner/mod.rs"
---
# Límite de núcleos

`resources.cores` ([[omh resources]]) se aplica de tres formas:

1. **JVM** (servidores `jar` y `argsfile`): siempre `-XX:ActiveProcessorCount=<N>` → [[Comando de arranque]].
2. **Afinidad de CPU**, solo si `N < CPUs del sistema`:
   - **Linux**: en el proceso hijo, antes de `exec` (`preexec_fn`), `os.sched_setaffinity(0, {0, 1, …, N-1})`.
   - **Windows**: justo después de lanzar el servidor, `powershell -NoProfile -Command "(Get-Process -Id <pid>).ProcessorAffinity = <(1<<N)-1>"` sin mostrar nada.
3. **Bedrock vanilla**: `max-threads=<N>` en [[server.properties]].

Afecta a cualquier tipo de servidor (java, bedrock_server, php), y los procesos que lance el servidor heredan la afinidad.

## En Rust
Afinidad nativa, sin PowerShell → [[Decisiones tecnicas#D5]]:
- Linux: `sched_setaffinity` en `pre_exec`, junto con el `setsid`.
- Windows: `SetProcessAffinityMask` en cuanto se lanza el proceso (es lo mismo que hace `ProcessorAffinity`).

> [!bug] Heredado de Python (decidido)
> Si el proceso solo puede usar otras CPUs (contenedor, `taskset`), fijar `0..N-1` falla y Python revienta → [[Bugs y ambiguedades#B4]].
