---
tipo: comando
tags: [omh/comando]
aliases: [admin, op]
fuente: "omh.py · cmd_admin (L2082)"
rust: "src/players.rs"
---
# omh admin

`gestionar los administradores (op)`. Requiere el paso 1.

`omh admin {add,remove,list} [jugador…]`

- `list`: `Administradores: …`. Es la unión de lo que hay en el archivo de operadores y `network.admins` de [[omh.json]], ordenada sin distinguir mayúsculas.
- `add` / `remove`: sin jugadores, `Indica el jugador. Ejemplo: omh admin add Steve`.
- Cada jugador se añade o quita de `network.admins` en [[omh.json]] (aunque aún no se haya hecho `omh network`), y después del archivo o la consola → [[Whitelist y administradores]].
- Bedrock vanilla: con el servidor apagado no se puede escribir el permiso (hace falta el XUID); el supervisor le da `op` cuando entra → [[Supervisor (Runner)]].

> [!warning]
> Usarlo antes de `omh network` deja `"network": {"admins": [...]}` en omh.json, lo que provoca [[Bugs y ambiguedades#B1]].
