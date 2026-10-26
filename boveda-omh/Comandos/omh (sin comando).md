---
tipo: comando
tags: [omh/comando]
aliases: [omh sin comando, omh --version, omh --help]
fuente: "omh.py · main (L2342), build_parser (L2258), print_title (L167), print_guide (L180)"
rust: "src/main.rs, src/cli.rs, src/ui.rs"
---
# omh (sin comando)

- `omh` sin subcomando muestra el título ASCII y la guía del primer paso, y sale con código 0. Es lo que ejecutan los instaladores al terminar → [[Instalacion]].
- `omh --version` → `open-mc-host 0.0.1` (en Rust será `open-mc-host 0.1.0`). Solo existe `--version`, sin `-V`.
- `omh -h` / `omh --help` → ayuda generada por argparse:
  - programa: `omh`
  - descripción: `OPEN MINECRAFT HOST - convierte tu PC en un servidor de Minecraft.`
  - epílogo: `Orden: 1) omh create  2) omh resources  3) omh network  4) omh start`
  - nombre de los subcomandos en el uso: `COMANDO`
- Cada subcomando tiene su `-h` con su descripción (la misma frase que aparece en la lista de comandos).

## La guía (`print_guide`)
Texto fijo: `PASO 1 de 3 - Crear el servidor (edicion, core y version):`, el ejemplo `omh create --edition java --core paper --version 1.21.1 --dir ./mi-servidor`, el aviso de que `omh create` sin opciones abre un asistente, `omh list` para ver cores y versiones, y los ejemplos de `PASO 2`, `PASO 3` e `INICIAR`. Termina con `Ayuda completa: omh --help  |  Ayuda de un comando: omh create --help`.

Relacionado: [[Mensajes y titulo]], [[Errores y codigos de salida]], [[Mapa de comandos]].
