//! Instalacion de cada core (un modulo por core) y piezas comunes: EULA, el
//! instalador de Forge/NeoForge y como arrancarlos despues.

pub mod bedrock;
pub mod fabric;
pub mod forge;
pub mod neoforge;
pub mod nukkit;
pub mod paper;
pub mod pocketmine;
pub mod purpur;
pub mod vanilla;

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;
use serde_json::Value;

use crate::config::{RUN_DIR, ServerConfig, StartSpec};
use crate::error::fail;
use crate::java::ensure_java;
use crate::platform::exit_code_text;
use crate::ui;
use crate::util::{glob_names, name_key, read_text_lossy, relpath, write_text};

/// Descarga e instala el servidor y devuelve como arrancarlo.
pub fn install(cfg: &mut ServerConfig) -> anyhow::Result<StartSpec> {
    match (cfg.edition.as_str(), cfg.core.as_str()) {
        ("java", "vanilla") => vanilla::install(cfg),
        ("java", "paper") => paper::install(cfg),
        ("java", "purpur") => purpur::install(cfg),
        ("java", "fabric") => fabric::install(cfg),
        ("java", "forge") => forge::install(cfg),
        ("java", "neoforge") => neoforge::install(cfg),
        ("bedrock", "vanilla") => bedrock::install(cfg),
        ("bedrock", "pocketmine") => pocketmine::install(cfg),
        ("bedrock", "nukkit") => nukkit::install(cfg),
        (edition, core) => Err(fail(format!("Core no valido para {edition}: {core}."))),
    }
}

const EULA_TEXT: &str = "# Aceptado con open-mc-host. https://aka.ms/MinecraftEULA\neula=true\n";

pub fn write_eula(dir: &Path) -> anyhow::Result<()> {
    write_text(&dir.join("eula.txt"), EULA_TEXT)
}

pub fn eula_accepted(dir: &Path) -> bool {
    read_text_lossy(&dir.join("eula.txt"))
        .is_ok_and(|text| text.replace(' ', "").to_lowercase().contains("eula=true"))
}

/// Nombre de un asset de una release de GitHub.
fn asset_name(asset: &Value) -> &str {
    asset.get("name").and_then(Value::as_str).unwrap_or("")
}

/// Ejecuta el instalador de Forge/NeoForge con la salida en `.omh/installer.log`.
pub fn run_installer(cfg: &ServerConfig, installer: &Path) -> anyhow::Result<()> {
    let major = cfg
        .java
        .ok_or_else(|| fail("Este servidor no tiene version de Java."))?;
    let java = ensure_java(major)?;
    ui::info("Ejecutando el instalador (puede tardar unos minutos)...");
    let log_path = cfg.dir.join(RUN_DIR).join("installer.log");
    let log = File::create(&log_path)
        .with_context(|| format!("No se pudo crear {}", log_path.display()))?;
    let status = Command::new(&java)
        .arg("-jar")
        .arg(installer)
        .arg("--installServer")
        .current_dir(&cfg.dir)
        .stdout(log.try_clone()?)
        .stderr(log)
        .status()
        .with_context(|| format!("No se pudo ejecutar {}", java.display()))?;
    if !status.success() {
        return Err(fail(format!(
            "El instalador fallo (codigo {}). Revisa {}",
            exit_code_text(&status),
            log_path.display()
        )));
    }
    let mut installer_log = installer.as_os_str().to_owned();
    installer_log.push(".log");
    let mut leftovers = vec![installer.to_path_buf(), PathBuf::from(installer_log)];
    leftovers.extend(glob_names(&cfg.dir, &|name| {
        let name = name_key(name);
        name.strip_suffix(".log")
            .is_some_and(|stem| stem.contains("installer"))
    }));
    for leftover in leftovers {
        if leftover.exists() {
            fs::remove_file(&leftover)
                .with_context(|| format!("No se pudo borrar {}", leftover.display()))?;
        }
    }
    Ok(())
}

/// Detecta como arrancar Forge/NeoForge tras la instalacion: el `*_args.txt`
/// de las versiones modernas o el jar `forge-*.jar` de las antiguas.
pub fn modloader_start(dir: &Path, group: &[&str]) -> anyhow::Result<StartSpec> {
    let group_dir = group
        .iter()
        .fold(dir.join("libraries"), |path, part| path.join(part));
    for version_dir in glob_names(&group_dir, &|_| true) {
        let hits = glob_names(&version_dir, &|name| name_key(name).ends_with("_args.txt"));
        if !hits.is_empty() {
            return Ok(StartSpec::Argsfile {
                dir: relpath(&version_dir, dir),
            });
        }
    }
    let mut jars: Vec<String> = glob_names(dir, &|name| {
        let name = name_key(name);
        name.starts_with("forge-") && name.ends_with(".jar")
    })
    .iter()
    .filter_map(|path| path.file_name().map(|n| n.to_string_lossy().into_owned()))
    .filter(|name| !name.contains("installer"))
    .collect();
    jars.sort_by_key(|name| (!name.contains("universal"), name.chars().count()));
    jars.into_iter()
        .next()
        .map(|jar| StartSpec::Jar { jar })
        .ok_or_else(|| fail("No se encontro el jar del servidor tras instalar."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eula_como_python() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!eula_accepted(tmp.path()));
        write_eula(tmp.path()).unwrap();
        assert!(eula_accepted(tmp.path()));
        let expected = crate::util::native_newlines(EULA_TEXT);
        assert_eq!(
            fs::read_to_string(tmp.path().join("eula.txt")).unwrap(),
            expected
        );
        fs::write(tmp.path().join("eula.txt"), "EULA = TRUE\r\n").unwrap();
        assert!(eula_accepted(tmp.path()));
        fs::write(tmp.path().join("eula.txt"), "eula=false\n").unwrap();
        assert!(!eula_accepted(tmp.path()));
    }

    #[test]
    fn arranque_de_forge_moderno_y_antiguo() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let args_dir = dir.join("libraries/net/minecraftforge/forge/1.20.1-47.2.0");
        fs::create_dir_all(&args_dir).unwrap();
        fs::write(args_dir.join("unix_args.txt"), "").unwrap();
        fs::write(args_dir.join("win_args.txt"), "").unwrap();
        let expected = [
            "libraries",
            "net",
            "minecraftforge",
            "forge",
            "1.20.1-47.2.0",
        ]
        .join(std::path::MAIN_SEPARATOR_STR);
        assert_eq!(
            modloader_start(dir, &["net", "minecraftforge", "forge"]).unwrap(),
            StartSpec::Argsfile { dir: expected }
        );

        let old = tempfile::tempdir().unwrap();
        for jar in [
            "forge-1.12.2-14.23.5.2859.jar",
            "forge-1.12.2-14.23.5.2859-universal.jar",
            "forge-1.12.2-installer.jar",
        ] {
            fs::write(old.path().join(jar), "").unwrap();
        }
        assert_eq!(
            modloader_start(old.path(), &["net", "minecraftforge", "forge"]).unwrap(),
            StartSpec::Jar {
                jar: "forge-1.12.2-14.23.5.2859-universal.jar".into()
            }
        );
        let empty = tempfile::tempdir().unwrap();
        assert!(modloader_start(empty.path(), &["net", "neoforged", "neoforge"]).is_err());
    }
}
