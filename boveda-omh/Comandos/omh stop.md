---
tipo: comando
tags: [omh/comando]
aliases: [stop, parar]
fuente: "omh.py · cmd_stop (L1833), kill_pid (L1821)"
rust: "src/runner/commands.rs"
---
# omh stop

Detiene el servidor guardando el mundo.

1. Si el [[Canal de control]] no responde: `El servidor '<nombre>' no esta en ejecucion.` y sale con 0.
2. `Deteniendo '<nombre>' (guardando el mundo)...` y envía `stop` a la consola del servidor.
3. Comprueba cada segundo, hasta 90 s, si sigue en marcha. Cuando para: `[OK] Servidor detenido.`
4. Si a los 90 s no ha parado: `[AVISO] El servidor no respondio en 90 segundos. Se fuerza el cierre.` Lee [[run.json]] y mata `pid` (el servidor) y `wrapper_pid` (el supervisor):
   - Windows: `taskkill /PID <pid> /T /F` (mata también los procesos hijos, como el agente de playit);
   - Linux: `SIGTERM`.
   Después borra `run.json`.

## Detalles
- Si el supervisor murió de golpe, `run.json` se queda (como pasa ahora en `mc-vanilla-1.14.1`). No molesta, porque «en marcha» se decide por el ping al canal y no por el archivo. Pero si Java se quedó huérfano, `omh stop` no lo ve.
- Un `omh stop` de la versión Rust tiene que poder parar un servidor lanzado por la versión Python, y al revés → [[Compatibilidad de formatos]].

Relacionado: [[omh start]], [[omh restart]], [[Supervisor (Runner)]].
