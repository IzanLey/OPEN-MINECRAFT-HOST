---
tipo: archivo
tags: [omh/archivo]
aliases: [~/.open-mc-host, HOME_DIR, playit.toml]
fuente: "omh.py · HOME_DIR (L47)"
rust: "src/config.rs, src/platform.rs"
---
# Carpeta global (`~/.open-mc-host/`)

`HOME_DIR = expanduser("~")/.open-mc-host`. En Windows `~` es `%USERPROFILE%` y en Linux `$HOME` → [[Rutas y plataforma]].

| Ruta | Contenido |
|---|---|
| `state.json` | [[state.json]] |
| `java/<versión>/` | Una carpeta por versión mayor de Java (8, 17, 21, 25…) → [[Java Temurin]] |
| `tools/playit-v0.17.1` (`.exe` en Windows) | Agente de playit.gg → [[Tunel playit.gg]] |
| `playit.toml` | **Clave secreta del agente.** La crea y la usa playit; omh solo le pasa la ruta (`--secret_path`) y nunca la lee |

> [!danger] Secreto
> `playit.toml` da control sobre el túnel. Está fuera de cualquier repositorio y el `.gitignore` excluye `*.toml` y `playit*` por si acaso. Esta bóveda no guarda ni debe guardar su contenido.

Puede haber restos de versiones anteriores, como `tools/playit.exe`, que omh ya no usa.
