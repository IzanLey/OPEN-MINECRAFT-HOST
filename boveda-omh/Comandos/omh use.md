---
tipo: comando
tags: [omh/comando]
aliases: [use]
fuente: "omh.py · cmd_use (L2238)"
rust: "src/servers.rs"
---
# omh use

`elegir el servidor activo`

- `objetivo`: `nombre o carpeta del servidor`.
- Si es un nombre registrado en [[state.json]], usa su carpeta; si no, lo trata como carpeta (`abspath`).
- Carga su [[omh.json]] (si no hay: `No hay ningun servidor en <carpeta>. Crea uno con: omh create`), lo vuelve a registrar con el nombre de su omh.json y lo marca como activo.
- `[OK] Servidor activo: <nombre> (<carpeta>)`

→ [[Servidor activo]]
