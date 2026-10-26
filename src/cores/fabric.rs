//! Fabric: lanzador del servidor desde Fabric Meta.

use serde_json::Value;

use crate::config::{ServerConfig, StartSpec};
use crate::error::fail;
use crate::http::{self, Checksums};
use crate::util::py_truthy;

const META: &str = "https://meta.fabricmc.net/v2/versions";

/// La primera version marcada como estable de una lista de Fabric Meta.
fn first_stable(list: &Value) -> anyhow::Result<String> {
    list.as_array()
        .into_iter()
        .flatten()
        .find(|item| item.get("stable").is_some_and(py_truthy))
        .and_then(|item| item.get("version").and_then(Value::as_str))
        .map(str::to_owned)
        .ok_or_else(|| fail("Respuesta inesperada de Fabric: no hay ninguna version estable."))
}

pub fn install(cfg: &ServerConfig) -> anyhow::Result<StartSpec> {
    let games = http::get_json(&format!("{META}/game"), &[])?;
    let supported = games
        .as_array()
        .into_iter()
        .flatten()
        .any(|game| game.get("version").and_then(Value::as_str) == Some(cfg.version.as_str()));
    if !supported {
        return Err(fail(format!(
            "Fabric no soporta la version {}.",
            cfg.version
        )));
    }
    let loader = first_stable(&http::get_json(&format!("{META}/loader"), &[])?)?;
    let installer = first_stable(&http::get_json(&format!("{META}/installer"), &[])?)?;
    http::download(
        &format!(
            "{META}/loader/{}/{loader}/{installer}/server/jar",
            cfg.version
        ),
        &cfg.dir.join("fabric-server-launch.jar"),
        Checksums::default(),
        Some(&format!("fabric {}", cfg.version)),
    )?;
    Ok(StartSpec::Jar {
        jar: "fabric-server-launch.jar".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn primera_version_estable() {
        let list = json!([{"version": "0.17.0-beta", "stable": false}, {"version": "0.16.14", "stable": true}, {"version": "0.16.13", "stable": true}]);
        assert_eq!(first_stable(&list).unwrap(), "0.16.14");
        assert!(first_stable(&json!([])).is_err());
    }
}
