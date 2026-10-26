---
tipo: mapa
tags: [omh/mapa]
aliases: [Comandos, Subcomandos]
fuente: "omh.py · build_parser (L2258)"
---
# Mapa de comandos

| Comando | Para qué | Opciones | Paso previo |
|---|---|---|---|
| [[omh (sin comando)\|omh]] | Título y guía | `--version`, `-h` | — |
| [[omh create]] | PASO 1: edición, core, versión e instalación | `-e/--edition`, `-c/--core`, `-v/--version`, `-d/--dir`, `-n/--name`, `--eula`, `--force` | — |
| [[omh resources]] | PASO 2: RAM, almacenamiento, núcleos | `-r/--ram`, `-s/--storage`, `-c/--cores` | 1 |
| [[omh network]] | PASO 3: red, puerto, whitelist, admins | `-m/--mode`, `--public-method`, `-p/--port`, `--ip`, `-w/--whitelist`, `-a/--admin` (repetible), `--online-mode`, `--max-players`, `--motd` | 2 |
| [[omh start]] | Iniciar | `--detach`, `--daemon` (oculta) | 3 |
| [[omh stop]] | Parar guardando el mundo | — | — |
| [[omh restart]] | Parar y arrancar en segundo plano | — | 3 |
| [[omh status]] | Configuración, disco, direcciones, estado | — | — |
| [[omh console]] | Consola de un servidor en segundo plano | — | en marcha |
| [[omh logs]] | Últimas líneas del registro | `-l/--lines` (50) | — |
| [[omh cmd]] | Enviar un comando a la consola | `texto…` (1 o más) | en marcha |
| [[omh whitelist]] | Gestionar la whitelist | `add`, `remove`, `list`, `on`, `off` + `[jugador…]` | 1 |
| [[omh admin]] | Gestionar administradores (op) | `add`, `remove`, `list` + `[jugador…]` | 1 |
| [[omh plugin]] | Plugins y mods | `search`, `add`, `remove`, `list` + `[objetivo…]` | 1 |
| [[omh eula]] | Aceptar el EULA (Java) | `--accept` | — |
| [[omh tunnel]] | Vincular el túnel de playit.gg | — | — |
| [[omh list]] | Ediciones, cores y versiones | — | — |
| [[omh servers]] | Servidores creados | — | — |
| [[omh use]] | Elegir el servidor activo | `objetivo` | — |
| [[omh update]] | Actualizar omh | — | — |

## Reglas comunes
- **`--dir`**: lo tienen todos los subcomandos salvo `list`, `servers`, `use` y `update`. `create` usa el suyo (`-d/--dir`). Ayuda: `carpeta del servidor (por defecto el servidor activo)`.
- Sin `--dir` se usa la carpeta actual si tiene [[omh.json]] y, si no, el [[Servidor activo]].
- **Paso previo**: si falta, `Falta el paso N. Ejecuta primero: <comando>` → [[Flujo de 3 pasos]].
- **Errores**: `  [ERROR] …` y código 1; sintaxis inválida: argparse, código 2; Ctrl+C: `Cancelado.`, código 130 → [[Errores y codigos de salida]].
- Si faltan opciones obligatorias, el comando pregunta → [[Preguntas interactivas]].

← [[OPEN-MC-HOST]]
