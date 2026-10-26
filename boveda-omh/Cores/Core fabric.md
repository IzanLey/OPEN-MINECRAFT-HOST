---
tipo: core
tags: [omh/core]
aliases: [fabric]
fuente: "omh.py · install_fabric (L847)"
rust: "src/cores/fabric.rs"
---
# Core fabric

`Cargador de mods ligero.`

| | |
|---|---|
| Edición | java |
| Versión mínima | 1.14 |
| `latest` | `latest.release` de Mojang (no la de Fabric) → [[Bugs y ambiguedades#B11]] |
| Complementos | mods en `mods/`; Modrinth con `fabric` |

## Instalación → [[API Fabric Meta]]
1. Comprueba que Fabric soporte la versión: si no, `Fabric no soporta la version %s.`
2. Coge el primer *loader* estable y el primer *installer* estable.
3. Descarga el lanzador como `fabric-server-launch.jar` (etiqueta `fabric <v>`), sin hash.
4. `start = {"type": "jar", "jar": "fabric-server-launch.jar"}`.
