//! Bedrock Dedicated Server oficial (solo Windows y Linux x64).

use std::fs;
use std::sync::LazyLock;

use anyhow::Context;
use regex::Regex;
use serde_json::Value;

use crate::catalog::BDS_LINKS;
use crate::config::{RUN_DIR, ServerConfig, StartSpec};
use crate::error::fail;
use crate::extract::safe_extract;
use crate::http::{self, Checksums};
use crate::platform::{self, IS_WIN};

static VERSION_IN_URL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"bedrock-server-([0-9.]+)\.zip").expect("expresion regular fija y probada")
});

/// Enlace de descarga de la ultima version publicada.
pub fn latest_url() -> anyhow::Result<String> {
    let (os, arch) = platform::platform_ids();
    if !matches!(os, "windows" | "linux") || arch != "x64" {
        return Err(fail(
            "Bedrock Dedicated Server solo existe para Windows y Linux de 64 bits (x64).",
        ));
    }
    let key = if IS_WIN {
        "serverBedrockWindows"
    } else {
        "serverBedrockLinux"
    };
    let data = http::cached_json(BDS_LINKS)?;
    let links = data
        .get("result")
        .and_then(|result| result.get("links"))
        .and_then(Value::as_array)
        .ok_or_else(|| fail("Respuesta inesperada de Minecraft: falta 'result.links'."))?;
    for link in links {
        if link.get("downloadType").and_then(Value::as_str) == Some(key) {
            return Ok(http::str_field(link, "downloadUrl")?.to_owned());
        }
    }
    Err(fail(
        "No se encontro el enlace de descarga de Bedrock Dedicated Server.",
    ))
}

pub fn latest_version() -> anyhow::Result<String> {
    Ok(version_from_url(&latest_url()?).unwrap_or_else(|| "latest".to_owned()))
}

fn version_from_url(url: &str) -> Option<String> {
    VERSION_IN_URL
        .captures(url)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_owned())
}

pub fn install(cfg: &ServerConfig) -> anyhow::Result<StartSpec> {
    let url = if cfg.version == "latest" || cfg.version == latest_version()? {
        latest_url()?
    } else {
        latest_url()?; // comprueba que el sistema sea compatible
        format!(
            "https://www.minecraft.net/bedrockdedicatedserver/bin-{}/bedrock-server-{}.zip",
            if IS_WIN { "win" } else { "linux" },
            cfg.version
        )
    };
    let archive = cfg.dir.join(RUN_DIR).join("bedrock-server.zip");
    http::download(
        &url,
        &archive,
        Checksums::default(),
        Some(&format!("bedrock server {}", cfg.version)),
    )?;
    safe_extract(&archive, &cfg.dir)?;
    fs::remove_file(&archive)
        .with_context(|| format!("No se pudo borrar {}", archive.display()))?;
    let exe = if IS_WIN {
        "bedrock_server.exe"
    } else {
        "bedrock_server"
    };
    platform::make_executable(&cfg.dir.join(exe))?;
    Ok(StartSpec::Native { exe: exe.into() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_desde_el_enlace() {
        assert_eq!(
            version_from_url("https://www.minecraft.net/bedrockdedicatedserver/bin-linux/bedrock-server-1.21.111.1.zip")
                .as_deref(),
            Some("1.21.111.1")
        );
        assert_eq!(version_from_url("https://x/otro.zip"), None);
    }
}
