---
tipo: core
tags: [omh/core]
aliases: [pocketmine, PocketMine-MP, PHP]
fuente: "omh.py · install_pocketmine (L946)"
rust: "src/cores/pocketmine.rs"
---
# Core pocketmine

`PocketMine-MP (PHP). Plugins de Poggit.`

| | |
|---|---|
| Edición | bedrock |
| Versión | siempre la última release; se guarda su *tag* en `version` |
| Java | no (`java: null`); usa un PHP propio |
| Complementos | plugins `.phar` de [[API Poggit]], por URL o archivo |
| Puerto | 19132/UDP |

## Instalación → [[API GitHub]]
1. Última release de `pmmp/PocketMine-MP` y su asset `PocketMine-MP.phar`; si no está: `No se encontro PocketMine-MP.phar en la ultima version.` Lo descarga (etiqueta `PocketMine-MP <tag>`).
2. PHP:
   - `PM<mayor>` sale del tag (5.x → `PM5`); sistema `Windows` + `x64` o `Linux` + `x86_64`;
   - con el asset `build_info.json` → `php_download_url` → *tag* → release de `pmmp/PHP-Binaries` con ese tag;
   - si falla, las últimas 30 releases de `pmmp/PHP-Binaries`;
   - se salta borradores y prereleases y elige el primer asset cuyo nombre contenga sistema, arquitectura y `PM<mayor>`, sin `debug`, y que acabe en `.zip` o `.tar.gz`. Si no hay: `No se encontro un binario de PHP compatible para tu sistema.`
3. Descarga a `.omh/`, borra `bin/`, extrae en `bin/` → [[Extraccion segura]] y borra el archivo.
4. Busca el ejecutable: en Windows el primer `php.exe`; en Linux un `php` dentro de una carpeta `bin`. Si no: `No se encontro el ejecutable de PHP.` Le da permiso de ejecución.
5. Linux: si hay una carpeta `no-debug-zts-*` y un `php.ini` junto a php, quita las líneas `extension_dir…` y añade `extension_dir="<ruta absoluta>"`.
6. `start = {"type": "php", "php": "<ruta relativa>", "phar": "PocketMine-MP.phar"}` → [[Comando de arranque]] (`php PocketMine-MP.phar --no-wizard`).

## Particularidades
- No se puede fijar la RAM desde fuera.
- Jugadores en `white-list.txt` y `ops.txt`; propiedades `white-list` y `xbox-auth` con `on`/`off` → [[server.properties]].
- `extension_dir` es una ruta absoluta: si se mueve la carpeta del servidor, deja de valer.
