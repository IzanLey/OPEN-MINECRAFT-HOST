---
tipo: mapa
tags: [omh/mapa]
aliases: [Cores, Servidores disponibles]
fuente: "omh.py · CORES (L614), INSTALLERS (L1034)"
---
# Mapa de cores

| Edición | Core | Descripción (texto exacto) | Desde | Complementos | Java | Arranque |
|---|---|---|---|---|---|---|
| java | [[Core vanilla (Java)\|vanilla]] | `Servidor oficial de Mojang. Sin plugins ni mods.` | 1.7.10 | — | según Mojang | `jar` `server.jar` |
| java | [[Core paper\|paper]] | `Alto rendimiento. Plugins Bukkit/Spigot/Paper.` | 1.8.8 | plugins | según Mojang | `jar` `server.jar` |
| java | [[Core purpur\|purpur]] | `Fork de Paper con mas opciones. Plugins Bukkit/Spigot/Paper.` | 1.14.1 | plugins | según Mojang | `jar` `server.jar` |
| java | [[Core fabric\|fabric]] | `Cargador de mods ligero.` | 1.14 | mods | según Mojang | `jar` `fabric-server-launch.jar` |
| java | [[Core forge\|forge]] | `Cargador de mods clasico (1.8.9, 1.12.2, 1.16.5, 1.20.1...).` | 1.7.10 | mods | según Mojang | `argsfile` o `jar` |
| java | [[Core neoforge\|neoforge]] | `Fork moderno de Forge (1.20.2 en adelante).` | 1.20.2 | mods | según Mojang | `argsfile` |
| bedrock | [[Core vanilla (Bedrock)\|vanilla]] | `Bedrock Dedicated Server oficial de Mojang.` | — | — | no | `native` |
| bedrock | [[Core pocketmine\|pocketmine]] | `PocketMine-MP (PHP). Plugins de Poggit.` | — | plugins | no (PHP) | `php` |
| bedrock | [[Core nukkit\|nukkit]] | `PowerNukkitX, fork mantenido de Nukkit (Java). Plugins .jar.` | — | plugins | 21 | `jar` |

- Orden de la tabla = orden en que se muestran y se ofrecen como opciones.
- `powernukkitx` se acepta como sinónimo de `nukkit` en `--core`.
- En Bedrock solo `vanilla` admite una versión concreta; los demás usan la última release → [[Catalogo de cores y versiones]].
- Los tipos de arranque están en [[Comando de arranque]] y se guardan en `start` de [[omh.json]].
- Cargadores para Modrinth: paper → `paper, spigot, bukkit, purpur`; purpur → `purpur, paper, spigot, bukkit`; fabric, forge y neoforge → el suyo → [[Plugins y mods]].

← [[OPEN-MC-HOST]]
