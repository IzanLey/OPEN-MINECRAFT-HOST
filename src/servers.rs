//! `omh servers` y `omh use`: la lista de servidores y el servidor activo.

use std::path::{Path, PathBuf};

use crate::config::{self, CONFIG_FILE};
use crate::runner::control;
use crate::ui;
use crate::util::abspath;

pub fn cmd_servers() -> anyhow::Result<()> {
    let state = config::load_state();
    let mut servers = state.servers();
    if servers.is_empty() {
        ui::info("No hay servidores. Crea uno con: omh create");
        return Ok(());
    }
    // Como Python: sin servidor activo se compara con la carpeta actual.
    let current = abspath(Path::new(state.current.as_deref().unwrap_or("")));
    servers.sort();
    for (name, dir) in servers {
        let path = Path::new(&dir);
        let mark = if abspath(path) == current { "*" } else { " " };
        let status = if !path.join(CONFIG_FILE).exists() {
            "no encontrado"
        } else if control::is_running(path) {
            "EN EJECUCION"
        } else {
            "detenido"
        };
        ui::info(&format!("{mark} {name:<20} {status:<13} {dir}"));
    }
    ui::info("* = servidor activo. Cambia con: omh use <nombre o carpeta>");
    Ok(())
}

pub fn cmd_use(target: &str) -> anyhow::Result<()> {
    let state = config::load_state();
    let dir = state
        .servers
        .get(target)
        .and_then(|dir| dir.as_str())
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| abspath(Path::new(target)));
    let cfg = config::load_config(&dir)?;
    config::register_server(&cfg)?;
    ui::ok(&format!(
        "Servidor activo: {} ({})",
        cfg.name,
        cfg.dir.display()
    ));
    Ok(())
}
