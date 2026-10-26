//! Whitelist y administradores: `omh whitelist`, `omh admin` y los archivos de
//! jugadores de cada core.

use std::fs;
use std::path::{Path, PathBuf};

use md5::{Digest, Md5};
use serde_json::{Map, Value};

use crate::cli::{AdminArgs, WhitelistArgs};
use crate::config::{self, ServerConfig};
use crate::error::{OmhError, fail};
use crate::http;
use crate::network::apply_network_properties;
use crate::pyjson;
use crate::runner::control::{is_running, send_command};
use crate::ui;
use crate::util::{py_quote, py_strip, read_text_lossy, text_lines, valid_player, write_text};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Whitelist,
    Admin,
}

/// UUID de una cuenta: el de Mojang con online-mode y el "offline" si no.
pub fn player_uuid(name: &str, online: bool) -> anyhow::Result<String> {
    let raw = if online {
        let url = format!(
            "https://api.mojang.com/users/profiles/minecraft/{}",
            py_quote(name, "/")
        );
        let missing = || {
            fail(format!(
                "No existe una cuenta de Minecraft llamada '{name}'."
            ))
        };
        match http::get_json(&url, &[]) {
            Ok(data) => data
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(missing)?,
            // B7: sin conexion se dice que es un error de red, no que la cuenta no existe.
            Err(err) if matches!(err.downcast_ref::<OmhError>(), Some(OmhError::Network(_))) => {
                return Err(err);
            }
            Err(_) => return Err(missing()),
        }
    } else {
        let mut digest = [0u8; 16];
        digest.copy_from_slice(&Md5::digest(format!("OfflinePlayer:{name}").as_bytes()));
        digest[6] = (digest[6] & 0x0F) | 0x30;
        digest[8] = (digest[8] & 0x3F) | 0x80;
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    };
    let at = |a: usize, b: usize| raw.get(a..b).unwrap_or_default();
    Ok(format!(
        "{}-{}-{}-{}-{}",
        at(0, 8),
        at(8, 12),
        at(12, 16),
        at(16, 20),
        raw.get(20..).unwrap_or_default()
    ))
}

fn read_json_list(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
}

fn write_json_list(path: &Path, entries: &[Value]) -> anyhow::Result<()> {
    write_text(path, &pyjson::to_pretty(entries)?)
}

fn read_txt_list(path: &Path) -> Vec<String> {
    let Ok(text) = read_text_lossy(path) else {
        return Vec::new();
    };
    text_lines(&text)
        .into_iter()
        .filter(|line| !py_strip(line).is_empty() && !line.starts_with('#'))
        .map(|line| py_strip(line).to_owned())
        .collect()
}

fn write_txt_list(path: &Path, names: &[String]) -> anyhow::Result<()> {
    let mut text = names.join("\n");
    if !names.is_empty() {
        text.push('\n');
    }
    write_text(path, &text)
}

/// Archivo donde cada core guarda la whitelist o los administradores.
fn list_file(cfg: &ServerConfig, kind: Kind) -> PathBuf {
    let name = match (cfg.is_java(), cfg.is_bedrock_vanilla(), kind) {
        (true, _, Kind::Whitelist) => "whitelist.json",
        (true, _, Kind::Admin) => "ops.json",
        (false, true, Kind::Whitelist) => "allowlist.json",
        (false, true, Kind::Admin) => "permissions.json",
        (false, false, Kind::Whitelist) => "white-list.txt",
        (false, false, Kind::Admin) => "ops.txt",
    };
    cfg.dir.join(name)
}

fn console_name(name: &str) -> String {
    if name.contains(' ') {
        format!("\"{name}\"")
    } else {
        name.to_owned()
    }
}

fn entry_name(entry: &Value) -> String {
    entry
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_lowercase()
}

