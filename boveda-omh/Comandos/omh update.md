---
tipo: comando
tags: [omh/comando, omh/bug]
aliases: [update, actualizar]
fuente: "omh.py · cmd_update (L2246)"
rust: "src/update.rs"
---
# omh update

`actualizar open-mc-host desde GitHub`

## Python (0.0.1)
1. `Descargando la ultima version desde https://raw.githubusercontent.com/IzanLey/OPEN-MINECRAFT-HOST/main/omh.py`
2. Descarga `omh.py` de la rama `main` (sin hash) a `omh.py.new` y lo pone en lugar del actual.
3. `[OK] open-mc-host actualizado.`

## Rust (0.1.0)
Ya no hay `omh.py`: descarga el binario de la **última release** de GitHub, comprueba su SHA-256 y sustituye el ejecutable. En Windows renombra antes el `.exe` en uso a `.old`, porque el supervisor `omh start --daemon` lo mantiene abierto mientras el servidor está en marcha → [[Distribucion del binario]].

> [!bug] Heredado de Python (decidido)
> Cuando `omh.py` pase a `legacy/`, el `omh update` de quien tenga la versión Python dará 404 → [[Bugs y ambiguedades#B9]].
