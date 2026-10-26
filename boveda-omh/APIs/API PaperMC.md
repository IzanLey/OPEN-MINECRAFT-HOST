---
tipo: api
tags: [omh/api]
aliases: [PaperMC, Fill, Fill v3]
fuente: "omh.py · paper_versions (L673), paper_download (L682)"
rust: "src/cores/paper.rs"
---
# API PaperMC

## Versiones
1. `GET https://fill.papermc.io/v3/projects/paper` → `versions` es un objeto `{grupo: [versiones]}` y se juntan todas.
2. Si falla por cualquier motivo, `GET https://api.papermc.io/v2/projects/paper` → `versions`.
Se quedan las versiones de tipo release (`\d+(\.\d+)+`), ordenadas con `vt` → [[Tamanos y versiones]].

## Descarga
1. `GET https://fill.papermc.io/v3/projects/paper/versions/<v>/builds/latest` → `downloads["server:default"].url` y `.checksums.sha256`.
2. Si falla: `GET https://api.papermc.io/v2/projects/paper/versions/<v>/builds` → la última build `{build, downloads.application.{name, sha256}}` → `…/versions/<v>/builds/<n>/downloads/<name>`.
3. Si fallan las dos: `Paper no tiene builds para la version %s.`

Verificación: **SHA-256** → [[Descargas verificadas]]. Lo usa [[Core paper]].
