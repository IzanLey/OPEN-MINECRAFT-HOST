//! `omh network` (PASO 3): modo, puerto, IP, whitelist y administradores; IPs,
//! puertos libres y direcciones del servidor.

pub mod tunnel;
pub mod upnp;

use std::collections::HashSet;
use std::net::{Ipv4Addr, TcpListener, UdpSocket};
use std::sync::LazyLock;

use regex::Regex;
use reqwest::Method;
use serde_json::Map;

use crate::catalog::default_port;
use crate::cli::NetworkArgs;
use crate::config::{self, Network, NetworkConfig, RUN_DIR, ServerConfig};
use crate::error::fail;
use crate::http;
use crate::platform::IS_WIN;
use crate::players::{self, Kind};
use crate::runner::control::is_running;
use crate::ui;
use crate::util::{py_strip, tail_lines, valid_player};

static CLAIM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https://playit\.gg/claim/\S+").expect("expresion regular fija y probada")
});

/// IP de este equipo en la red local (sin enviar nada).
pub fn lan_ip() -> String {
    let probe = || -> std::io::Result<String> {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
        socket.connect(("10.255.255.255", 1))?;
        Ok(socket.local_addr()?.ip().to_string())
    };
    probe().unwrap_or_else(|_| "127.0.0.1".to_owned())
}

/// IP publica segun api.ipify.org (`None` si no hay respuesta).
pub fn public_ip() -> Option<String> {
    let response = http::request(Method::GET, "https://api.ipify.org", &[], None, 10).ok()?;
    let text = String::from_utf8(http::read_body(response).ok()?).ok()?;
    Some(py_strip(&text).to_owned())
}

/// Si se puede abrir el puerto en todas las interfaces. En TCP se usa SO_REUSEADDR
/// en Linux (B3), como Java al abrir el puerto del servidor.
pub fn port_free(port: u16, udp: bool) -> bool {
    let address = (Ipv4Addr::UNSPECIFIED, port);
    if udp {
        UdpSocket::bind(address).is_ok()
    } else {
        TcpListener::bind(address).is_ok()
    }
}

pub fn find_free_port(
    start: u16,
    udp: bool,
    exclude: &HashSet<u16>,
    need_next: bool,
) -> anyhow::Result<u16> {
    for port in (start..=u16::MAX).take(500) {
        let next = port.checked_add(1);
        let next_excluded = need_next && next.is_none_or(|next| exclude.contains(&next));
        if exclude.contains(&port) || next_excluded {
            continue;
        }
        let next_free = !need_next || next.is_some_and(|next| port_free(next, udp));
        if port_free(port, udp) && next_free {
            return Ok(port);
        }
    }
    Err(fail(format!(
        "No se encontro ningun puerto libre a partir de {start}."
    )))
}

/// IP valida para `socket.inet_aton`: 1 a 4 partes en decimal, octal o hexadecimal.
pub fn inet_aton_ok(text: &str) -> bool {
    let parts: Vec<&str> = text.split('.').collect();
    if parts.len() > 4 {
        return false;
    }
    let count = parts.len();
    parts.iter().enumerate().all(|(i, part)| {
        let value = if let Some(hex) = part.strip_prefix("0x").or_else(|| part.strip_prefix("0X")) {
            if hex.is_empty() {
                Some(0)
            } else {
                u64::from_str_radix(hex, 16)
                    .ok()
                    .filter(|_| hex.chars().all(|c| c.is_ascii_hexdigit()))
            }
        } else if !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()) {
            if part.len() > 1 && part.starts_with('0') {
                u64::from_str_radix(&part[1..], 8).ok()
            } else {
                part.parse().ok()
            }
        } else {
            None
        };
        let max = if i + 1 == count {
            (1u64 << (8 * (5 - count))) - 1
        } else {
            255
        };
        value.is_some_and(|value| value <= max)
    })
}

pub fn on_off(value: &str) -> bool {
    matches!(
        value.to_lowercase().as_str(),
        "on" | "true" | "si" | "s" | "yes" | "1"
    )
}

