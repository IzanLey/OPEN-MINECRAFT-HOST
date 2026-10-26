---
tipo: comando
tags: [omh/comando]
aliases: [list]
fuente: "omh.py · cmd_list (L2203)"
rust: "src/catalog.rs"
---
# omh list

`ver ediciones, cores y versiones disponibles`

1. Título ASCII → [[Mensajes y titulo]].
2. Por edición: `== Edicion java ==` y cada core como `%-10s %s%s` (nombre, descripción y ` (desde X)` si tiene mínimo) → [[Mapa de cores]].
3. `== Versiones ==`
   - `Java (comunes): 1.8.9, 1.12.2, …` → [[Catalogo de cores y versiones]]
   - `Java (ultima):  <última release de Mojang>`; si falla la red, se omite → [[API Mojang]]
   - `Bedrock:        latest = <versión> (o una version concreta, solo core vanilla)`; si falla, `Bedrock:        latest (o una version concreta, solo core vanilla)` → [[API Minecraft Services]]
   - `Usa 'latest' para la ultima version disponible de cada core.`
4. Línea en blanco.
