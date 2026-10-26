//! Purpur: API v2 de PurpurMC (sin hash publicado).

use serde_json::Value;

use crate::config::{ServerConfig, StartSpec};
use crate::error::fail;
use crate::http::{self, Checksums};
use crate::util::vt;

const API: &str = "https://api.purpurmc.org/v2/purpur";

/// Versiones de Purpur de menor a mayor.
pub fn versions() -> anyhow::Result<Vec<String>> {
    let data = http::get_json(API, &[])?;
    let mut versions: Vec<String> = data
        .get("versions")
        .and_then(Value::as_array)
        .ok_or_else(|| fail("Respuesta inesperada de Purpur: falta 'versions'."))?
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();
    versions.sort_by_key(|v| vt(v));
    Ok(versions)
}

pub fn install(cfg: &ServerConfig) -> anyhow::Result<StartSpec> {
    if !versions()?.contains(&cfg.version) {
        return Err(fail(format!("Purpur no tiene la version {}.", cfg.version)));
    }
    http::download(
        &format!("{API}/{}/latest/download", cfg.version),
        &cfg.dir.join("server.jar"),
        Checksums::default(),
        Some(&format!("purpur {}", cfg.version)),
    )?;
    Ok(StartSpec::Jar {
        jar: "server.jar".into(),
    })
}
