---
tipo: comando
tags: [omh/comando]
aliases: [plugin, mods, plugins]
fuente: "omh.py · cmd_plugin (L2135), modrinth_install (L2111)"
rust: "src/plugins.rs"
---
# omh plugin

`buscar, instalar, listar o quitar plugins y mods`. Requiere el paso 1.

`omh plugin {search,add,remove,list} [objetivo…]` (ayuda del objetivo: `nombre en Modrinth/Poggit, URL o archivo local`). Las palabras del objetivo se unen con espacios.

Antes de cualquier acción comprueba que el core admite complementos y crea `plugins/` o `mods/`. Si no los admite: `El core %s no admite plugins ni mods. Usa paper/purpur (plugins) o fabric/forge/neoforge (mods).` → [[Plugins y mods]].

## list
`Plugins en <carpeta>:` (o `Mods en …`) y cada `.jar`, `.phar` o `.zip` ordenado, o `  (ninguno)`.

## search (solo Java)
- Fuera de Java: `La busqueda usa Modrinth y solo funciona en servidores de Java. Para PocketMine busca en https://poggit.pmmp.io`
- Busca en la [[API Modrinth]] (15 resultados, filtrados por tipo, cargador y versión).
- Cada resultado: `%-28s %8d descargas  %s` (slug, descargas y descripción con los espacios normalizados, cortada a 60 caracteres). Sin resultados: `Sin resultados para '%s' en %s %s.`
- Final: `Instala con: omh plugin add <nombre de la primera columna>`

## add
Por este orden:
1. `http://` o `https://`: descarga directa a `<carpeta>/<nombre saneado>`, sin hash.
2. Archivo local existente: se copia conservando fechas (`copy2`).
3. Core Java: Modrinth con dependencias obligatorias (recursivo, sin repetir proyectos, verifica SHA-1). Mensajes: `No existe el proyecto '%s' en Modrinth. Prueba: omh plugin search %s`, `'%s' no tiene version para %s %s.` y `Instalando dependencia necesaria...`
4. PocketMine: la versión más alta en [[API Poggit]]. Si no existe: `No existe el plugin '%s' en Poggit.`
5. Nukkit: `Para %s indica una URL directa o un archivo .jar local.`
Final: `[OK] Instalado en <carpeta>. Reinicia el servidor para cargarlo.`

## remove
Busca archivos de la carpeta cuyo nombre contenga el texto (sin distinguir mayúsculas). Si no hay ninguno: `No hay ningun archivo que coincida con '%s'.`; si hay varios: `Coinciden varios archivos: %s. Se mas especifico.`; si hay uno: `[OK] Eliminado %s. Reinicia el servidor para aplicarlo.`
