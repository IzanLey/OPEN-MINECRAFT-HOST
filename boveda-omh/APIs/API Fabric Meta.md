---
tipo: api
tags: [omh/api]
aliases: [Fabric Meta, meta.fabricmc.net]
fuente: "omh.py · install_fabric (L847)"
rust: "src/cores/fabric.rs"
---
# API Fabric Meta

1. `GET https://meta.fabricmc.net/v2/versions/game` → lista de `{version}`. Si la versión no está: `Fabric no soporta la version %s.`
2. `GET https://meta.fabricmc.net/v2/versions/loader` → la **primera** con `stable: true`.
3. `GET https://meta.fabricmc.net/v2/versions/installer` → la **primera** con `stable: true`.
4. Descarga `https://meta.fabricmc.net/v2/versions/loader/<mc>/<loader>/<installer>/server/jar` como `fabric-server-launch.jar`.

Sin hash publicado. Lo usa [[Core fabric]].
