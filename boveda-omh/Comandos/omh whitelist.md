---
tipo: comando
tags: [omh/comando, omh/bug]
aliases: [whitelist]
fuente: "omh.py · cmd_whitelist (L2060)"
rust: "src/players.rs"
---
# omh whitelist

`gestionar la whitelist`. Requiere el paso 1.

`omh whitelist {add,remove,list,on,off} [jugador…]`

- `list`: `Whitelist: a, b` o `Whitelist: (vacia)` → [[Listas de jugadores]].
- `on` / `off`:
  - sin red configurada: `Configura antes la red con: omh network`;
  - guarda `network.whitelist`, reescribe [[server.properties]] y, si está en marcha, envía `whitelist on|off` (`allowlist on|off` en Bedrock vanilla);
  - `[OK] Whitelist on.`
- `add` / `remove`: sin jugadores, `Indica el jugador. Ejemplo: omh whitelist add Steve`. Con jugadores, cada uno se procesa por separado → [[Whitelist y administradores]].

Funciona con el servidor encendido (envía el comando a la consola) o apagado (edita los archivos).

> [!bug] Heredado de Python (decidido)
> Con una red a medias en [[omh.json]], `on`/`off` revienta en Python → [[Bugs y ambiguedades#B1]].
