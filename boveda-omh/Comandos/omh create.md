---
tipo: comando
tags: [omh/comando]
aliases: [create, PASO 1]
fuente: "omh.py · cmd_create (L1064)"
rust: "src/create.rs"
---
# omh create (PASO 1)

Elige edición, core y versión, descarga el servidor (y Java si hace falta) y deja el servidor como [[Servidor activo]].

## Opciones (texto de ayuda exacto)
- `--edition`, `-e`: `java` o `bedrock` (lo valida argparse). `edicion de Minecraft`
- `--core`, `-c`: `java: vanilla, paper, purpur, fabric, forge, neoforge | bedrock: vanilla, pocketmine, nukkit`. Acepta `powernukkitx` como sinónimo de `nukkit`.
- `--version`, `-v`: `version de Minecraft o 'latest'`
- `--dir`, `-d`: `carpeta donde se instala el servidor`
- `--name`, `-n`: `nombre del servidor (por defecto el de la carpeta)`
- `--eula`: `aceptar el EULA de Minecraft (https://aka.ms/MinecraftEULA)`
- `--force`: `reinstalar si ya existe un servidor en la carpeta`

## Preguntas si falta la opción → [[Preguntas interactivas]]
1. `Edicion [java/bedrock] (por defecto: java)`
2. Lista los cores (`  <core en 10 columnas> <descripción>`) y pregunta `Core [...] (por defecto: paper)` en Java o `vanilla` en Bedrock.
3. En Java muestra `Versiones comunes: ...` y pregunta `Version (o latest) (por defecto: latest)`.
4. `Carpeta del servidor (por defecto: ./mc-<core>-<version>)` (`.\mc-…` en Windows).
5. En Java, sin `--eula` ni EULA previo: `Aceptas el EULA de Minecraft? [s/N]`.

## Qué hace, en orden
1. Título ASCII y `== PASO 1 de 3 - Edicion, core y version ==`.
2. Valida la edición (`Edicion no valida: %s (usa java o bedrock)`) y el core (`Core no valido para %s: %s. Opciones: %s`).
3. Resuelve la versión → [[Catalogo de cores y versiones]] (puede avisar o fallar).
4. Carpeta: `abspath(expanduser(...))`. Si ya tiene [[omh.json]] y no hay `--force`: `Ya existe un servidor en %s. Usa --force para reinstalarlo.`
5. Nombre: `--name`, o el de la carpeta, o `servidor`; lo que no sea `[A-Za-z0-9_.-]` pasa a `-`.
6. Crea `<carpeta>/.omh/` → [[Carpeta .omh]].
7. Calcula qué Java necesita → [[Java Temurin]] y muestra `Servidor: <nombre> | <edición> <core> <versión> | <carpeta>`.
8. Descarga Java antes que el servidor, salvo en forge/neoforge (lo hace su instalador).
9. `== Descargando el servidor ==` y ejecuta el instalador del core → [[Mapa de cores]]. Lo que devuelve se guarda en `start`.
10. EULA: en Java → [[EULA]]; en Bedrock vanilla avisa de que se aceptan el EULA y la política de privacidad de Microsoft.
11. `step = 1`, guarda [[omh.json]], lo registra en [[state.json]] y lo marca como activo.
12. `[OK] Servidor '<nombre>' creado en <carpeta>` y el siguiente paso (`omh resources --ram 4G --storage 20G --cores 2`).

## Detalles que hay que conservar
- `omh.json` solo se escribe al final: si la descarga falla, el servidor no queda registrado (aunque sí los archivos descargados y algún `.part`).
- `--force` crea una configuración nueva: se pierden `resources` y `network`.
- Reinstalar Bedrock vanilla extrae el zip encima y sobrescribe `server.properties`, `allowlist.json` y `permissions.json`.
- Si la carpeta ya tenía el EULA aceptado, vuelve a escribir `eula.txt` sin preguntar.

Relacionado: [[Flujo de 3 pasos]], [[omh resources]], [[omh list]], [[Descargas verificadas]].
