//! Servidor oficial de Mojang (Java).

use crate::catalog;
use crate::config::{ServerConfig, StartSpec};
use crate::http::{self, Checksums};

pub fn install(cfg: &ServerConfig) -> anyhow::Result<StartSpec> {
    let (url, sha1) = catalog::mojang_server(&cfg.version)?;
    http::download(
        &url,
        &cfg.dir.join("server.jar"),
        Checksums::sha1(sha1.as_deref()),
        Some(&format!("vanilla {}", cfg.version)),
    )?;
    Ok(StartSpec::Jar {
        jar: "server.jar".into(),
    })
}
