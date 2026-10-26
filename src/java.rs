//! Java (Eclipse Temurin desde Adoptium): que version necesita cada servidor y
//! descarga en `~/.open-mc-host/java/<version>`.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde_json::Value;

use crate::catalog;
use crate::error::{fail, http_status};
use crate::extract::safe_extract;
use crate::http::{self, Checksums};
use crate::platform::{self, IS_WIN};
use crate::ui;
use crate::util::{py_basename, vt, walk_find};

/// Version de Java necesaria para cada servidor (`None` si no usa Java).
pub fn java_needed(edition: &str, core: &str, version: &str) -> Option<u32> {
    if edition == "bedrock" {
        return (core == "nukkit").then_some(21);
    }
    match catalog::mojang_java_major(version) {
        Ok(Some(major)) => Some(major.max(8)),
        _ => Some(java_by_version(version)),
    }
}

/// Regla de respaldo si Mojang no responde.
pub fn java_by_version(version: &str) -> u32 {
    let v = vt(version);
    if v.first() != Some(&1) {
        25 // versiones con numeracion por ano (26.x en adelante)
    } else if v < vec![1, 17] {
        8
    } else if v < vec![1, 20, 5] {
        17
    } else {
        21
    }
}

fn find_java_exe(base: &Path) -> Option<PathBuf> {
    let name = if IS_WIN { "java.exe" } else { "java" };
    walk_find(base, &mut |dir: &Path, files: &[String]| {
        let in_bin = dir.file_name().is_some_and(|n| n == "bin");
        (in_bin && files.iter().any(|f| f == name)).then(|| dir.join(name))
    })
}

/// Descarga Java en `~/.open-mc-host/java/<version>` si aun no esta y devuelve el ejecutable.
pub fn ensure_java(major: u32) -> anyhow::Result<PathBuf> {
    let java_root = platform::app_home().join("java");
    let base = java_root.join(major.to_string());
    if base.is_dir()
        && let Some(exe) = find_java_exe(&base)
    {
        return Ok(exe);
    }
    let (os, arch) = platform::platform_ids();
    ui::header(&format!("Instalando Java {major}"));
    let mut package = None;
    for image in ["jre", "jdk"] {
        let url = format!(
            "https://api.adoptium.net/v3/assets/latest/{major}/hotspot?os={os}&architecture={arch}&image_type={image}&vendor=eclipse"
        );
        let assets = match http::get_json(&url, &[]) {
            Ok(assets) => assets,
            Err(err) if http_status(&err).is_some() => Value::Null,
            Err(err) => return Err(err),
        };
        if let Some(first) = assets.as_array().and_then(|list| list.first()) {
            package = first.get("binary").and_then(|b| b.get("package")).cloned();
            break;
        }
    }
    let Some(package) = package else {
        return Err(fail(format!(
            "Adoptium no tiene Java {major} para {os}/{arch}."
        )));
    };
    let name = http::str_field(&package, "name")?;
    let link = http::str_field(&package, "link")?;
    let checksum = package.get("checksum").and_then(Value::as_str);
    let archive = java_root.join(py_basename(name));
    http::download(
        link,
        &archive,
        Checksums::sha256(checksum),
        Some(&format!("Java {major}")),
    )?;
    if base.is_dir() {
        fs::remove_dir_all(&base)
            .with_context(|| format!("No se pudo borrar {}", base.display()))?;
    }
    safe_extract(&archive, &base)?;
    fs::remove_file(&archive)
        .with_context(|| format!("No se pudo borrar {}", archive.display()))?;
    let exe = find_java_exe(&base)
        .ok_or_else(|| fail("No se encontro el ejecutable de Java tras la instalacion."))?;
    platform::make_executable(&exe)?;
    ui::ok(&format!("Java {major} instalado en {}", base.display()));
    Ok(exe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regla_de_respaldo_como_python() {
        assert_eq!(java_by_version("1.8.9"), 8);
        assert_eq!(java_by_version("1.16.5"), 8);
        assert_eq!(java_by_version("1.17"), 17);
        assert_eq!(java_by_version("1.20.4"), 17);
        assert_eq!(java_by_version("1.20.5"), 21);
        assert_eq!(java_by_version("1.21.11"), 21);
        assert_eq!(java_by_version("26.1.2"), 25);
    }

    #[test]
    fn bedrock_solo_nukkit_usa_java() {
        assert_eq!(java_needed("bedrock", "nukkit", "latest"), Some(21));
        assert_eq!(java_needed("bedrock", "vanilla", "1.21.50.10"), None);
        assert_eq!(java_needed("bedrock", "pocketmine", "latest"), None);
    }

    #[test]
    fn busca_java_dentro_de_bin() {
        let tmp = tempfile::tempdir().unwrap();
        let exe = if IS_WIN { "java.exe" } else { "java" };
        fs::create_dir_all(tmp.path().join("jdk-21").join("bin")).unwrap();
        fs::write(tmp.path().join("jdk-21").join(exe), "").unwrap();
        fs::write(tmp.path().join("jdk-21").join("bin").join(exe), "").unwrap();
        assert_eq!(
            find_java_exe(tmp.path()),
            Some(tmp.path().join("jdk-21").join("bin").join(exe))
        );
    }
}
