---
tipo: core
tags: [omh/core]
aliases: [neoforge]
fuente: "omh.py · install_neoforge (L911)"
rust: "src/cores/neoforge.rs"
---
# Core neoforge

`Fork moderno de Forge (1.20.2 en adelante).`

| | |
|---|---|
| Edición | java |
| Versión mínima | 1.20.2 |
| `latest` | `latest.release` de Mojang, aunque NeoForge aún no la tenga → [[Bugs y ambiguedades#B11]] |
| Complementos | mods; Modrinth con `neoforge` |

## Elegir la versión de NeoForge → [[API NeoForge]]
- Minecraft `1.X.Y` → prefijo `X.Y.` (1.21.1 → `21.1.`; 1.20.2 → `20.2.`; 1.21 → `21.0.`).
- Minecraft con numeración por año (`26.1`, `26.1.2`) → prefijo `A.B.C.` (26.1 → `26.1.0.`).
- Se quitan las que contienen `beta` o `alpha` (si no queda ninguna, se usan todas) y se coge la mayor según `vt`.
- Si no hay ninguna: `NeoForge no tiene builds para la version %s.` Muestra `NeoForge <v>`.

## Instalación
Descarga `.omh/neoforge-installer.jar` (sin hash), ejecuta el mismo instalador que [[Core forge]] y arranca con `modloader_start` sobre el grupo `net/neoforged/neoforge` (normalmente `argsfile`).