pub fn add_player(
    cfg: &mut ServerConfig,
    kind: Kind,
    name: &str,
    remove: bool,
) -> anyhow::Result<()> {
    let name = valid_player(name)?;
    let lower = name.to_lowercase();
    if kind == Kind::Admin {
        let admins = cfg.admins_mut();
        if remove {
            admins.retain(|admin| admin.to_lowercase() != lower);
        } else if !admins.iter().any(|admin| admin.to_lowercase() == lower) {
            admins.push(name.clone());
        }
        config::save_config(cfg)?;
    }

    let bedrock_vanilla = cfg.is_bedrock_vanilla();
    if is_running(&cfg.dir) {
        let command = match kind {
            Kind::Whitelist => format!(
                "{} {} {}",
                if bedrock_vanilla {
                    "allowlist"
                } else {
                    "whitelist"
                },
                if remove { "remove" } else { "add" },
                console_name(&name)
            ),
            Kind::Admin => format!(
                "{} {}",
                if remove { "deop" } else { "op" },
                console_name(&name)
            ),
        };
        send_command(&cfg.dir, &command);
        ui::ok(&format!("Enviado al servidor: {command}"));
        return Ok(());
    }

    let path = list_file(cfg, kind);
    let online = cfg.network().is_none_or(|network| network.online_mode);
    if cfg.is_java() {
        let mut entries: Vec<Value> = read_json_list(&path)
            .into_iter()
            .filter(|entry| entry_name(entry) != lower)
            .collect();
        if !remove {
            let mut entry = Map::new();
            entry.insert("uuid".into(), player_uuid(&name, online)?.into());
            entry.insert("name".into(), name.clone().into());
            if kind == Kind::Admin {
                entry.insert("level".into(), 4.into());
                entry.insert("bypassesPlayerLimit".into(), false.into());
            }
            entries.push(Value::Object(entry));
        }
        write_json_list(&path, &entries)?;
    } else if bedrock_vanilla {
        if kind == Kind::Whitelist {
            let mut entries: Vec<Value> = read_json_list(&path)
                .into_iter()
                .filter(|entry| entry_name(entry) != lower)
                .collect();
            if !remove {
                let mut entry = Map::new();
                entry.insert("ignoresPlayerLimit".into(), false.into());
                entry.insert("name".into(), name.clone().into());
                entries.push(Value::Object(entry));
            }
            write_json_list(&path, &entries)?;
        } else if !remove {
            ui::info(&format!(
                "En Bedrock vanilla el rango de admin se aplica cuando {name} entre al servidor."
            ));
        }
    } else {
        let mut names: Vec<String> = read_txt_list(&path)
            .into_iter()
            .filter(|n| n.to_lowercase() != lower)
            .collect();
        if !remove {
            names.push(lower.clone());
        }
        write_txt_list(&path, &names)?;
    }
    ui::ok(&format!(
        "{name} {} {}.",
        if remove { "eliminado de" } else { "anadido a" },
        if kind == Kind::Whitelist {
            "la whitelist"
        } else {
            "los administradores"
        }
    ));
    Ok(())
}

pub fn show_list(cfg: &ServerConfig, kind: Kind) {
    let path = list_file(cfg, kind);
    let mut names: Vec<String> = if path.extension().is_some_and(|ext| ext == "json") {
        read_json_list(&path)
            .iter()
            .map(|entry| {
                entry
                    .get("name")
                    .filter(|name| crate::util::py_truthy(name))
                    .or_else(|| entry.get("xuid"))
                    .and_then(Value::as_str)
                    .unwrap_or("?")
                    .to_owned()
            })
            .collect()
    } else {
        read_txt_list(&path)
    };
    if kind == Kind::Admin {
        for admin in cfg.admins() {
            names.push(admin.clone());
        }
        let mut unique: Vec<String> = Vec::new();
        for name in names {
            if !unique.contains(&name) {
                unique.push(name);
            }
        }
        unique.sort_by_key(|name| name.to_lowercase());
        names = unique;
    }
    let label = if kind == Kind::Whitelist {
        "Whitelist"
    } else {
        "Administradores"
    };
    let list = if names.is_empty() {
        "(vacia)".to_owned()
    } else {
        names.join(", ")
    };
    ui::info(&format!("{label}: {list}"));
}