/// Escribe en server.properties las claves de red de cada edicion.
pub fn apply_network_properties(cfg: &ServerConfig) -> anyhow::Result<()> {
    let Some(n) = cfg.network() else {
        return Err(fail("Configura antes la red con: omh network"));
    };
    let tf = |value: bool| if value { "true" } else { "false" }.to_owned();
    let oo = |value: bool| if value { "on" } else { "off" }.to_owned();
    let values: Vec<(&str, String)> = if cfg.is_java() {
        vec![
            ("server-port", n.port.to_string()),
            ("server-ip", n.bind_ip.clone()),
            ("white-list", tf(n.whitelist)),
            ("enforce-whitelist", tf(n.whitelist)),
            ("online-mode", tf(n.online_mode)),
            ("max-players", n.max_players.to_string()),
            ("motd", n.motd.clone()),
        ]
    } else if cfg.core == "vanilla" {
        vec![
            ("server-port", n.port.to_string()),
            (
                "server-portv6",
                n.port_v6
                    .map_or_else(|| "None".to_owned(), |p| p.to_string()),
            ),
            ("allow-list", tf(n.whitelist)),
            ("online-mode", tf(n.online_mode)),
            ("max-players", n.max_players.to_string()),
            ("server-name", n.motd.clone()),
        ]
    } else {
        let bind = if n.bind_ip.is_empty() {
            "0.0.0.0"
        } else {
            n.bind_ip.as_str()
        };
        vec![
            ("server-port", n.port.to_string()),
            ("server-ip", bind.to_owned()),
            ("white-list", oo(n.whitelist)),
            ("xbox-auth", oo(n.online_mode)),
            ("max-players", n.max_players.to_string()),
            ("motd", n.motd.clone()),
        ]
    };
    config::set_properties(&cfg.dir.join("server.properties"), &values)
}

/// Lineas con las direcciones para conectarse.
pub fn server_addresses(cfg: &ServerConfig) -> Vec<String> {
    let Some(n) = cfg.network() else {
        return Vec::new();
    };
    let port = n.port;
    let mut lines = vec![format!("Este PC:        127.0.0.1:{port}")];
    if n.mode == "lan" || n.mode == "public" {
        lines.push(format!("Red local (LAN): {}:{port}", lan_ip()));
    }
    if n.mode == "public" {
        if n.public_method.as_deref() == Some("playit") {
            let log = tail_lines(&cfg.dir.join(RUN_DIR).join("playit.log"), 20).join("\n");
            if let Some(claim) = CLAIM.find(&log) {
                // B12: el tipo de tunel segun la edicion (Python ponia siempre "Java").
                let kind = if cfg.is_java() { "Java" } else { "Bedrock" };
                lines.push(format!(
                    "Internet:       NO DISPONIBLE. playit.gg sin vincular: abre {}",
                    claim.as_str()
                ));
                lines.push(format!(
                    "                y crea un tunel Minecraft {kind} hacia 127.0.0.1:{port}"
                ));
            } else {
                lines.push(
                    "Internet:       la direccion de tu tunel en https://playit.gg/account/tunnels"
                        .to_owned(),
                );
            }
        } else {
            let ip = public_ip().unwrap_or_else(|| "IP-PUBLICA".to_owned());
            lines.push(format!("Internet:       {ip}:{port}"));
        }
    }
    lines
}

/// Enlace de vinculacion de playit.gg en el texto (si aun no esta vinculado).
pub fn playit_claim(text: &str) -> Option<String> {
    CLAIM.find(text).map(|m| m.as_str().to_owned())
}

fn given(value: &Option<String>) -> Option<&str> {
    value.as_deref().filter(|value| !value.is_empty())
}

