//! Forge: build recomendada (o la ultima) y su instalador oficial.

use serde_json::{Map, Value};

use crate::catalog::{self, FORGE_PROMOS};
use crate::config::{RUN_DIR, ServerConfig, StartSpec};
use crate::cores::{modloader_start, run_installer};
use crate::error::{fail, http_status};
use crate::http::{self, Checksums};
use crate::ui;
use crate::util::{is_release, py_truthy, vt};

const MAVEN: &str = "https://maven.minecraftforge.net/net/minecraftforge/forge/";

fn promos() -> anyhow::Result<Map<String, Value>> {
    http::cached_json(FORGE_PROMOS)?
        .get("promos")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| fail("Respuesta inesperada de Forge: falta 'promos'."))
}

/// Version de Minecraft mas alta con alguna build de Forge.
pub fn latest_minecraft() -> anyhow::Result<Option<String>> {
    Ok(latest_from_promos(promos()?.keys()))
}

fn latest_from_promos<'a>(keys: impl Iterator<Item = &'a String>) -> Option<String> {
    let mut best: Option<String> = None;
    for key in keys {
        let minecraft = key.rsplit_once('-').map_or(key.as_str(), |(mc, _)| mc);
        if is_release(minecraft) && best.as_deref().is_none_or(|b| vt(minecraft) > vt(b)) {
            best = Some(minecraft.to_owned());
        }
    }
    best
}

pub fn install(cfg: &ServerConfig) -> anyhow::Result<StartSpec> {
    let version = cfg.version.as_str();
    let promos = promos()?;
    let promo = |kind: &str| {
        promos
            .get(&format!("{version}-{kind}"))
            .filter(|value| py_truthy(value))
            .and_then(Value::as_str)
    };
    let Some(forge_version) = promo("recommended").or_else(|| promo("latest")) else {
        return Err(fail(format!(
            "Forge no tiene builds para la version {version}."
        )));
    };
    ui::info(&format!("Forge {version}-{forge_version}"));
    let installer = cfg.dir.join(RUN_DIR).join("forge-installer.jar");
    let mut found = false;
    for name in [
        format!("{version}-{forge_version}"),
        format!("{version}-{forge_version}-{version}"),
    ] {
        let url = format!("{MAVEN}{name}/forge-{name}-installer.jar");
        match http::download(
            &url,
            &installer,
            Checksums::default(),
            Some("forge installer"),
        ) {
            Ok(()) => {
                found = true;
                break;
            }
            Err(err) if http_status(&err) == Some(404) => continue,
            Err(err) => return Err(err),
        }
    }
    if !found {
        return Err(fail(format!(
            "No se encontro el instalador de Forge para {version}."
        )));
    }
    if vt(version) < vec![1, 17] {
        // Los instaladores antiguos usan enlaces de Mojang que ya no existen.
        let (url, sha1) = catalog::mojang_server(version)?;
        http::download(
            &url,
            &cfg.dir.join(format!("minecraft_server.{version}.jar")),
            Checksums::sha1(sha1.as_deref()),
            Some(&format!("vanilla {version}")),
        )?;
    }
    run_installer(cfg, &installer)?;
    modloader_start(&cfg.dir, &["net", "minecraftforge", "forge"])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Mismo resultado que el max(..., key=vt) de Python con estas promociones.
    #[test]
    fn ultima_version_de_minecraft_con_forge() {
        let promos = json!({
            "1.7.10-latest": "10.13.4.1614", "1.12.2-recommended": "14.23.5.2859", "1.20.1-latest": "47.4.0",
            "1.21.1-latest": "52.0.20", "1.21.10-latest": "60.0.1", "1.9-latest": "12.16.1.1938"
        });
        let keys = promos.as_object().unwrap().keys();
        assert_eq!(latest_from_promos(keys).as_deref(), Some("1.21.10"));
    }
}
