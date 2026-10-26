---
tipo: migracion
tags: [omh/migracion]
aliases: [modulos, plan de modulos, arquitectura]
---
# Arquitectura de módulos

Un solo crate (`open-mc-host`, binario `omh`, edición 2024). Cada comando vive en el módulo de su dominio; `main.rs` solo analiza los argumentos, despacha y convierte errores en `[ERROR]` y códigos de salida.

```text
src/
├── main.rs          entrada: clap → comando; errores → "  [ERROR] …" y códigos 0/1/2/130
├── cli.rs           definición clap (derive) de los 19 subcomandos y sus textos de ayuda
├── ui.rs            título ASCII, guía, info/ok/warn/header, ask/confirm, manejador de Ctrl+C
├── error.rs         errores tipados (thiserror): fallo de omh, HTTP, red
├── util.rs          vt/is_release, parse_size_mb/fmt_mb, valid_player, splitlines y strip de Python,
│                    abspath, recorrido de carpetas, dir_size_mb, safe_filename, quote, tail
├── platform.rs      RAM total, CPUs lógicas, disco libre, chmod +x, afinidad, flags de proceso,
│                    setsid, kill, carpeta de usuario (libc / windows-sys)
├── pyjson.rs        serializador idéntico a json.dump de Python (indent=2 y compacto, ensure_ascii)
├── config.rs        omh.json (ServerConfig), state.json (State), resolve_dir, require_step,
│                    other_ports, set_properties (server.properties)
├── http.rs          cliente reqwest (rustls), get_json + caché, download (progreso, .part, sha1/sha256),
│                    releases de GitHub
├── extract.rs       zip y tar(.gz) seguros
├── java.rs          java_needed, ensure_java (Adoptium)
├── catalog.rs       CORES, JAVA_COMMON, puertos por defecto, resolve_version, omh list
├── cores/           mod.rs (despacho, EULA, run_installer, modloader_start) + vanilla, paper, purpur,
│                    fabric, forge, neoforge, bedrock, pocketmine, nukkit
├── create.rs        omh create, omh eula
├── resources.rs     omh resources
├── network/         mod.rs (omh network, IP LAN/pública, puertos, direcciones, propiedades de red),
│                    upnp.rs, tunnel.rs (playit + omh tunnel)
├── players.rs       whitelist/admin (UUID online/offline, listas JSON y txt)
├── plugins.rs       omh plugin (Modrinth, Poggit, URL, archivo)
├── runner/          mod.rs (supervisor), control.rs (run.json + socket con token, cliente y servidor),
│                    commands.rs (start, stop, restart, status, console, logs, cmd)
├── servers.rs       omh servers, omh use
└── update.rs        omh update (binario de la última release)
```

## De Python a Rust
| `omh.py` | Rust |
|---|---|
| `main`, `build_parser` | `main.rs`, `cli.rs` |
| `FONT`, `render_word`, `print_title`, `print_guide`, `info/ok/warn/header`, `ask`, `confirm` | `ui.rs` |
| `OmhError`, `fail`, `except` de `main` | `error.rs`, `main.rs` |
| `vt`, `is_release`, `parse_size_mb`, `fmt_mb`, `valid_player`, `tail_lines`, `safe_filename`, `dir_size_mb` | `util.rs` |
| `total_ram_mb`, `platform_ids`, `make_executable`, `kill_pid`, afinidad | `platform.rs` |
| `load/save_state`, `register_server`, `load/save_config`, `resolve_dir`, `require_step`, `other_ports`, `set_properties` | `config.rs` (+ `pyjson.rs`) |
| `http_open`, `get_json`, `cached_json`, `download`, `github_latest_release` | `http.rs` |
| `_inside`, `safe_extract` | `extract.rs` |
| `java_needed`, `find_java_exe`, `ensure_java` | `java.rs` |
| `JAVA_COMMON`, `CORES`, `DEFAULT_PORT`, `MODRINTH_LOADERS`, `mojang_*`, `resolve_version`, `cmd_list` | `catalog.rs` |
| `install_*`, `INSTALLERS`, `run_installer`, `modloader_start`, `write_eula`, `eula_accepted`, `paper_*`, `purpur_*`, `forge_promos`, `bedrock_*` | `cores/*` |
| `cmd_create`, `cmd_eula` | `create.rs` |
| `cmd_resources` | `resources.rs` |
| `lan_ip`, `public_ip`, `port_free`, `find_free_port`, `on_off`, `apply_network_properties`, `server_addresses`, `cmd_network` | `network/mod.rs` |
| `upnp_*` | `network/upnp.rs` |
| `PLAYIT_VERSION`, `playit_path`, `playit_command`, `ensure_playit`, `cmd_tunnel` | `network/tunnel.rs` |
| `Runner`, `build_command`, `READY_RE`, `BEDROCK_JOIN_RE` | `runner/mod.rs` |
| `run_file`, `control_request`, `is_running`, `send_command`, `handle_client` | `runner/control.rs` |
| `cmd_start/stop/restart/status/console/logs/cmd` | `runner/commands.rs` |
| `player_uuid`, `read/write_json_list`, `read/write_txt_list`, `list_files`, `console_name`, `add_player`, `show_list`, `cmd_whitelist`, `cmd_admin` | `players.rs` |
| `addons_dir`, `modrinth_install`, `cmd_plugin` | `plugins.rs` |
| `cmd_servers`, `cmd_use` | `servers.rs` |
| `cmd_update` | `update.rs` |

El módulo del túnel se llama `tunnel.rs` y no `playit.rs` porque el `.gitignore` ignora `playit*` → [[Decisiones tecnicas#D13]].
