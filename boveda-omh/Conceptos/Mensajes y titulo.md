---
tipo: concepto
tags: [omh/concepto]
aliases: [info, ok, warn, header, titulo ASCII, print_title]
fuente: "omh.py · FONT (L58), render_word (L153), print_title (L167), info/ok/warn/header (L210-224)"
rust: "src/ui.rs"
---
# Mensajes y título

## Tipos de línea (todas por **stdout**)
| Función | Formato |
|---|---|
| `info(msg)` | `  msg` |
| `ok(msg)` | `  [OK] msg` |
| `warn(msg)` | `  [AVISO] msg` |
| `header(msg)` | línea vacía y `  == msg ==` |
| error (en `main`) | `  [ERROR] msg` → [[Errores y codigos de salida]] |
| `Runner.say(msg)` | consola: `  [omh] msg`; `console.log`: `[omh] msg` → [[Supervisor (Runner)]] |

Los textos del programa están en español **sin tildes ni eñes** («Edicion», «nucleos», «anadido») para que se vean bien en cualquier consola. Hay que conservarlos así en Rust → [[Compatibilidad de formatos]].

## Título ASCII
- Una fuente propia con las letras `O P E N M I C R A F T H S`, en el estilo «standard» de figlet (5 filas).
- `render_word` une las letras solapando una columna cuando en cada fila una de las dos tiene un espacio en ese borde (como figlet).
- `print_title`: línea vacía; las palabras `OPEN`, `MINECRAFT` y `HOST` centradas respecto a la más ancha, cada fila con 2 espacios delante y sin espacios al final, y una línea vacía tras cada palabra; después `  Convierte tu PC en un servidor de Minecraft  -  v<versión>` y `  ` más tantos `-` como el ancho máximo.
- Lo muestran [[omh (sin comando)]], [[omh create]] y [[omh list]].

```text
                     ___   ____   _____  _   _
                    / _ \ |  _ \ | ____|| \ | |
                   | | | || |_) ||  _|  |  \| |
                   | |_| ||  __/ | |___ | |\  |
                    \___/ |_|    |_____||_| \_|
```
(primera palabra; el README tiene las tres)

En Rust, el título tiene que salir idéntico carácter a carácter: se comprobará con un test contra la salida de Python → [[Plan de pruebas]].
