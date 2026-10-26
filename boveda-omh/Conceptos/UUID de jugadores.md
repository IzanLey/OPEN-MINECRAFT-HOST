---
tipo: concepto
tags: [omh/concepto, omh/bug]
aliases: [player_uuid, UUID offline, OfflinePlayer]
fuente: "omh.py · player_uuid (L1946)"
rust: "src/players.rs"
---
# UUID de jugadores

Solo para servidores **Java** con el servidor parado (con el servidor en marcha lo resuelve el propio servidor).

- **online-mode activo**: `GET https://api.mojang.com/users/profiles/minecraft/<nombre>` → `id` → [[API Mojang]]. Si algo falla: `No existe una cuenta de Minecraft llamada '%s'.`
- **online-mode desactivado** (UUID «offline», igual que el servidor vanilla): `MD5("OfflinePlayer:" + nombre)` en UTF-8, con `byte[6] = (byte[6] & 0x0F) | 0x30` (versión 3) y `byte[8] = (byte[8] & 0x3F) | 0x80` (variante).
- Formato: `8-4-4-4-12` caracteres hexadecimales en minúsculas.

Ejemplo de prueba: `OfflinePlayer:Notch` → `b50ad385-829d-3141-a216-7e7d7539ba7f`.

> [!bug] Heredado de Python (decidido)
> Sin internet, el error dice que la cuenta no existe → [[Bugs y ambiguedades#B7]].
