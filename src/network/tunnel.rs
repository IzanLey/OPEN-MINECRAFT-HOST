//! Tunel gratuito de playit.gg: el agente `v0.17.1` y `omh tunnel`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::Context;
use serde_json::Value;

use crate::error::fail;
use crate::http::{self, Checksums};
use crate::platform::{self, IS_WIN};
use crate::ui::{self, CtrlC};

/// La serie 1.0 de playit es un servicio que necesita otro programa para vincularse.
/// La 0.17.1 (tambien mantenida) muestra el enlace de vinculacion en la terminal.
pub const PLAYIT_VERSION: &str = "v0.17.1";

pub fn playit_path() -> PathBuf {
    platform::app_home().join("tools").join(format!(
        "playit-{PLAYIT_VERSION}{}",
        if IS_WIN { ".exe" } else { "" }
    ))
}

/// La clave del agente se guarda fuera de cualquier repositorio, siempre en el mismo sitio.
pub fn playit_command(exe: &Path) -> Command {
    let mut command = Command::new(exe);
    command
        .arg("--secret_path")
        .arg(platform::app_home().join("playit.toml"))
        .arg("--stdout")
        .arg("start");
    command
}

/// Puntuacion de un asset de la release para este sistema (`None` si no vale).
fn asset_score(name: &str, os: &str, arch: &str) -> Option<u8> {
    let name = name.to_lowercase();
    let arch_keys: &[&str] = if arch == "x64" {
        &["x86_64", "amd64"]
    } else {
        &["aarch64", "arm64"]
    };
    if !name.contains(os) || !arch_keys.iter().any(|key| name.contains(key)) {
        return None;
    }
    if IS_WIN && !name.ends_with(".exe") {
        return None;
    }
    if !IS_WIN && name.replace("x86_64", "").contains('.') {
        return None; // descarta .deb, .rpm, .sha256...
    }
    Some(if name.contains("cli") { 0 } else { 2 } + u8::from(name.contains("signed")))
}

pub fn ensure_playit() -> anyhow::Result<PathBuf> {
    let exe = playit_path();
    if exe.exists() {
        return Ok(exe);
    }
    let (os, arch) = platform::platform_ids();
    let release = http::get_json(
        &format!(
            "https://api.github.com/repos/playit-cloud/playit-agent/releases/tags/{PLAYIT_VERSION}"
        ),
        &[("Accept", "application/vnd.github+json")],
    )?;
    let mut best: Option<(u8, &Value)> = None;
    for asset in release
        .get("assets")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let name = asset
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if let Some(score) = asset_score(name, os, &arch)
            && best.is_none_or(|(best_score, _)| score > best_score)
        {
            best = Some((score, asset));
        }
    }
    let Some((_, asset)) = best else {
        return Err(fail(format!(
            "No se encontro el agente de playit.gg para {os}/{arch}."
        )));
    };
    http::download(
        http::str_field(asset, "browser_download_url")?,
        &exe,
        Checksums::default(),
        Some("agente playit.gg"),
    )?;
    platform::make_executable(&exe)?;
    Ok(exe)
}

pub fn cmd_tunnel() -> anyhow::Result<()> {
    let exe = ensure_playit()?;
    ui::info("Abriendo el agente de playit.gg.");
    ui::info("1. La primera vez aparece 'Visit link to setup https://playit.gg/claim/...'.");
    ui::info("   Abre ese enlace, crea tu cuenta gratuita (o entra) y acepta el agente.");
    ui::info("2. En https://playit.gg/account/tunnels crea un tunel 'Minecraft Java' o");
    ui::info("   'Minecraft Bedrock' con IP local 127.0.0.1 y el puerto de tu servidor.");
    ui::info("3. La web te muestra la direccion publica (por ejemplo algo.joinmc.link).");
    ui::info("   Esa es la direccion que deben usar tus amigos.");
    ui::info("Cuando este vinculado pulsa Ctrl+C. 'omh start' lanzara el tunel solo.");
    ui::set_ctrlc(CtrlC::Notify);
    let mut child = playit_command(&exe)
        .spawn()
        .with_context(|| format!("No se pudo ejecutar {}", exe.display()))?;
    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if ui::take_ctrlc() {
            let _ = child.kill();
            let _ = child.wait();
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    ui::set_ctrlc(CtrlC::Cancel);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eleccion_del_agente() {
        if IS_WIN {
            assert_eq!(
                asset_score("playit-windows-x86_64-signed.exe", "windows", "x64"),
                Some(3)
            );
            assert_eq!(
                asset_score("playit-windows-x86_64.exe", "windows", "x64"),
                Some(2)
            );
            assert_eq!(asset_score("playit-linux-amd64", "windows", "x64"), None);
        } else {
            assert_eq!(asset_score("playit-linux-amd64", "linux", "x64"), Some(2));
            assert_eq!(asset_score("playit-linux-amd64.deb", "linux", "x64"), None);
            assert_eq!(asset_score("playit-linux-aarch64", "linux", "x64"), None);
        }
    }
}
