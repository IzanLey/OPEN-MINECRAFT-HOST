---
tipo: comando
tags: [omh/comando, omh/bug]
aliases: [logs, registro]
fuente: "omh.py · cmd_logs (L1870), tail_lines (L1862)"
rust: "src/runner/commands.rs"
---
# omh logs

`ver las ultimas lineas del registro`

- `--lines`, `-l` (entero, por defecto 50): `numero de lineas`.
- Imprime tal cual (sin sangría) las últimas N líneas de `.omh/console.log` → [[Carpeta .omh]]. Lee el archivo en UTF-8 sustituyendo bytes inválidos y lo parte en líneas al estilo de Python (`splitlines`).
- No comprueba si el servidor está en marcha. Si no hay registro, no imprime nada.

> [!bug] Curiosidad heredada
> Usa `lineas[-N:]`: con `-l 0` muestra **todo** y con `-l -5` se salta las 5 primeras líneas → [[Bugs y ambiguedades#B8]].
