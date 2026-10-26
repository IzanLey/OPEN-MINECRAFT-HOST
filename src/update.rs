//! `omh update`: descarga el binario de la ultima release de GitHub, comprueba su
//! SHA-256 con `SHA256SUMS` y sustituye el ejecutable.

use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use reqwest::Method;
use serde_json::Value;

use crate::error::fail;
use crate::http::{self, Checksums};
use crate::platform::{self, IS_WIN};
use crate::ui;

/// Nombre del binario de cada sistema en las releases.
pub const ASSET: &str = if IS_WIN {
    "omh-windows-x86_64.exe"
} else {
    "omh-linux-x86_64"
};

fn with_suffix(path: &std::path::Path, suffix: &str) -> PathBuf {
    let mut text = path.as_os_str().to_owned();
    text.push(suffix);
    PathBuf::from(text)
}

/// Hash de `name` en un archivo `SHA256SUMS` (formato de `sha256sum`).
fn hash_for<'a>(sums: &'a str, name: &str) -> Option<&'a str> {
    sums.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        (parts.next()?.trim_start_matches('*') == name).then_some(hash)
    })
}

fn asset_url<'a>(release: &'a Value, name: &str) -> Option<&'a str> {
    release
        .get("assets")?
        .as_array()?
        .iter()
        .find(|asset| asset.get("name").and_then(Value::as_str) == Some(name))?
        .get("browser_download_url")?
        .as_str()
}

pub fn cmd_update() -> anyhow::Result<()> {
    let release = http::github_latest_release(crate::GITHUB_REPO)?;
    let url = asset_url(&release, ASSET).ok_or_else(|| {
        fail(format!(
            "La ultima version no tiene binario para este sistema ({ASSET})."
        ))
    })?;
    let sums_url = asset_url(&release, "SHA256SUMS")
        .ok_or_else(|| fail("La ultima version no publica SHA256SUMS."))?;
    let sums = http::read_body(http::request(
        Method::GET,
        sums_url,
        &[],
        None,
        http::JSON_TIMEOUT,
    )?)?;
    let sums = String::from_utf8_lossy(&sums);
    let expected =
        hash_for(&sums, ASSET).ok_or_else(|| fail(format!("SHA256SUMS no incluye {ASSET}.")))?;

    let me = env::current_exe().context("No se pudo localizar el ejecutable de omh")?;
    ui::info(&format!("Descargando la ultima version desde {url}"));
    let new = with_suffix(&me, ".new");
    http::download(url, &new, Checksums::sha256(Some(expected)), Some("omh"))?;
    platform::make_executable(&new)?;
    if IS_WIN {
        // Un .exe en uso (por ejemplo el supervisor de un servidor en marcha) no se
        // puede sobrescribir, pero si renombrar.
        let old = with_suffix(&me, ".old");
        let _ = fs::remove_file(&old);
        fs::rename(&me, &old).with_context(|| format!("No se pudo renombrar {}", me.display()))?;
    }
    fs::rename(&new, &me).with_context(|| format!("No se pudo reemplazar {}", me.display()))?;
    ui::ok("open-mc-host actualizado.");
    Ok(())
}

/// Borra el `.old` que deja una actualizacion en Windows.
pub fn cleanup_old_binary() {
    if IS_WIN && let Ok(me) = env::current_exe() {
        let _ = fs::remove_file(with_suffix(&me, ".old"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_de_sha256sums() {
        let sums = "aaa  omh-linux-x86_64\nbbb *omh-windows-x86_64.exe\n";
        assert_eq!(hash_for(sums, "omh-linux-x86_64"), Some("aaa"));
        assert_eq!(hash_for(sums, "omh-windows-x86_64.exe"), Some("bbb"));
        assert_eq!(hash_for(sums, "otro"), None);
    }
}
