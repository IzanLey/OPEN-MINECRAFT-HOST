//! `omh create` (PASO 1: edicion, core y version) y `omh eula`.

use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use anyhow::Context;
use regex::Regex;
use serde_json::Map;

use crate::catalog::{self, JAVA_COMMON};
use crate::cli::{CreateArgs, EulaArgs};
use crate::config::{self, CONFIG_FILE, RUN_DIR, ServerConfig};
use crate::cores::{self, eula_accepted, write_eula};
use crate::error::fail;
use crate::java;
use crate::ui;
use crate::util::{abspath, expanduser, local_timestamp, py_basename};

static NAME_CHARS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^A-Za-z0-9_.-]").expect("expresion regular fija y probada"));

/// Valor de una opcion si se indico y no esta vacio (`args.x or ...` de Python).
fn given(value: &Option<String>) -> Option<&str> {
    value.as_deref().filter(|value| !value.is_empty())
}

/// Nombre del servidor: el indicado o el de la carpeta, con solo `[A-Za-z0-9_.-]`.
fn server_name(name: Option<&str>, directory: &Path) -> String {
    let raw = match name {
        Some(name) => name.to_owned(),
        None => {
            let text = directory.to_string_lossy();
            let base = py_basename(text.trim_end_matches(['\\', '/']));
            if base.is_empty() {
                "servidor".to_owned()
            } else {
                base.to_owned()
            }
        }
    };
    NAME_CHARS.replace_all(&raw, "-").into_owned()
}

pub fn cmd_create(args: &CreateArgs) -> anyhow::Result<()> {
    ui::print_title();
    ui::header("PASO 1 de 3 - Edicion, core y version");
    let edition = match given(&args.edition) {
        Some(edition) => edition.to_owned(),
        None => ui::ask("Edicion", Some("java"), &catalog::EDITIONS)?,
    }
    .to_lowercase();
    let cores = catalog::cores_of(&edition);
    if cores.is_empty() {
        return Err(fail(format!(
            "Edicion no valida: {edition} (usa java o bedrock)"
        )));
    }
    let names: Vec<&str> = cores.iter().map(|core| core.name).collect();
    if given(&args.core).is_none() {
        for core in &cores {
            ui::info(&format!("  {:<10} {}", core.name, core.desc));
        }
    }
    let default_core = if edition == "java" {
        "paper"
    } else {
        "vanilla"
    };
    let mut core = match given(&args.core) {
        Some(core) => core.to_owned(),
        None => ui::ask("Core", Some(default_core), &names)?,
    }
    .to_lowercase();
    if core == "powernukkitx" {
        core = "nukkit".to_owned();
    }
    if !names.contains(&core.as_str()) {
        return Err(fail(format!(
            "Core no valido para {edition}: {core}. Opciones: {}",
            names.join(", ")
        )));
    }
    let version = match given(&args.version) {
        Some(version) => version.to_owned(),
        None => {
            if edition == "java" {
                ui::info(&format!("Versiones comunes: {}", JAVA_COMMON.join(", ")));
            }
            ui::ask("Version (o latest)", Some("latest"), &[])?
        }
    };
    let version = catalog::resolve_version(&edition, &core, &version)?;

    let default_dir = Path::new(".").join(format!("mc-{core}-{version}"));
    let directory = match given(&args.dir) {
        Some(dir) => dir.to_owned(),
        None => ui::ask(
            "Carpeta del servidor",
            Some(&default_dir.to_string_lossy()),
            &[],
        )?,
    };
    let directory = abspath(&expanduser(&directory));
    if directory.join(CONFIG_FILE).exists() && !args.force {
        return Err(fail(format!(
            "Ya existe un servidor en {}. Usa --force para reinstalarlo.",
            directory.display()
        )));
    }
    let name = server_name(given(&args.name), &directory);
    let run_dir = directory.join(RUN_DIR);
    fs::create_dir_all(&run_dir)
        .with_context(|| format!("No se pudo crear {}", run_dir.display()))?;

    let java = java::java_needed(&edition, &core, &version);
    let mut cfg = ServerConfig {
        name,
        dir: directory,
        edition,
        core,
        version,
        java,
        step: 0,
        created: local_timestamp(),
        start: None,
        resources: None,
        network: None,
        extra: Map::new(),
    };
    ui::info(&format!(
        "Servidor: {} | {} {} {} | {}",
        cfg.name,
        cfg.edition,
        cfg.core,
        cfg.version,
        cfg.dir.display()
    ));
    if let Some(major) = cfg.java
        && !matches!(cfg.core.as_str(), "forge" | "neoforge")
    {
        java::ensure_java(major)?;
    }
    ui::header("Descargando el servidor");
    cfg.start = Some(cores::install(&mut cfg)?);

    if cfg.is_java() {
        if args.eula || eula_accepted(&cfg.dir) {
            write_eula(&cfg.dir)?;
        } else {
            ui::info("Para iniciar un servidor de Java debes aceptar el EULA de Minecraft:");
            ui::info("https://aka.ms/MinecraftEULA");
            if ui::confirm("Aceptas el EULA de Minecraft?", false) {
                write_eula(&cfg.dir)?;
            } else {
                ui::warn("EULA no aceptado. Podras aceptarlo despues con: omh eula --accept");
            }
        }
    } else if cfg.core == "vanilla" {
        ui::info("Al usar Bedrock Dedicated Server aceptas el EULA de Minecraft y la politica");
        ui::info("de privacidad de Microsoft: https://aka.ms/MinecraftEULA");
    }

    cfg.step = 1;
    config::save_config(&cfg)?;
    config::register_server(&cfg)?;
    ui::ok(&format!(
        "Servidor '{}' creado en {}",
        cfg.name,
        cfg.dir.display()
    ));
    print!(
        "\n  Siguiente paso:\n    PASO 2 de 3 - Recursos:  omh resources --ram 4G --storage 20G --cores 2\n\n"
    );
    Ok(())
}

pub fn cmd_eula(args: &EulaArgs) -> anyhow::Result<()> {
    let cfg = config::load_config(&config::resolve_dir(args.dir.as_deref())?)?;
    if !cfg.is_java() {
        ui::info("Solo los servidores de Java necesitan eula.txt.");
        return Ok(());
    }
    if !args.accept {
        ui::info("Lee el EULA en https://aka.ms/MinecraftEULA y acepta con: omh eula --accept");
        return Ok(());
    }
    write_eula(&cfg.dir)?;
    ui::ok("EULA aceptado.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nombre_del_servidor_como_python() {
        let dir = abspath(Path::new("mis servidores/Mi Servidor (1)"));
        assert_eq!(server_name(None, &dir), "Mi-Servidor--1-");
        assert_eq!(server_name(Some("caf\u{e9}.v2"), &dir), "caf-.v2");
        assert_eq!(server_name(None, Path::new("/")), "servidor");
    }
}
