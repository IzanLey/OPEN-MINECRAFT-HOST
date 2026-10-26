---
tipo: archivo
tags: [omh/archivo]
aliases: [run file]
fuente: "omh.py · run_file (L1499), Runner.run (L1761)"
rust: "src/runner/control.rs"
---
# run.json

`<servidor>/.omh/run.json`. Lo escribe el [[Supervisor (Runner)]] nada más lanzar el servidor y lo borra al terminar.

```json
{"port": 51234, "token": "00112233445566778899aabbccddeeff", "pid": 4520, "wrapper_pid": 9764, "started": "2026-10-26 00:05:51"}
```

| Clave | Significado |
|---|---|
| `port` | Puerto TCP en `127.0.0.1` del [[Canal de control]] (lo elige el sistema) |
| `token` | 16 bytes aleatorios en hexadecimal (`secrets.token_hex(16)`); se crea en cada arranque |
| `pid` | PID del servidor (java, bedrock_server o php) |
| `wrapper_pid` | PID del supervisor (`omh start`) |
| `started` | Hora local de arranque |

- Formato: `json.dump` compacto de Python, con `", "` y `": "` como separadores y sin salto final.
- En Linux se deja con permisos `600` (solo el usuario puede leer el token).
- Si el supervisor muere de golpe, el archivo se queda, pero no molesta: «en marcha» se decide con un ping al canal.
- [[omh stop]] lo usa para forzar el cierre (`pid` y `wrapper_pid`) si el servidor no responde en 90 s.
