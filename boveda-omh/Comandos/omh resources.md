---
tipo: comando
tags: [omh/comando, omh/bug]
aliases: [resources, PASO 2]
fuente: "omh.py · cmd_resources (L1150)"
rust: "src/resources.rs"
---
# omh resources (PASO 2)

Asigna RAM, almacenamiento máximo y núcleos. Requiere el paso 1.

## Opciones
- `--ram`, `-r`: `RAM, por ejemplo 4G o 4096M`
- `--storage`, `-s`: `almacenamiento maximo, por ejemplo 20G`
- `--cores`, `-c` (entero): `numero de nucleos de CPU`
- `--dir`

## Qué hace
1. `== PASO 2 de 3 - Recursos (<nombre>) ==`
2. Mide el equipo → [[Rutas y plataforma]] y muestra `Este equipo: RAM <X> | <N> nucleos | disco libre <X> | servidor ocupa <X>` (RAM `desconocida` si no se pudo leer).
3. **RAM**. Pregunta `RAM para el servidor (ej. 4G)`. Por defecto sugiere la mitad de la RAM redondeada a 512 MB, entre 1 GB y 4 GB (2 GB si no se conoce), escrita como `4GB`, `1.5GB`… Comprobaciones:
   - `< 512` MB: `La RAM minima es 512M.`
   - `> total - 512`: `No puedes asignar X: el equipo tiene Y y necesita al menos 512 MB para el sistema.`
   - `> 80 %` del total: `[AVISO] Asignas mas del 80% de la RAM del equipo. El sistema puede ir lento.`
4. **Almacenamiento**. Pregunta `Almacenamiento maximo (ej. 20G)` (por defecto `10G`).
   - `< ocupado + 256 MB`: `El limite de almacenamiento debe ser mayor que lo que ya ocupa el servidor (X).`
   - `límite - ocupado > libre`: `No hay espacio suficiente en disco: pides X y hay Y libres.`
5. **Núcleos**. Pregunta `Nucleos de CPU (1-N)` (por defecto `min(2, N)`). Fuera de rango: `Los nucleos deben estar entre 1 y N.`
6. Guarda `resources = {ram_mb, storage_mb, cores}` y `step = max(step, 2)` en [[omh.json]].
7. Bedrock vanilla: escribe `max-threads=<núcleos>` en [[server.properties]] y avisa `Bedrock vanilla no permite limitar la RAM. Se limitan los nucleos y el disco.`. PocketMine: `PocketMine (PHP) no permite fijar la RAM desde fuera. Se limitan los nucleos y el disco.`
8. `[OK] Recursos: RAM X | almacenamiento Y | N nucleos` y el siguiente paso.

Los tamaños se leen con `parse_size_mb` y se muestran con `fmt_mb` → [[Tamanos y versiones]]. Cómo se aplican: [[Limite de nucleos]], [[Limite de almacenamiento]], [[Comando de arranque]] (`-Xms`/`-Xmx`).

> [!bug] Heredado de Python (decidido)
> - `--cores 0` no da error: el 0 cuenta como «no indicado» → [[Bugs y ambiguedades#B2]].
> - Escribir algo que no es un número en la pregunta de núcleos acaba con una traza de Python (en Rust será un `[ERROR]` claro).
