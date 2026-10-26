---
tipo: concepto
tags: [omh/concepto]
aliases: [storage, storage_watch, almacenamiento]
fuente: "omh.py · cmd_resources (L1170), Runner.preflight (L1602), Runner.storage_watch (L1705)"
rust: "src/runner/mod.rs"
---
# Límite de almacenamiento

`resources.storage_mb` es el tamaño máximo de **toda la carpeta del servidor** (mundo, registros, `.omh/`, librerías…). Java está en la carpeta global y no cuenta. No es una cuota del sistema de archivos: omh lo vigila.

| Momento | Comprobación |
|---|---|
| [[omh resources]] | límite ≥ ocupado + 256 MB y hueco suficiente en disco |
| [[omh start]] (preflight) | si ocupado ≥ límite: `El servidor ocupa X y su limite es Y. Amplialo con: omh resources` |
| Con el servidor en marcha | un hilo del [[Supervisor (Runner)]] mide cada **120 s** |

## Vigilancia en marcha (`storage_watch`)
- Uso ≥ 100 %: `[omh] Limite de almacenamiento alcanzado (X de Y). Deteniendo.`, envía `say Limite de almacenamiento alcanzado. El servidor se detiene.`, espera 2 s y envía `stop`.
- Uso ≥ 90 %: `[omh] Aviso: el servidor usa X de Y de almacenamiento.` (en cada medición).
- Termina cuando el servidor se para.

El tamaño se calcula con `dir_size_mb` → [[Rutas y plataforma]].
