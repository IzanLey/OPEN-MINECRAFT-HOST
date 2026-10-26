//! Catalogo de ediciones, cores y versiones, la resolucion de `latest` y `omh list`.

use serde_json::Value;

use crate::cores::{bedrock, forge, paper, purpur};
use crate::error::fail;
use crate::http;
use crate::ui;
use crate::util::{is_release, py_strip, vt};

pub const JAVA_COMMON: [&str; 16] = [
    "1.8.9", "1.12.2", "1.16.5", "1.17.1", "1.18.2", "1.19.4", "1.20.1", "1.20.4", "1.20.6",
    "1.21.1", "1.21.4", "1.21.8", "1.21.11", "26.1.2", "26.2", "26.3",
];

pub const MOJANG_MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
pub const BDS_LINKS: &str =
    "https://net-secondary.web.minecraft-services.net/api/v1.0/download/links";
pub const FORGE_PROMOS: &str =
    "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
pub const NEOFORGE_VERSIONS: &str =
    "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";

/// Tipo de complemento que admite un core.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Addons {
    Plugins,
    Mods,
}

pub struct Core {
    pub edition: &'static str,
    pub name: &'static str,
    pub min: Option<&'static str>,
    pub addons: Option<Addons>,
    pub desc: &'static str,
}

/// En el mismo orden que en Python: asi se listan y se ofrecen.
pub const CORES: [Core; 9] = [
    Core {
        edition: "java",
        name: "vanilla",
        min: Some("1.7.10"),
        addons: None,
        desc: "Servidor oficial de Mojang. Sin plugins ni mods.",
    },
    Core {
        edition: "java",
        name: "paper",
        min: Some("1.8.8"),
        addons: Some(Addons::Plugins),
        desc: "Alto rendimiento. Plugins Bukkit/Spigot/Paper.",
    },
    Core {
        edition: "java",
        name: "purpur",
        min: Some("1.14.1"),
        addons: Some(Addons::Plugins),
        desc: "Fork de Paper con mas opciones. Plugins Bukkit/Spigot/Paper.",
    },
    Core {
        edition: "java",
        name: "fabric",
        min: Some("1.14"),
        addons: Some(Addons::Mods),
        desc: "Cargador de mods ligero.",
    },
    Core {
        edition: "java",
        name: "forge",
        min: Some("1.7.10"),
        addons: Some(Addons::Mods),
        desc: "Cargador de mods clasico (1.8.9, 1.12.2, 1.16.5, 1.20.1...).",
    },
    Core {
        edition: "java",
        name: "neoforge",
        min: Some("1.20.2"),
        addons: Some(Addons::Mods),
        desc: "Fork moderno de Forge (1.20.2 en adelante).",
    },
    Core {
        edition: "bedrock",
        name: "vanilla",
        min: None,
        addons: None,
        desc: "Bedrock Dedicated Server oficial de Mojang.",
    },
    Core {
        edition: "bedrock",
        name: "pocketmine",
        min: None,
        addons: Some(Addons::Plugins),
        desc: "PocketMine-MP (PHP). Plugins de Poggit.",
    },
    Core {
        edition: "bedrock",
        name: "nukkit",
        min: None,
        addons: Some(Addons::Plugins),
        desc: "PowerNukkitX, fork mantenido de Nukkit (Java). Plugins .jar.",
    },
];

pub const EDITIONS: [&str; 2] = ["java", "bedrock"];

pub fn cores_of(edition: &str) -> Vec<&'static Core> {
    CORES
        .iter()
        .filter(|core| core.edition == edition)
        .collect()
}

pub fn core(edition: &str, name: &str) -> Option<&'static Core> {
    CORES
        .iter()
        .find(|core| core.edition == edition && core.name == name)
}

/// Cargadores con los que se busca en Modrinth (solo cores de Java con complementos).
pub fn modrinth_loaders(core: &str) -> Option<&'static [&'static str]> {
    match core {
        "paper" => Some(&["paper", "spigot", "bukkit", "purpur"]),
        "purpur" => Some(&["purpur", "paper", "spigot", "bukkit"]),
        "fabric" => Some(&["fabric"]),
        "forge" => Some(&["forge"]),
        "neoforge" => Some(&["neoforge"]),
        _ => None,
    }
}

