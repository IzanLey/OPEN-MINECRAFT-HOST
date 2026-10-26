---
tipo: api
tags: [omh/api]
aliases: [Mojang, piston-meta, version_manifest_v2]
fuente: "omh.py · MOJANG_MANIFEST (L649), mojang_latest/releases/server (L655-670), java_needed (L723), player_uuid (L1946)"
rust: "src/catalog.rs, src/cores/vanilla.rs, src/players.rs"
---
# API Mojang

## Manifiesto de versiones
`GET https://piston-meta.mojang.com/mc/game/version_manifest_v2.json` (se guarda en memoria durante la ejecución)
- `latest.release` → última versión ([[omh list]], `latest` en vanilla, fabric y neoforge).
- `versions[]` con `id`, `type` (`release`, `snapshot`…) y `url`.

## JSON de una versión
`GET <versions[i].url>` (sin caché)
- `downloads.server.url` y `downloads.server.sha1` → `server.jar`. Sin `server`: `Mojang no publica servidor para la version %s.`. Si la versión no está en el manifiesto: `La version %s no existe en Mojang.`
- `javaVersion.majorVersion` → Java necesario, con un mínimo de 8 → [[Java Temurin]].

Lo usan [[Core vanilla (Java)]], [[Core forge]] (jar vanilla para versiones anteriores a 1.17) y `java_needed` para todos los cores de Java.

## Perfiles
`GET https://api.mojang.com/users/profiles/minecraft/<nombre>` (nombre codificado en la URL)
- `id` (UUID sin guiones) → [[UUID de jugadores]] cuando `online_mode` está activo.
- Cualquier fallo, incluida la falta de red, se muestra como `No existe una cuenta de Minecraft llamada '%s'.` → [[Bugs y ambiguedades#B7]].
