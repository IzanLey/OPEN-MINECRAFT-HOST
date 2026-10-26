//! NeoForge: version mas reciente para la de Minecraft y su instalador oficial.

use serde_json::Value;

use crate::catalog::NEOFORGE_VERSIONS;
use crate::config::{RUN_DIR, ServerConfig, StartSpec};
use crate::cores::{modloader_start, run_installer};
use crate::error::fail;
use crate::http::{self, Checksums};
use crate::ui;
use crate::util::vt;

/// Prefijo de las versiones de NeoForge para una de Minecraft: 1.21.1 -> `21.1.`,
/// 26.1 -> `26.1.0.`.
fn prefix_for(version: &str) -> String {
    let parts = vt(version);
    let at = |i: usize| parts.get(i).copied().unwrap_or(0);
    if parts.first() == Some(&1) {
        format!("{}.{}.", at(1), at(2))
    } else {
        format!("{}.{}.{}.", at(0), at(1), at(2))
    }
}

/// La version mas alta para esa de Minecraft, mejor sin `beta` ni `alpha`.
fn pick(version: &str, versions: &[String]) -> Option<String> {
    let prefix = prefix_for(version);
    let candidates: Vec<&String> = versions.iter().filter(|v| v.starts_with(&prefix)).collect();
    let stable: Vec<&String> = candidates
        .iter()
        .copied()
        .filter(|v| !v.contains("beta") && !v.contains("alpha"))
        .collect();
    let pool = if stable.is_empty() {
        candidates
    } else {
        stable
    };
    let mut best: Option<&String> = None;
    for candidate in pool {
        if best.is_none_or(|b| vt(candidate) > vt(b)) {
            best = Some(candidate);
        }
    }
    best.cloned()
}

pub fn install(cfg: &ServerConfig) -> anyhow::Result<StartSpec> {
    let data = http::get_json(NEOFORGE_VERSIONS, &[])?;
    let versions: Vec<String> = data
        .get("versions")
        .and_then(Value::as_array)
        .ok_or_else(|| fail("Respuesta inesperada de NeoForge: falta 'versions'."))?
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();
    let Some(neo) = pick(&cfg.version, &versions) else {
        return Err(fail(format!(
            "NeoForge no tiene builds para la version {}.",
            cfg.version
        )));
    };
    ui::info(&format!("NeoForge {neo}"));
    let installer = cfg.dir.join(RUN_DIR).join("neoforge-installer.jar");
    http::download(
        &format!(
            "https://maven.neoforged.net/releases/net/neoforged/neoforge/{neo}/neoforge-{neo}-installer.jar"
        ),
        &installer,
        Checksums::default(),
        Some("neoforge installer"),
    )?;
    run_installer(cfg, &installer)?;
    modloader_start(&cfg.dir, &["net", "neoforged", "neoforge"])
}

#[cfg(test)]
mod tests {
    use super::*;

    // Resultados de la misma seleccion en legacy/omh.py.
    #[test]
    fn eleccion_como_python() {
        let versions: Vec<String> = [
            "20.2.3-beta",
            "20.2.86",
            "20.4.80-beta",
            "20.4.237",
            "21.0.167",
            "21.1.1",
            "21.1.77",
            "21.1.209",
            "21.10.3-beta",
            "26.1.0.1-beta",
            "26.1.0.5-beta",
            "26.1.2.3",
        ]
        .map(String::from)
        .to_vec();
        let cases = [
            ("1.20.2", "20.2.", Some("20.2.86")),
            ("1.20.4", "20.4.", Some("20.4.237")),
            ("1.21", "21.0.", Some("21.0.167")),
            ("1.21.1", "21.1.", Some("21.1.209")),
            ("1.21.10", "21.10.", Some("21.10.3-beta")),
            ("26.1", "26.1.0.", Some("26.1.0.5-beta")),
            ("26.1.2", "26.1.2.", Some("26.1.2.3")),
            ("1.19.2", "19.2.", None),
        ];
        for (minecraft, prefix, expected) in cases {
            assert_eq!(prefix_for(minecraft), prefix, "{minecraft}");
            assert_eq!(
                pick(minecraft, &versions).as_deref(),
                expected,
                "{minecraft}"
            );
        }
    }
}
