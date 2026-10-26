---
tipo: concepto
tags: [omh/concepto]
aliases: [parse_size_mb, fmt_mb, vt, is_release]
fuente: "omh.py · vt (L273), is_release (L279), parse_size_mb (L283), fmt_mb (L294)"
rust: "src/util.rs"
---
# Tamaños y versiones

## `parse_size_mb(texto)` → megabytes (entero)
Expresión regular sin distinguir mayúsculas, que tiene que cubrir el texto entero: `\s*(\d+(?:\.\d+)?)\s*([KMGT]?)(?:I?B)?\s*`
- Sin unidad se entiende **GB**. Factores: K = 1/1024, M = 1, G = 1024, T = 1048576. El resultado se trunca a entero.
- Ejemplos: `4G` → 4096, `4096M` → 4096, `4GB` → 4096, `4` → 4096, `1.5G` → 1536, `2GiB` → 2048, `512K` → 0.
- Si no encaja: `Tamano no valido: '<texto>'. Usa por ejemplo 4G o 4096M.`

## `fmt_mb(mb)`
- `>= 1024`: `%.1f GB` (`4096` → `4.0 GB`).
- Si no: `%d MB` (`312 MB`).

## `vt(versión)` → tupla para comparar
Todos los grupos de dígitos como enteros: `1.20.4` → `(1, 20, 4)`; `1.21.1-rc1` → `(1, 21, 1, 1)`; sin dígitos → `(0,)`. Se compara elemento a elemento y la tupla más corta va antes si coinciden (`(1, 20) < (1, 20, 0)`).

## `is_release(versión)`
`\d+(\.\d+)+` sobre el texto entero: al menos dos números separados por puntos (`1.21`, `26.1.2`). `1.21-pre1` no lo es.

Lo usan [[omh resources]], [[Catalogo de cores y versiones]], [[Core neoforge]], [[API Poggit]] y [[API PaperMC]].
