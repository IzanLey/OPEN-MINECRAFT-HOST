---
tipo: concepto
tags: [omh/concepto]
aliases: [resolve_version, JAVA_COMMON, CORES, latest]
fuente: "omh.py · JAVA_COMMON (L608), CORES (L614), DEFAULT_PORT (L647), resolve_version (L743)"
rust: "src/catalog.rs"
---
# Catálogo de cores y versiones

## Versiones comunes de Java (`JAVA_COMMON`)
`1.8.9, 1.12.2, 1.16.5, 1.17.1, 1.18.2, 1.19.4, 1.20.1, 1.20.4, 1.20.6, 1.21.1, 1.21.4, 1.21.8, 1.21.11, 26.1.2, 26.2, 26.3`

Solo sirven para informar: una versión fuera de la lista da un aviso, pero se intenta igualmente.

## `resolve_version(edición, core, versión)`
1. `pedida = (versión o "latest").strip().lower()`
2. **Bedrock**
   - core distinto de vanilla: si no es `latest`, `[AVISO] <core> solo se instala en su ultima version publicada.`; devuelve `latest`.
   - vanilla: `latest` pasa a la última versión publicada → [[API Minecraft Services]]; cualquier otra se usa tal cual, sin validar (si no existe, la descarga da error HTTP).
3. **Java**
   - `latest`: paper → la última de PaperMC; purpur → la última de Purpur; forge → la versión más alta de las promociones; el resto (vanilla, fabric, neoforge) → `latest.release` de Mojang. Muestra `Ultima version disponible para <core>: <v>`.
   - Si no es una release (`\d+(\.\d+)+`): `Version no valida: '<lo escrito>'. Ejemplo: 1.20.4`.
   - paper 1.8.x distinta de 1.8.8 → 1.8.8 con aviso → [[Core paper]].
   - Por debajo del mínimo del core: `<core> necesita la version <min> o superior.`
   - Fuera de `JAVA_COMMON`: `[AVISO] La version %s no esta en la lista de versiones comunes. Se intentara igualmente.`

## Otros datos del catálogo
- Cores, descripciones, mínimos y tipo de complemento → [[Mapa de cores]].
- Puertos por defecto: java 25565 (TCP) y bedrock 19132 (UDP) → [[Puertos]].
- Cargadores de Modrinth → [[API Modrinth]].
- Comparación de versiones con `vt` → [[Tamanos y versiones]].

> [!question] Ambigüedad
> `latest` no significa lo mismo en todos los cores (el último del core o el último de Mojang) → [[Bugs y ambiguedades#B11]].
