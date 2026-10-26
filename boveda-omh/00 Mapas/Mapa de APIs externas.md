---
tipo: mapa
tags: [omh/mapa]
aliases: [APIs, APIs externas, Endpoints]
---
# Mapa de APIs externas

Todas las peticiones HTTP llevan `User-Agent: open-mc-host/<versión> (https://github.com/IzanLey/OPEN-MINECRAFT-HOST)`. Timeout por operación de socket: 30 s en JSON y 60 s en descargas → [[Descargas verificadas]].

| Servicio | Endpoint | Para qué | Hash | Nota |
|---|---|---|---|---|
| Mojang | `https://piston-meta.mojang.com/mc/game/version_manifest_v2.json` | Versiones y última release | — | [[API Mojang]] |
| Mojang | JSON de cada versión (`url` del manifiesto) | `downloads.server` y `javaVersion.majorVersion` | — | [[API Mojang]] |
| Mojang | `downloads.server.url` | `server.jar` | SHA-1 | [[Core vanilla (Java)]] |
| Mojang | `https://api.mojang.com/users/profiles/minecraft/<nombre>` | UUID de una cuenta | — | [[UUID de jugadores]] |
| Minecraft Services | `https://net-secondary.web.minecraft-services.net/api/v1.0/download/links` | Última versión de Bedrock Dedicated Server | — | [[API Minecraft Services]] |
| minecraft.net | `https://www.minecraft.net/bedrockdedicatedserver/bin-{win,linux}/bedrock-server-<v>.zip` | Una versión concreta de BDS | — | [[Core vanilla (Bedrock)]] |
| PaperMC Fill v3 | `https://fill.papermc.io/v3/projects/paper` y `…/versions/<v>/builds/latest` | Versiones y jar | SHA-256 | [[API PaperMC]] |
| PaperMC v2 | `https://api.papermc.io/v2/projects/paper[/versions/<v>/builds]` | Respaldo si falla Fill | SHA-256 | [[API PaperMC]] |
| Purpur | `https://api.purpurmc.org/v2/purpur[/<v>/latest/download]` | Versiones y jar | ninguno | [[API Purpur]] |
| Fabric Meta | `https://meta.fabricmc.net/v2/versions/{game,loader,installer}` y `…/loader/<mc>/<loader>/<installer>/server/jar` | Versiones y lanzador | ninguno | [[API Fabric Meta]] |
| Forge | `https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json` y `https://maven.minecraftforge.net/…` | Build recomendada e instalador | ninguno | [[API Forge]] |
| NeoForge | `https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge` y `…/releases/net/neoforged/neoforge/<v>/…-installer.jar` | Versiones e instalador | ninguno | [[API NeoForge]] |
| GitHub | `https://api.github.com/repos/<repo>/releases/{latest,tags/<tag>}` | PocketMine, PHP, PowerNukkitX, playit | ninguno | [[API GitHub]] |
| Adoptium | `https://api.adoptium.net/v3/assets/latest/<java>/hotspot?…` | Java Temurin | SHA-256 | [[API Adoptium]] |
| Modrinth | `https://api.modrinth.com/v2/project/<id>/version` y `/v2/search` | Plugins y mods | SHA-1 | [[API Modrinth]] |
| Poggit | `https://poggit.pmmp.io/releases.min.json?name=<nombre>` | Plugins de PocketMine | ninguno | [[API Poggit]] |
| ipify | `https://api.ipify.org` | IP pública | — | [[API ipify]] |
| UPnP (router) | SSDP `239.255.255.250:1900` + SOAP | Abrir el puerto | — | [[UPnP]] |
| playit.gg | Agente `playit` (release `v0.17.1`) y `https://playit.gg/claim/…` | Túnel público | ninguno | [[Tunel playit.gg]] |
| raw.githubusercontent.com | `…/IzanLey/OPEN-MINECRAFT-HOST/main/omh.py` | `omh update` (Python) | ninguno | [[omh update]] |

Las respuestas de tres URLs se guardan en memoria durante la ejecución (`cached_json`): el manifiesto de Mojang, los enlaces de BDS y las promociones de Forge.

← [[OPEN-MC-HOST]]
