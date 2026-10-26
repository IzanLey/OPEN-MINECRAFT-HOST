//! Paper: la API Fill v3 de PaperMC y, si falla, la API v2.

use serde_json::Value;

use crate::config::{ServerConfig, StartSpec};
use crate::error::fail;
use crate::http::{self, Checksums};
use crate::util::{is_release, vt};

const FILL: &str = "https://fill.papermc.io/v3/projects/paper";
const V2: &str = "https://api.papermc.io/v2/projects/paper";

/// Versiones normales de Paper, de menor a mayor.
pub fn versions() -> anyhow::Result<Vec<String>> {
    let all = match http::get_json(FILL, &[]).and_then(|data| fill_versions(&data)) {
        Ok(all) => all,
        Err(_) => strings(http::get_json(V2, &[])?.get("versions"))
            .ok_or_else(|| fail("Respuesta inesperada de PaperMC: falta 'versions'."))?,
    };
    let mut releases: Vec<String> = all.into_iter().filter(|v| is_release(v)).collect();
    releases.sort_by_key(|v| vt(v));
    Ok(releases)
}

/// Fill v3 agrupa las versiones: `{"versions": {"1.21": ["1.21.1", ...], ...}}`.
fn fill_versions(data: &Value) -> anyhow::Result<Vec<String>> {
    let Some(groups) = data.get("versions") else {
        return Ok(Vec::new());
    };
    let groups = groups
        .as_object()
        .ok_or_else(|| fail("Respuesta inesperada de PaperMC."))?;
    let mut all = Vec::new();
    for group in groups.values() {
        all.extend(strings(Some(group)).ok_or_else(|| fail("Respuesta inesperada de PaperMC."))?);
    }
    Ok(all)
}

fn strings(value: Option<&Value>) -> Option<Vec<String>> {
    value?
        .as_array()?
        .iter()
        .map(|v| v.as_str().map(str::to_owned))
        .collect()
}

/// URL y SHA-256 de la ultima build de una version.
pub fn download_info(version: &str) -> anyhow::Result<(String, Option<String>)> {
    if let Ok(found) = fill_download(version) {
        return Ok(found);
    }
    v2_download(version)
        .map_err(|_| fail(format!("Paper no tiene builds para la version {version}.")))
}

fn fill_download(version: &str) -> anyhow::Result<(String, Option<String>)> {
    let build = http::get_json(&format!("{FILL}/versions/{version}/builds/latest"), &[])?;
    let download = build
        .get("downloads")
        .and_then(|downloads| downloads.get("server:default"))
        .ok_or_else(|| fail("Respuesta inesperada de PaperMC."))?;
    let sha256 = download
        .get("checksums")
        .and_then(|sums| sums.get("sha256"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    Ok((http::str_field(download, "url")?.to_owned(), sha256))
}

fn v2_download(version: &str) -> anyhow::Result<(String, Option<String>)> {
    let base = format!("{V2}/versions/{version}");
    let builds = http::get_json(&format!("{base}/builds"), &[])?;
    let last = builds
        .get("builds")
        .and_then(Value::as_array)
        .and_then(|builds| builds.last())
        .ok_or_else(|| fail("Respuesta inesperada de PaperMC."))?;
    let number = last
        .get("build")
        .and_then(Value::as_i64)
        .ok_or_else(|| fail("Respuesta inesperada de PaperMC."))?;
    let app = last
        .get("downloads")
        .and_then(|downloads| downloads.get("application"))
        .ok_or_else(|| fail("Respuesta inesperada de PaperMC."))?;
    let name = http::str_field(app, "name")?;
    let sha256 = app.get("sha256").and_then(Value::as_str).map(str::to_owned);
    Ok((format!("{base}/builds/{number}/downloads/{name}"), sha256))
}

pub fn install(cfg: &ServerConfig) -> anyhow::Result<StartSpec> {
    let (url, sha256) = download_info(&cfg.version)?;
    http::download(
        &url,
        &cfg.dir.join("server.jar"),
        Checksums::sha256(sha256.as_deref()),
        Some(&format!("paper {}", cfg.version)),
    )?;
    Ok(StartSpec::Jar {
        jar: "server.jar".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn versiones_agrupadas_de_fill() {
        let data = json!({"versions": {"1.21": ["1.21.4", "1.21.1"], "1.8": ["1.8.8"]}});
        assert_eq!(fill_versions(&data).unwrap(), ["1.21.4", "1.21.1", "1.8.8"]);
        assert!(fill_versions(&json!({})).unwrap().is_empty());
        assert!(fill_versions(&json!({"versions": []})).is_err());
    }
}
