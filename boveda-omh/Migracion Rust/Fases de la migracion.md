---
tipo: migracion
tags: [omh/migracion]
aliases: [fases, roadmap]
---
# Fases de la migración

Rama `rust-rewrite`, un commit por fase. Al terminar cada fase: `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` y `cargo test`, en **Windows** y en **Linux (WSL Ubuntu)**. Los comandos que aún no estén portados responden `[ERROR] Este comando aun no esta portado` hasta su fase.

- [x] **Fase 0**: rama `rust-rewrite` y esta bóveda.
- [x] **Fase 1: cli + ui + config/state**
  - `Cargo.toml`, `.gitignore` ([[Decisiones tecnicas#D13]]), `omh.py` → `legacy/omh.py`.
  - `cli.rs` con los 19 subcomandos y sus opciones; `main.rs` con códigos de salida; `ui.rs` (título, guía, mensajes, ask/confirm, Ctrl+C); `error.rs`; `util.rs`; `pyjson.rs`; `config.rs`; `platform.rs` (carpeta de usuario).
  - Funcionan: `omh`, `omh --version`, `omh --help`, `omh servers`, `omh use`.
  - Tests: tamaños, versiones, `server.properties`, JSON de Python, título idéntico, `splitlines`, `abspath`.
- [x] **Fase 2: create + list + eula**
  - `http.rs` (descargas con sha1/sha256), `extract.rs`, `java.rs`, `catalog.rs`, `cores/` (los 9).
  - Además `omh plugin`, porque usa las mismas piezas (HTTP, catálogo).
  - Tests: extracción segura (zip/tar con `../`, rutas absolutas, enlaces), `resolve_version` sin red, verificación de hash.
- [x] **Fase 3: resources**
  - RAM, CPUs y disco nativos en `platform.rs`; `resources.rs`.
  - Tests: sugerencia de RAM, validaciones.
- [x] **Fase 4: network + tunnel**
  - `network/` (puertos, IPs, direcciones, propiedades), `upnp.rs`, `tunnel.rs`, `players.rs` (`omh network` añade admins y whitelist) con `omh whitelist` y `omh admin`.
  - Tests: puertos libres (`need_next`, excluidos), UUID offline, archivos de jugadores, SOAP generado.
- [ ] **Fase 5: runner**
  - `runner/` (supervisor, canal de control, afinidad), `start`, `stop`, `restart`, `status`, `console`, `logs`, `cmd`.
  - Tests: protocolo de control (ping, cmd, token incorrecto), `run.json`, compatibilidad con un supervisor Python.
- [ ] **Entrega**
  - `omh update`, workflows de [[CI y releases]], instaladores ([[Distribucion del binario]]), README 0.1.0.
  - Prueba real ([[Plan de pruebas]]) y PR con resumen y pendientes.
