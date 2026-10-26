---
tipo: api
tags: [omh/api]
aliases: [Purpur, PurpurMC]
fuente: "omh.py · purpur_versions (L699), install_purpur (L839)"
rust: "src/cores/purpur.rs"
---
# API Purpur

- `GET https://api.purpurmc.org/v2/purpur` → `versions` (ordenadas con `vt`).
- Descarga: `https://api.purpurmc.org/v2/purpur/<v>/latest/download` → `server.jar`.
- Si la versión no está en la lista: `Purpur no tiene la version %s.`
- **Sin verificación de hash.** La API publica un MD5 en `/v2/purpur/<v>/latest`, pero omh no lo usa; sería una mejora, no una corrección.

Lo usa [[Core purpur]].
