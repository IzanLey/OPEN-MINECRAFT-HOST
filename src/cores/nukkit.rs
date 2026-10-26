//! PowerNukkitX: el jar de la ultima release o, si solo hay zip, el jar que trae.

use std::cmp::Reverse;
use std::fs;
use std::path::{MAIN_SEPARATOR_STR, Path, PathBuf};

use anyhow::Context;
use serde_json::Value;

use super::asset_name;
use crate::config::{RUN_DIR, ServerConfig, StartSpec};
use crate::error::fail;
use crate::extract::safe_extract;
use crate::http::{self, Checksums};
use crate::util::{glob_recursive, name_key, py_basename, relpath};

pub fn install(cfg: &mut ServerConfig) -> anyhow::Result<StartSpec> {
    let release = http::github_latest_release("PowerNukkitX/PowerNukkitX")?;
    let assets: Vec<&Value> = release
        .get("assets")
        .and_then(Value::as_array)
        .map(|assets| assets.iter().collect())
        .unwrap_or_default();
    let mut jars: Vec<&Value> = assets
        .iter()
        .copied()
        .filter(|a| asset_name(a).ends_with(".jar"))
        .collect();
    let zips: Vec<&Value> = assets
        .iter()
        .copied()
        .filter(|a| asset_name(a).ends_with(".zip"))
        .collect();
    cfg.version = release
        .get("tag_name")
        .and_then(Value::as_str)
        .unwrap_or("latest")
        .to_owned();
    if !jars.is_empty() {
        // Primero los que traen todas las dependencias y, entre ellos, el mas grande.
        jars.sort_by_key(|asset| {
            let name = asset_name(asset).to_lowercase();
            let size = asset.get("size").and_then(Value::as_u64).unwrap_or(0);
            (
                !(name.contains("all") || name.contains("shaded")),
                Reverse(size),
            )
        });
        http::download(
            http::str_field(jars[0], "browser_download_url")?,
            &cfg.dir.join("powernukkitx.jar"),
            Checksums::default(),
            Some("PowerNukkitX"),
        )?;
        return Ok(StartSpec::Jar {
            jar: "powernukkitx.jar".into(),
        });
    }
    let Some(zip) = zips.first() else {
        return Err(fail(
            "No se encontro ningun archivo descargable de PowerNukkitX.",
        ));
    };
    let archive = cfg.dir.join(RUN_DIR).join(py_basename(asset_name(zip)));
    http::download(
        http::str_field(zip, "browser_download_url")?,
        &archive,
        Checksums::default(),
        Some("PowerNukkitX"),
    )?;
    safe_extract(&archive, &cfg.dir)?;
    fs::remove_file(&archive)
        .with_context(|| format!("No se pudo borrar {}", archive.display()))?;
    let jar = largest_server_jar(&cfg.dir)?;
    Ok(StartSpec::Jar {
        jar: relpath(&jar, &cfg.dir),
    })
}

/// El jar mas grande que no este en `plugins/` ni `libs/`. Se mira la ruta dentro de
/// la carpeta del servidor (Python miraba la ruta completa; ver B6 en la boveda).
fn largest_server_jar(dir: &Path) -> anyhow::Result<PathBuf> {
    let mut jars = glob_recursive(dir, &|name: &str| name_key(name).ends_with(".jar"));
    jars.sort_by_key(|jar| fs::metadata(jar).map(|meta| meta.len()).unwrap_or(0));
    jars.retain(|jar| {
        let inside = format!("{MAIN_SEPARATOR_STR}{}", relpath(jar, dir));
        let plugins = format!("{MAIN_SEPARATOR_STR}plugins{MAIN_SEPARATOR_STR}");
        let libs = format!("{MAIN_SEPARATOR_STR}libs{MAIN_SEPARATOR_STR}");
        !inside.contains(&plugins) && !inside.contains(&libs)
    });
    jars.pop()
        .ok_or_else(|| fail("No se encontro el jar de PowerNukkitX."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jar_del_servidor_aunque_la_carpeta_este_en_libs() {
        let tmp = tempfile::tempdir().unwrap();
        // La carpeta del servidor esta dentro de otra llamada "libs" (caso B6).
        let dir = tmp.path().join("libs").join("servidor");
        fs::create_dir_all(dir.join("plugins")).unwrap();
        fs::create_dir_all(dir.join("libs")).unwrap();
        fs::write(dir.join("powernukkitx.jar"), vec![0u8; 100]).unwrap();
        fs::write(dir.join("plugins").join("grande.jar"), vec![0u8; 1000]).unwrap();
        fs::write(dir.join("libs").join("grande.jar"), vec![0u8; 1000]).unwrap();
        fs::write(dir.join("pequeno.jar"), vec![0u8; 10]).unwrap();
        assert_eq!(
            largest_server_jar(&dir).unwrap(),
            dir.join("powernukkitx.jar")
        );
    }
}
