---
tipo: concepto
tags: [omh/concepto]
aliases: [add_player, show_list, valid_player, op]
fuente: "omh.py · valid_player (L364), console_name (L1998), add_player (L2002), show_list (L2049)"
rust: "src/players.rs"
---
# Whitelist y administradores

## Nombres válidos (`valid_player`)
`[A-Za-z0-9_ ]{1,32}` sobre el nombre entero (con espacio, para los gamertags de Bedrock) y después sin espacios a los lados. Si no: `Nombre de jugador no valido: '%s'`. En consola, si tiene espacios se escribe entre comillas (`console_name`).

## `add_player(cfg, tipo, nombre, quitar=False)`
1. **Admin**: actualiza `network.admins` en [[omh.json]] (lo crea si no existe; sin repetir, sin distinguir mayúsculas) y guarda.
2. **Si el servidor está en marcha** → [[Canal de control]]:
   - whitelist: `whitelist add|remove <nombre>` (`allowlist …` en Bedrock vanilla);
   - admin: `op <nombre>` / `deop <nombre>`;
   - `[OK] Enviado al servidor: <comando>` y termina (no toca archivos).
3. **Si está parado** → [[Listas de jugadores]]:
   - Java: quita la entrada con ese nombre y, al añadir, consulta el UUID ([[UUID de jugadores]], según `online_mode`; por defecto activo). Añade `{"uuid", "name"}` y, si es admin, `"level": 4, "bypassesPlayerLimit": false`.
   - Bedrock vanilla: la whitelist usa `{"ignoresPlayerLimit": false, "name"}`. El admin no se puede escribir: `En Bedrock vanilla el rango de admin se aplica cuando <nombre> entre al servidor.` y el supervisor le da op al entrar → [[Supervisor (Runner)]].
   - PocketMine / Nukkit: `.txt` con el nombre en minúsculas.
4. `[OK] <nombre> anadido a|eliminado de la whitelist|los administradores.`

## Listar (`show_list`)
`Whitelist: …` o `Administradores: …`, separados por comas, o `(vacia)`. Los admins se juntan con `network.admins`.

→ [[omh whitelist]], [[omh admin]], [[omh network]] (añade los `--admin`)
