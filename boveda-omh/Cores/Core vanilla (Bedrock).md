---
tipo: core
tags: [omh/core]
aliases: [bedrock vanilla, BDS, Bedrock Dedicated Server]
fuente: "omh.py · install_bedrock_vanilla (L930), BEDROCK_JOIN_RE (L1496)"
rust: "src/cores/bedrock.rs"
---
# Core vanilla (Bedrock)

`Bedrock Dedicated Server oficial de Mojang.`

| | |
|---|---|
| Edición | bedrock |
| Plataformas | solo Windows y Linux x64 |
| Versión | `latest` (la de la [[API Minecraft Services]]) o una concreta, como `1.21.50.10` |
| Java | no |
| Complementos | ninguno |
| Puertos | 19132/UDP y el siguiente para IPv6 (`server-portv6`) → [[Puertos]] |

## Instalación
1. URL: si la versión es `latest` o coincide con la última publicada, el enlace de la API; si no, `https://www.minecraft.net/bedrockdedicatedserver/bin-<win|linux>/bedrock-server-<v>.zip` (antes comprueba la plataforma).
2. Descarga `.omh/bedrock-server.zip` (sin hash), lo extrae en la carpeta del servidor → [[Extraccion segura]] y borra el zip.
3. Da permiso de ejecución a `bedrock_server` (Linux).
4. `start = {"type": "native", "exe": "bedrock_server.exe" | "bedrock_server"}` → [[Comando de arranque]] (en Linux con `LD_LIBRARY_PATH=<carpeta>`).

## Particularidades
- No se puede limitar la RAM. [[omh resources]] escribe `max-threads` → [[Limite de nucleos]].
- Whitelist en `allowlist.json` y propiedades `allow-list`, `server-name`… → [[server.properties]], [[Listas de jugadores]].
- Administradores: el supervisor detecta `Player connected: <nombre>, xuid` y envía `op "<nombre>"` una vez por sesión a los que están en `network.admins` → [[Supervisor (Runner)]].
- El comando de whitelist en consola es `allowlist add|remove|on|off`.
- `bind_ip` no se aplica: escucha en todas las interfaces.
- Windows: para entrar desde el mismo PC hace falta una excepción de loopback → [[Problemas frecuentes]]. Linux: Mojang solo da soporte a Ubuntu (puede faltar `libcurl`).
- Reinstalar extrae el zip encima: sobrescribe `server.properties`, `allowlist.json` y `permissions.json`.