pub fn default_port(edition: &str) -> u16 {
    if edition == "bedrock" { 19132 } else { 25565 }
}

// ---------------------------------------------------------------------------
// Mojang
// ---------------------------------------------------------------------------

pub fn mojang_latest() -> anyhow::Result<String> {
    let manifest = http::cached_json(MOJANG_MANIFEST)?;
    manifest
        .get("latest")
        .and_then(|latest| latest.get("release"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| fail("Respuesta inesperada de Mojang: falta 'latest.release'."))
}

/// JSON de una version del manifiesto de Mojang (`None` si no existe).
fn mojang_version_json(version: &str) -> anyhow::Result<Option<Value>> {
    let manifest = http::cached_json(MOJANG_MANIFEST)?;
    let entries = manifest.get("versions").and_then(Value::as_array);
    for entry in entries.into_iter().flatten() {
        if entry.get("id").and_then(Value::as_str) == Some(version) {
            return http::get_json(http::str_field(entry, "url")?, &[]).map(Some);
        }
    }
    Ok(None)
}

/// URL y SHA-1 del `server.jar` oficial de una version.
pub fn mojang_server(version: &str) -> anyhow::Result<(String, Option<String>)> {
    let Some(data) = mojang_version_json(version)? else {
        return Err(fail(format!("La version {version} no existe en Mojang.")));
    };
    let server = data
        .get("downloads")
        .and_then(|downloads| downloads.get("server"))
        .filter(|server| server.as_object().is_some_and(|map| !map.is_empty()));
    let Some(server) = server else {
        return Err(fail(format!(
            "Mojang no publica servidor para la version {version}."
        )));
    };
    let sha1 = server
        .get("sha1")
        .and_then(Value::as_str)
        .map(str::to_owned);
    Ok((http::str_field(server, "url")?.to_owned(), sha1))
}

/// `javaVersion.majorVersion` de una version (`None` si Mojang no la tiene).
pub fn mojang_java_major(version: &str) -> anyhow::Result<Option<u32>> {
    let Some(data) = mojang_version_json(version)? else {
        return Ok(None);
    };
    let major = data
        .get("javaVersion")
        .and_then(|java| java.get("majorVersion"))
        .and_then(|major| {
            major
                .as_u64()
                .or_else(|| major.as_str()?.trim().parse().ok())
        })
        .and_then(|major| u32::try_from(major).ok())
        .ok_or_else(|| fail("Respuesta inesperada de Mojang: falta 'javaVersion'."))?;
    Ok(Some(major))
}

// ---------------------------------------------------------------------------
// Resolucion de versiones
// ---------------------------------------------------------------------------

/// Version que se instala: resuelve `latest` y comprueba minimos, como Python.
pub fn resolve_version(edition: &str, core_name: &str, version: &str) -> anyhow::Result<String> {
    let requested = py_strip(if version.is_empty() {
        "latest"
    } else {
        version
    })
    .to_lowercase();
    if edition == "bedrock" {
        if core_name != "vanilla" {
            if requested != "latest" {
                ui::warn(&format!(
                    "{core_name} solo se instala en su ultima version publicada."
                ));
            }
            return Ok("latest".to_owned());
        }
        return if requested == "latest" {
            bedrock::latest_version()
        } else {
            Ok(requested)
        };
    }

    let mut v = requested;
    if v == "latest" {
        let latest = match core_name {
            "paper" => paper::versions()?.pop(),
            "purpur" => purpur::versions()?.pop(),
            "forge" => forge::latest_minecraft()?,
            _ => Some(mojang_latest()?),
        };
        v = latest
            .ok_or_else(|| fail(format!("No se encontro ninguna version de {core_name}.")))?;
        ui::info(&format!("Ultima version disponible para {core_name}: {v}"));
    }
    check_java_version(core_name, version, &v)
}

/// Comprobaciones de una version concreta de Java (sin red).
fn check_java_version(core_name: &str, original: &str, v: &str) -> anyhow::Result<String> {
    if !is_release(v) {
        return Err(fail(format!(
            "Version no valida: '{original}'. Ejemplo: 1.20.4"
        )));
    }
    if core_name == "paper" && vt(v).get(..2) == Some(&[1, 8][..]) && v != "1.8.8" {
        ui::warn(&format!(
            "Paper no tiene {v}. Se usa 1.8.8, compatible con todos los clientes 1.8.x."
        ));
        return Ok("1.8.8".to_owned());
    }
    if let Some(minimum) = core("java", core_name).and_then(|core| core.min)
        && vt(v) < vt(minimum)
    {
        return Err(fail(format!(
            "{core_name} necesita la version {minimum} o superior."
        )));
    }
    if !JAVA_COMMON.contains(&v) {
        ui::warn(&format!(
            "La version {v} no esta en la lista de versiones comunes. Se intentara igualmente."
        ));
    }
    Ok(v.to_owned())
}

// ---------------------------------------------------------------------------
// omh list
// ---------------------------------------------------------------------------

pub fn cmd_list() -> anyhow::Result<()> {
    ui::print_title();
    for edition in EDITIONS {
        ui::header(&format!("Edicion {edition}"));
        for core in cores_of(edition) {
            let extra = core
                .min
                .map(|min| format!(" (desde {min})"))
                .unwrap_or_default();
            ui::info(&format!("{:<10} {}{extra}", core.name, core.desc));
        }
    }
    ui::header("Versiones");
    ui::info(&format!("Java (comunes): {}", JAVA_COMMON.join(", ")));
    if let Ok(latest) = mojang_latest() {
        ui::info(&format!("Java (ultima):  {latest}"));
    }
    match bedrock::latest_version() {
        Ok(latest) => ui::info(&format!(
            "Bedrock:        latest = {latest} (o una version concreta, solo core vanilla)"
        )),
        Err(_) => ui::info("Bedrock:        latest (o una version concreta, solo core vanilla)"),
    }
    ui::info("Usa 'latest' para la ultima version disponible de cada core.");
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::user_message;

    #[test]
    fn catalogo_en_el_orden_de_python() {
        let java: Vec<&str> = cores_of("java").iter().map(|c| c.name).collect();
        assert_eq!(
            java,
            ["vanilla", "paper", "purpur", "fabric", "forge", "neoforge"]
        );
        let bedrock: Vec<&str> = cores_of("bedrock").iter().map(|c| c.name).collect();
        assert_eq!(bedrock, ["vanilla", "pocketmine", "nukkit"]);
        assert!(cores_of("pc").is_empty());
        assert_eq!(default_port("java"), 25565);
        assert_eq!(default_port("bedrock"), 19132);
    }

    #[test]
    fn versiones_concretas_sin_red() {
        assert_eq!(
            check_java_version("vanilla", "1.21.1", "1.21.1").unwrap(),
            "1.21.1"
        );
        assert_eq!(
            check_java_version("paper", "1.8.9", "1.8.9").unwrap(),
            "1.8.8"
        );
        assert_eq!(
            check_java_version("paper", "1.8.8", "1.8.8").unwrap(),
            "1.8.8"
        );
        let err = check_java_version("purpur", "1.12.2", "1.12.2").unwrap_err();
        assert_eq!(
            user_message(&err),
            "purpur necesita la version 1.14.1 o superior."
        );
        let err = check_java_version("neoforge", "1.20.1", "1.20.1").unwrap_err();
        assert_eq!(
            user_message(&err),
            "neoforge necesita la version 1.20.2 o superior."
        );
        let err = check_java_version("paper", " ABC ", "abc").unwrap_err();
        assert_eq!(
            user_message(&err),
            "Version no valida: ' ABC '. Ejemplo: 1.20.4"
        );
        // Version no comun: solo avisa.
        assert_eq!(
            check_java_version("fabric", "1.19.2", "1.19.2").unwrap(),
            "1.19.2"
        );
    }

    #[test]
    fn bedrock_no_vanilla_siempre_latest() {
        assert_eq!(
            resolve_version("bedrock", "pocketmine", "5.0").unwrap(),
            "latest"
        );
        assert_eq!(
            resolve_version("bedrock", "nukkit", "latest").unwrap(),
            "latest"
        );
        assert_eq!(
            resolve_version("bedrock", "vanilla", "1.21.50.10").unwrap(),
            "1.21.50.10"
        );
    }
}