pub fn cmd_network(args: &NetworkArgs) -> anyhow::Result<()> {
    let mut cfg = config::load_config(&config::resolve_dir(args.dir.as_deref())?)?;
    config::require_step(&cfg, 2)?;
    ui::header(&format!("PASO 3 de 3 - Red ({})", cfg.name));
    let old = cfg.network().cloned();
    let old_admins = cfg.admins().to_vec();
    let udp = cfg.edition == "bedrock";
    let bedrock_vanilla = cfg.is_bedrock_vanilla();

    let old_mode = old.as_ref().map_or("lan", |n| n.mode.as_str()).to_owned();
    let mode = match given(&args.mode) {
        Some(mode) => mode.to_owned(),
        None => ui::ask("Modo de red", Some(&old_mode), &["local", "lan", "public"])?,
    }
    .to_lowercase();
    if !["local", "lan", "public"].contains(&mode.as_str()) {
        return Err(fail(format!(
            "Modo no valido: {mode} (local, lan o public)"
        )));
    }
    let mut method = None;
    if mode == "public" {
        ui::info("Metodos para abrirlo a internet:");
        ui::info("  playit  Tunel gratuito de playit.gg. Te da una direccion publica gratis y no");
        ui::info("          necesitas tocar el router. Recomendado.");
        ui::info(
            "  upnp    Abre el puerto en tu router automaticamente (si el router lo permite).",
        );
        ui::info("  manual  Abres tu el puerto en el router. Se usa tu IP publica.");
        let old_method = old
            .as_ref()
            .and_then(|n| n.public_method.clone())
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| "playit".to_owned());
        let chosen = match given(&args.public_method) {
            Some(method) => method.to_owned(),
            None => ui::ask("Metodo", Some(&old_method), &["playit", "upnp", "manual"])?,
        };
        method = Some(chosen.to_lowercase());
    }

    let exclude = config::other_ports(&cfg);
    let port_arg = match given(&args.port) {
        Some(port) => port.to_owned(),
        None => ui::ask("Puerto (numero o auto)", Some("auto"), &[])?,
    }
    .to_lowercase();
    let port = if port_arg == "auto" {
        let port = find_free_port(
            default_port(&cfg.edition),
            udp,
            &exclude,
            udp && bedrock_vanilla,
        )?;
        ui::info(&format!("Puerto libre encontrado: {port}"));
        port
    } else {
        let parsed = (!port_arg.is_empty() && port_arg.chars().all(|c| c.is_ascii_digit()))
            .then(|| port_arg.parse::<u16>().ok())
            .flatten()
            .filter(|port| *port >= 1024);
        let Some(port) = parsed else {
            return Err(fail(format!(
                "Puerto no valido: {port_arg} (usa 1024-65535 o auto)"
            )));
        };
        if exclude.contains(&port) {
            return Err(fail(format!(
                "El puerto {port} ya lo usa otro servidor de omh."
            )));
        }
        if !port_free(port, udp) && !is_running(&cfg.dir) {
            return Err(fail(format!(
                "El puerto {port} esta ocupado. Usa --port auto."
            )));
        }
        port
    };

    let bind_ip = match args.ip.as_deref() {
        Some(ip) if !ip.is_empty() && ip != "auto" => ip.to_owned(),
        _ => if mode == "local" { "127.0.0.1" } else { "" }.to_owned(),
    };
    if !bind_ip.is_empty() && !inet_aton_ok(&bind_ip) {
        return Err(fail(format!("IP no valida: {bind_ip}")));
    }
    let whitelist = on_off(&match given(&args.whitelist) {
        Some(value) => value.to_owned(),
        None => ui::ask(
            "Activar whitelist",
            Some(if mode == "public" { "on" } else { "off" }),
            &["on", "off"],
        )?,
    });
    let online_raw = given(&args.online_mode)
        .map(str::to_owned)
        .or_else(|| {
            old.as_ref()
                .map(|n| n.online_mode_raw.clone())
                .filter(|raw| !raw.is_empty())
        })
        .unwrap_or_else(|| "on".to_owned());
    let online_mode = on_off(&online_raw);
    if mode == "public" && !online_mode {
        ui::warn(
            "online-mode desactivado en un servidor publico: cualquiera puede entrar con cualquier nombre.",
        );
    }
    let requested_admins = if args.admin.is_empty() {
        let answer = ui::ask(
            "Administrador (nombre de jugador, vacio para ninguno)",
            Some(""),
            &[],
        )?;
        if answer.is_empty() {
            Vec::new()
        } else {
            vec![answer]
        }
    } else {
        args.admin.clone()
    };
    let new_admins = requested_admins
        .iter()
        .map(|name| valid_player(name))
        .collect::<anyhow::Result<Vec<String>>>()?;
    let mut admins: Vec<String> = Vec::new();
    for name in old_admins.iter().chain(&new_admins) {
        if !admins.contains(name) {
            admins.push(name.clone());
        }
    }
    admins.sort_by_key(|name| name.to_lowercase());

    // B2: un --max-players 0 explicito se respeta.
    let max_players = args
        .max_players
        .or_else(|| old.as_ref().map(|n| n.max_players))
        .unwrap_or(20);
    let motd = given(&args.motd)
        .map(str::to_owned)
        .or_else(|| {
            old.as_ref()
                .map(|n| n.motd.clone())
                .filter(|motd| !motd.is_empty())
        })
        .unwrap_or_else(|| format!("Servidor {} - open-mc-host", cfg.name));
    cfg.network = Some(NetworkConfig::Full(Network {
        mode: mode.clone(),
        public_method: method.clone(),
        port,
        port_v6: (udp && bedrock_vanilla).then(|| port.saturating_add(1)),
        bind_ip,
        whitelist,
        online_mode,
        online_mode_raw: if online_mode { "on" } else { "off" }.to_owned(),
        max_players,
        motd,
        admins,
        extra: Map::new(),
    }));
    apply_network_properties(&cfg)?;
    cfg.step = 3;
    config::save_config(&cfg)?;
    for name in &new_admins {
        players::add_player(&mut cfg, Kind::Admin, name, false)?;
        if whitelist {
            players::add_player(&mut cfg, Kind::Whitelist, name, false)?;
        }
    }

    let protocol = if udp { "UDP" } else { "TCP" };
    let method = method.as_deref();
    if mode == "local" {
        ui::info("Solo podras conectarte desde este PC.");
        if bedrock_vanilla {
            ui::info(
                "Bedrock vanilla escucha en todas las interfaces; tu firewall decide quien entra.",
            );
        }
    }
    if method == Some("playit") {
        tunnel::ensure_playit()?;
        ui::info("Ejecuta 'omh tunnel' una vez para vincular playit.gg con tu cuenta gratuita y");
        ui::info(&format!(
            "crear el tunel (tipo Minecraft {}, local 127.0.0.1:{port}).",
            if udp { "Bedrock" } else { "Java" }
        ));
        ui::info("Despues 'omh start' lanzara el tunel automaticamente junto al servidor.");
    }
    if method == Some("upnp") {
        ui::info("Buscando router con UPnP...");
        if upnp::discover().is_some() {
            ui::ok(&format!(
                "Router UPnP encontrado. El puerto {port}/{protocol} se abrira al iniciar y se cerrara al parar."
            ));
        } else {
            ui::warn(
                "Tu router no responde a UPnP. Activalo en el router, o usa --public-method playit.",
            );
        }
    }
    if method == Some("manual") {
        ui::info(&format!(
            "Abre en tu router el puerto {port} ({protocol}) hacia la IP {}.",
            lan_ip()
        ));
    }
    if mode == "lan" || mode == "public" {
        ui::info(
            "Si el firewall bloquea las conexiones, permite el puerto (requiere administrador):",
        );
        if IS_WIN {
            ui::info(&format!(
                "  netsh advfirewall firewall add rule name=\"Minecraft {port}\" dir=in action=allow protocol={protocol} localport={port}"
            ));
        } else {
            ui::info(&format!(
                "  sudo ufw allow {port}/{}",
                protocol.to_lowercase()
            ));
        }
    }
    ui::ok(&format!(
        "Red configurada: modo {mode}{} | puerto {port} | whitelist {}",
        method.map(|m| format!(" ({m})")).unwrap_or_default(),
        if whitelist { "on" } else { "off" }
    ));
    for line in server_addresses(&cfg) {
        ui::info(&format!("  {line}"));
    }
    print!(
        "\n  Todo listo. Inicia el servidor con:\n    omh start            (en esta terminal, puedes escribir comandos)\n    omh start --detach   (en segundo plano; luego: omh console, omh stop)\n\n"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn puertos_libres_y_excluidos() {
        let busy = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
        let port = busy.local_addr().unwrap().port();
        assert!(!port_free(port, false));
        let found = find_free_port(port, false, &HashSet::new(), false).unwrap();
        assert!(found > port);
        let exclude: HashSet<u16> = [found].into();
        assert_ne!(find_free_port(port, false, &exclude, false).unwrap(), found);
        let udp = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
        let uport = udp.local_addr().unwrap().port();
        assert!(!port_free(uport, true));
        // need_next: el siguiente tampoco puede estar ocupado.
        let p = find_free_port(uport - 1, true, &HashSet::new(), true).unwrap();
        assert!(p != uport - 1 && p != uport);
    }

    #[test]
    fn ips_como_inet_aton() {
        for ok in [
            "127.0.0.1",
            "0.0.0.0",
            "127.1",
            "10",
            "0x7f.1",
            "192.168.1.255",
            "017.0.0.1",
        ] {
            assert!(inet_aton_ok(ok), "{ok}");
        }
        for bad in [
            "",
            "256.0.0.1",
            "1.2.3.4.5",
            "a.b.c.d",
            "1.2.3.",
            "::1",
            "08.0.0.1",
        ] {
            assert!(!inet_aton_ok(bad), "{bad}");
        }
    }

    #[test]
    fn on_off_como_python() {
        for yes in ["on", "ON", "true", "si", "s", "yes", "1"] {
            assert!(on_off(yes));
        }
        for no in ["off", "no", "0", "S\u{ed}"] {
            assert!(!on_off(no));
        }
    }

    #[test]
    fn enlace_de_vinculacion() {
        assert_eq!(
            playit_claim("x\nVisit link to setup https://playit.gg/claim/abc123 now").as_deref(),
            Some("https://playit.gg/claim/abc123")
        );
        assert_eq!(playit_claim("tunnel running, 1 tunnels registered"), None);
    }
}
