---
tipo: comando
tags: [omh/comando]
aliases: [console, consola]
fuente: "omh.py · cmd_console (L1876)"
rust: "src/runner/commands.rs"
---
# omh console

`abrir la consola de un servidor en segundo plano`

1. Si no está en marcha: `El servidor no esta en ejecucion. Inicialo con: omh start`.
2. Imprime tal cual (sin sangría) las últimas 30 líneas de `.omh/console.log` → [[Carpeta .omh]].
3. `-- Consola de '<nombre>'. Escribe comandos. Ctrl+C para salir (el servidor sigue). --`
4. Un hilo sigue el archivo desde el final (revisa cada 0,3 s) e imprime lo nuevo.
5. Cada línea escrita se envía por el [[Canal de control]]. Si el envío falla: `El servidor ya no esta en ejecucion.` y termina.
6. Ctrl+C o fin de entrada: salto de línea y sale con 0. El servidor sigue en marcha → [[Ctrl+C y cancelacion]].

Funciona también con un servidor iniciado en primer plano en otra terminal: lee el mismo `console.log`.
