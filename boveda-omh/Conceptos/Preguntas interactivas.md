---
tipo: concepto
tags: [omh/concepto]
aliases: [ask, confirm, asistente, modo interactivo]
fuente: "omh.py · interactive (L227), ask (L231), confirm (L259)"
rust: "src/ui.rs"
---
# Preguntas interactivas

Hay terminal interactiva si stdin es una TTY (`sys.stdin.isatty()`).

## `ask(pregunta, por_defecto=None, opciones=None)`
- **Sin terminal**: devuelve el valor por defecto; si no hay, `Falta un valor obligatorio: <pregunta>`.
- Texto que muestra: `  <pregunta>[ [a/b/c]][ (por defecto: X)]: `. La parte `(por defecto: …)` no aparece si el valor es `None` o vacío.
- Bucle:
  - fin de entrada (EOF): por defecto o error, igual que sin terminal;
  - respuesta vacía con valor por defecto (aunque sea `""`): devuelve el valor por defecto **tal cual**, sin pasarlo a minúsculas;
  - respuesta vacía sin valor por defecto: vuelve a preguntar;
  - con opciones y una respuesta que no está (en minúsculas): `Opciones validas: a, b, c` y vuelve a preguntar;
  - devuelve la respuesta en minúsculas si hay opciones, o tal cual (sin espacios a los lados) si no.

## `confirm(pregunta, por_defecto=False)`
- Sin terminal: el valor por defecto (por eso sin terminal el EULA no se acepta).
- Muestra `  <pregunta> [S/n]: ` si el defecto es sí, o `[s/N]: ` si es no.
- EOF: salto de línea y el defecto. Vacío: el defecto. Cuentan como sí `s`, `si`, `y` y `yes` (sin tilde).

## Dónde se pregunta
[[omh create]] (edición, core, versión, carpeta, EULA), [[omh resources]] (RAM, almacenamiento, núcleos), [[omh network]] (modo, método, puerto, whitelist, admin) y [[omh start]] en primer plano (EULA).

Ctrl+C durante una pregunta: `Cancelado.` y código 130 → [[Ctrl+C y cancelacion]].
