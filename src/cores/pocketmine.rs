//! PocketMine-MP: el `.phar` de la ultima release y el PHP que publica su equipo.

use std::fs;
use std::path::Path;

use anyhow::Context;
use serde_json::Value;

use super::asset_name;
use crate::config::{RUN_DIR, ServerConfig, StartSpec};
use crate::error::fail;
use crate::extract::safe_extract;
use crate::http::{self, Checksums};
use crate::platform::{self, IS_WIN};
use crate::util::{
    glob_recursive, py_basename, py_splitlines, py_truthy, read_text_lossy, relpath, vt, walk_find,
    write_text,
};

const PHP_RELEASES: &str = "https://api.github.com/repos/pmmp/PHP-Binaries/releases";

pub fn install(cfg: &mut ServerConfig) -> anyhow::Result<StartSpec> {
    let release = http::github_latest_release("pmmp/PocketMine-MP")?;
    let assets: &[Value] = release
        .get("assets")
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice);
    let phar = assets
        .iter()
        .find(|asset| asset_name(asset) == "PocketMine-MP.phar")
        .ok_or_else(|| fail("No se encontro PocketMine-MP.phar en la ultima version."))?;
    let tag = http::str_field(&release, "tag_name")?;
    http::download(
        http::str_field(phar, "browser_download_url")?,
        &cfg.dir.join("PocketMine-MP.phar"),
        Checksums::default(),
        Some(&format!("PocketMine-MP {tag}")),
    )?;
    cfg.version = tag.to_owned();

    // Binario de PHP preparado por el equipo de PocketMine (el que indica build_info.json).
    let pm_major = format!("PM{}", vt(tag).first().copied().unwrap_or(0));
    let (os_key, arch_key) = if IS_WIN {
        ("Windows", "x64")
    } else {
        ("Linux", "x86_64")
    };
    let mut releases = Vec::new();
    if let Some(build_info) = assets
        .iter()
        .find(|asset| asset_name(asset) == "build_info.json")
        && let Ok(php_release) = php_release_from_build_info(build_info)
    {
        releases.push(php_release);
    }
    if releases.is_empty() {
        releases = http::get_json(&format!("{PHP_RELEASES}?per_page=30"), &[])?
            .as_array()
            .cloned()
            .unwrap_or_default();
    }
    let asset = pick_php_asset(&releases, os_key, arch_key, &pm_major)
        .ok_or_else(|| fail("No se encontro un binario de PHP compatible para tu sistema."))?;
    let archive = cfg.dir.join(RUN_DIR).join(py_basename(asset_name(asset)));
    http::download(
        http::str_field(asset, "browser_download_url")?,
        &archive,
        Checksums::default(),
        Some("PHP para PocketMine"),
    )?;
    let bin_dir = cfg.dir.join("bin");
    if bin_dir.is_dir() {
        fs::remove_dir_all(&bin_dir)
            .with_context(|| format!("No se pudo borrar {}", bin_dir.display()))?;
    }
    safe_extract(&archive, &bin_dir)?;
    fs::remove_file(&archive)
        .with_context(|| format!("No se pudo borrar {}", archive.display()))?;

    let name = if IS_WIN { "php.exe" } else { "php" };
    let php = walk_find(&bin_dir, &mut |dir: &Path, files: &[String]| {
        let in_bin = IS_WIN || dir.file_name().is_some_and(|n| n == "bin");
        (in_bin && files.iter().any(|f| f == name)).then(|| dir.join(name))
    })
    .ok_or_else(|| fail("No se encontro el ejecutable de PHP."))?;
    platform::make_executable(&php)?;
    if !IS_WIN {
        let extensions = glob_recursive(&bin_dir, &|n: &str| n.starts_with("no-debug-zts-"));
        let ini = php.parent().unwrap_or(&bin_dir).join("php.ini");
        if let Some(extensions) = extensions.first()
            && ini.exists()
        {
            let text = read_text_lossy(&ini)
                .with_context(|| format!("No se pudo leer {}", ini.display()))?;
            let mut lines: Vec<String> = py_splitlines(&text)
                .into_iter()
                .filter(|line| !line.starts_with("extension_dir"))
                .map(str::to_owned)
                .collect();
            lines.push(format!("extension_dir=\"{}\"", extensions.display()));
            write_text(&ini, &(lines.join("\n") + "\n"))?;
        }
    }
    Ok(StartSpec::Php {
        php: relpath(&php, &cfg.dir),
        phar: "PocketMine-MP.phar".into(),
    })
}

/// Release de PHP-Binaries que pide `build_info.json` (`php_download_url`).
fn php_release_from_build_info(build_info: &Value) -> anyhow::Result<Value> {
    let info = http::get_json(http::str_field(build_info, "browser_download_url")?, &[])?;
    let url = http::str_field(&info, "php_download_url")?;
    let (_, tag) = url
        .trim_end_matches('/')
        .rsplit_once('/')
        .ok_or_else(|| fail("Respuesta inesperada de PocketMine."))?;
    http::get_json(&format!("{PHP_RELEASES}/tags/{tag}"), &[])
}

/// El primer asset de PHP para el sistema, la arquitectura y la version mayor de
/// PocketMine (sin depuracion), saltando borradores y prereleases.
fn pick_php_asset<'a>(
    releases: &'a [Value],
    os_key: &str,
    arch_key: &str,
    pm_major: &str,
) -> Option<&'a Value> {
    releases
        .iter()
        .filter(|release| {
            !release.get("draft").is_some_and(py_truthy)
                && !release.get("prerelease").is_some_and(py_truthy)
        })
        .find_map(|release| {
            release.get("assets")?.as_array()?.iter().find(|asset| {
                let name = asset_name(asset);
                name.contains(os_key)
                    && name.contains(arch_key)
                    && name.contains(pm_major)
                    && !name.to_lowercase().contains("debug")
                    && (name.ends_with(".zip") || name.ends_with(".tar.gz"))
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn eleccion_del_php() {
        let releases = json!([
            {"draft": true, "assets": [{"name": "PHP-8.3-Linux-x86_64-PM5.tar.gz"}]},
            {"prerelease": false, "assets": [
                {"name": "PHP-8.3-Linux-x86_64-PM5-debugging-symbols.tar.gz"},
                {"name": "PHP-8.3-Windows-x64-PM5.zip"},
                {"name": "PHP-8.3-Linux-x86_64-PM5.tar.gz"}
            ]}
        ]);
        let releases = releases.as_array().unwrap();
        let linux = pick_php_asset(releases, "Linux", "x86_64", "PM5").unwrap();
        assert_eq!(asset_name(linux), "PHP-8.3-Linux-x86_64-PM5.tar.gz");
        let windows = pick_php_asset(releases, "Windows", "x64", "PM5").unwrap();
        assert_eq!(asset_name(windows), "PHP-8.3-Windows-x64-PM5.zip");
        assert!(pick_php_asset(releases, "Linux", "x86_64", "PM6").is_none());
    }
}
