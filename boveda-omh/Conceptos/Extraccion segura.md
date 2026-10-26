---
tipo: concepto
tags: [omh/concepto]
aliases: [safe_extract, path traversal, zip slip]
fuente: "omh.py · _inside (L433), safe_extract (L439)"
rust: "src/extract.rs"
---
# Extracción segura

`safe_extract(archivo, destino)` extrae zip o tar sin que nada salga de `destino`.

## Regla `_inside(base, ruta)`
`abspath(ruta) == abspath(base)` o empieza por `abspath(base) + separador`. Es una comprobación **léxica**: normaliza `..` y `.` sin resolver enlaces simbólicos.

## zip (se detecta por su estructura, no por la extensión)
- Antes de extraer nada, comprueba cada entrada con `join(destino, nombre)`. Si alguna sale (por `../`, una ruta absoluta o, en Windows, `..\` o una letra de unidad): `Archivo comprimido inseguro: <nombre>`.
- `zipfile` **no** aplica los permisos Unix guardados en el zip: por eso omh hace `chmod +x` a mano en `bedrock_server`.

## tar (`.tar.gz`; Python detecta también otras compresiones)
- Cada miembro tiene que quedar dentro: si no, `Archivo comprimido inseguro: <nombre>`.
- Enlaces simbólicos: el destino, relativo a la carpeta del miembro, tiene que quedar dentro. Enlaces duros: relativo a `destino`. Si no: `Enlace inseguro en el archivo: <nombre>`.
- Con Python ≥ 3.12 (o parcheado) se extrae además con `filter="data"`: rechaza dispositivos y FIFOs, quita setuid/setgid/sticky y la escritura de grupo y otros, y no restaura el dueño. Sí conserva el bit de ejecución.

## Dónde se usa
[[Java Temurin]] (zip en Windows, tar.gz en Linux), [[Core vanilla (Bedrock)]] (zip), [[Core pocketmine]] (PHP en zip o tar.gz), [[Core nukkit]] (zip de respaldo).

→ [[Seguridad]]
