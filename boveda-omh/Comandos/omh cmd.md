---
tipo: comando
tags: [omh/comando]
aliases: [cmd]
fuente: "omh.py · cmd_cmd (L1908)"
rust: "src/runner/commands.rs"
---
# omh cmd

`enviar un comando a la consola del servidor`

- `texto…` (uno o más; ayuda: `comando, por ejemplo: say hola`). Las palabras se unen con espacios.
- Se envía por el [[Canal de control]]. Si no está en marcha: `El servidor no esta en ejecucion.`
- Si se envía: `[OK] Enviado: <comando>`. No muestra la respuesta del servidor; para verla, [[omh logs]] o [[omh console]].
- El supervisor quita `\r`, cambia `\n` por espacios y recorta espacios antes de escribir en la consola del servidor → [[Supervisor (Runner)]].
- Las opciones se pueden mezclar con el texto (`omh cmd say hola --dir X`), como en argparse.
