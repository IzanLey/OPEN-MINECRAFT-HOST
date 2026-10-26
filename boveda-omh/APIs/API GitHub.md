---
tipo: api
tags: [omh/api]
aliases: [GitHub Releases, api.github.com]
fuente: "omh.py · github_latest_release (L466), install_pocketmine (L946), install_nukkit (L1011), ensure_playit (L1289)"
rust: "src/http.rs"
---
# API GitHub (releases)

| Petición | Para qué | Cabecera `Accept` |
|---|---|---|
| `GET /repos/pmmp/PocketMine-MP/releases/latest` | `PocketMine-MP.phar` y `build_info.json` | `application/vnd.github+json` |
| `GET <build_info.json>` → `php_download_url` | *tag* de los binarios de PHP | — |
| `GET /repos/pmmp/PHP-Binaries/releases/tags/<tag>` | PHP recomendado | — |
| `GET /repos/pmmp/PHP-Binaries/releases?per_page=30` | respaldo | — |
| `GET /repos/PowerNukkitX/PowerNukkitX/releases/latest` | jar o zip | `application/vnd.github+json` |
| `GET /repos/playit-cloud/playit-agent/releases/tags/v0.17.1` | agente | `application/vnd.github+json` |
| (Rust) `GET /repos/IzanLey/OPEN-MINECRAFT-HOST/releases/latest` | `omh update` | `application/vnd.github+json` |

- Base: `https://api.github.com`. Sin autenticación: límite de 60 peticiones por hora y por IP (un 403 se muestra como `El servidor remoto respondio 403 (...)`).
- Se usan `assets[].name`, `assets[].browser_download_url`, `assets[].size`, `tag_name`, `draft` y `prerelease`.
- Ninguna de estas descargas verifica hash. Hoy GitHub publica `assets[].digest` (`sha256:…`), así que se podría añadir como mejora.

→ [[Core pocketmine]], [[Core nukkit]], [[Tunel playit.gg]], [[Distribucion del binario]]