pub fn cmd_whitelist(args: &WhitelistArgs) -> anyhow::Result<()> {
    let mut cfg = config::load_config(&config::resolve_dir(args.dir.as_deref())?)?;
    config::require_step(&cfg, 1)?;
    let action = args.action.as_str();
    if action == "list" {
        show_list(&cfg, Kind::Whitelist);
        return Ok(());
    }
    if action == "on" || action == "off" {
        // B1: una red a medias (solo admins) cuenta como sin configurar.
        let Some(network) = cfg.network_mut() else {
            return Err(fail("Configura antes la red con: omh network"));
        };
        network.whitelist = action == "on";
        config::save_config(&cfg)?;
        apply_network_properties(&cfg)?;
        if is_running(&cfg.dir) {
            let base = if cfg.is_bedrock_vanilla() {
                "allowlist"
            } else {
                "whitelist"
            };
            send_command(&cfg.dir, &format!("{base} {action}"));
        }
        ui::ok(&format!("Whitelist {action}."));
        return Ok(());
    }
    if args.player.is_empty() {
        return Err(fail("Indica el jugador. Ejemplo: omh whitelist add Steve"));
    }
    for player in &args.player {
        add_player(&mut cfg, Kind::Whitelist, player, action == "remove")?;
    }
    Ok(())
}

pub fn cmd_admin(args: &AdminArgs) -> anyhow::Result<()> {
    let mut cfg = config::load_config(&config::resolve_dir(args.dir.as_deref())?)?;
    config::require_step(&cfg, 1)?;
    if args.action == "list" {
        show_list(&cfg, Kind::Admin);
        return Ok(());
    }
    if args.player.is_empty() {
        return Err(fail("Indica el jugador. Ejemplo: omh admin add Steve"));
    }
    for player in &args.player {
        add_player(&mut cfg, Kind::Admin, player, args.action == "remove")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Valores de player_uuid(nombre, False) en legacy/omh.py.
    #[test]
    fn uuid_offline_como_python() {
        assert_eq!(
            player_uuid("Notch", false).unwrap(),
            "b50ad385-829d-3141-a216-7e7d7539ba7f"
        );
        assert_eq!(
            player_uuid("Steve", false).unwrap(),
            "5627dd98-e6be-3c21-b8a8-e92344183641"
        );
        assert_eq!(
            player_uuid("jeb_", false).unwrap(),
            "a762f560-4fce-3236-812a-b80efff0b62b"
        );
        assert_eq!(
            player_uuid("Steve Jobs", false).unwrap(),
            "5f6dc8b1-3767-3ee6-bc43-ce405247103a"
        );
    }

    #[test]
    fn listas_de_cada_core() {
        let tmp = tempfile::tempdir().unwrap();
        let partial = r#"{"name": "b", "edition": "java", "core": "paper", "version": "1.21.1", "step": 1,
            "network": {"admins": ["Alex"]}}"#;
        let mut cfg: ServerConfig = serde_json::from_str(partial).unwrap();
        cfg.dir = tmp.path().to_path_buf();
        // Sin red completa se usa online-mode activo; aqui se fuerza offline cambiando
        // a un core de texto para no depender de internet.
        cfg.edition = "bedrock".into();
        cfg.core = "pocketmine".into();
        add_player(&mut cfg, Kind::Admin, "Steve", false).unwrap();
        add_player(&mut cfg, Kind::Whitelist, "Steve Jobs", false).unwrap();
        assert_eq!(read_txt_list(&tmp.path().join("ops.txt")), ["steve"]);
        assert_eq!(
            read_txt_list(&tmp.path().join("white-list.txt")),
            ["steve jobs"]
        );
        assert_eq!(cfg.admins(), ["Alex", "Steve"]);
        add_player(&mut cfg, Kind::Admin, "STEVE", true).unwrap();
        assert!(read_txt_list(&tmp.path().join("ops.txt")).is_empty());
        assert_eq!(cfg.admins(), ["Alex"]);
    }
}
