---
tipo: concepto
tags: [omh/concepto]
aliases: [resolve_dir, servidor actual, --dir]
fuente: "omh.py · resolve_dir (L551)"
rust: "src/config.rs"
---
# Servidor activo

Cómo decide omh a qué servidor se refiere un comando (`resolve_dir`):

1. Si hay `--dir`, usa `abspath(--dir)`. Ojo: aquí **no** se expande `~`, al contrario que en [[omh create]].
2. Si la carpeta actual tiene [[omh.json]], usa la carpeta actual.
3. Si `current` de [[state.json]] apunta a una carpeta con omh.json, usa esa.
4. Si no: `No se encontro ningun servidor. Crea uno con 'omh create' o indica --dir.`

Cambian el activo: [[omh create]] (el recién creado) y [[omh use]]. Se ve en [[omh servers]] (marcado con `*`).
