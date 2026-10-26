---
tipo: core
tags: [omh/core]
aliases: [forge, MinecraftForge]
fuente: "omh.py · install_forge (L885), run_installer (L858), modloader_start (L872)"
rust: "src/cores/forge.rs, src/cores/mod.rs"
---
# Core forge

`Cargador de mods clasico (1.8.9, 1.12.2, 1.16.5, 1.20.1...).`

| | |
|---|---|
| Edición | java |
| Versión mínima | 1.7.10 |
| `latest` | la versión de Minecraft más alta de `promotions_slim.json` → [[API Forge]] |
| Complementos | mods; Modrinth con `forge` |

## Instalación
1. Build de Forge: la `recommended` de la versión o, si no hay, la `latest`. Si no hay ninguna: `Forge no tiene builds para la version %s.` Muestra `Forge <mc>-<forge>`.
2. Descarga el instalador a `.omh/forge-installer.jar`, probando dos formatos de nombre (un 404 pasa al siguiente).
3. Versiones anteriores a 1.17: descarga también el servidor vanilla como `minecraft_server.<v>.jar` (con SHA-1), porque los instaladores antiguos apuntan a enlaces de Mojang que ya no existen.
4. **Instalador** (compartido con [[Core neoforge]]):
   - asegura Java → [[Java Temurin]];
   - `Ejecutando el instalador (puede tardar unos minutos)...`;
   - `java -jar <instalador> --installServer` con la carpeta del servidor como directorio de trabajo y la salida en `.omh/installer.log`;
   - si el código no es 0: `El instalador fallo (codigo %d). Revisa %s`;
   - borra el instalador, `<instalador>.log` y cualquier `*installer*.log` de la carpeta.
5. **Cómo arrancar** (`modloader_start`, grupo `net/minecraftforge/forge`):
   - si existe `libraries/net/minecraftforge/forge/*/*_args.txt` → `{"type": "argsfile", "dir": "libraries/net/minecraftforge/forge/<versión>"}` (ruta relativa con el separador del sistema; Forge moderno);
   - si no, el `forge-*.jar` que no sea el instalador, prefiriendo los que contienen `universal` y, entre ellos, el de nombre más corto → `{"type": "jar", ...}` (Forge antiguo);
   - si no hay ninguno: `No se encontro el jar del servidor tras instalar.`

La instalación tarda unos minutos. Si falla, hay que mirar `.omh/installer.log` → [[Problemas frecuentes]].
