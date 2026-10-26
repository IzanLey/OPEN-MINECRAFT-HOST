//! UPnP: buscar el router (SSDP) y abrir o cerrar puertos con SOAP.

use std::net::{Ipv4Addr, UdpSocket};
use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;
use reqwest::{Method, Url};

use super::lan_ip;
use crate::http;

const SERVICES: [&str; 3] = [
    "urn:schemas-upnp-org:service:WANIPConnection:1",
    "urn:schemas-upnp-org:service:WANIPConnection:2",
    "urn:schemas-upnp-org:service:WANPPPConnection:1",
];

fn regex(pattern: &str) -> Regex {
    Regex::new(pattern).expect("expresion regular fija y probada")
}

static LOCATION: LazyLock<Regex> = LazyLock::new(|| regex(r"(?im)^location:\s*(\S+)"));
static SERVICE: LazyLock<Regex> = LazyLock::new(|| regex(r"(?s)<service>(.*?)</service>"));
static CONTROL: LazyLock<Regex> =
    LazyLock::new(|| regex(r"(?s)<controlURL>\s*(.*?)\s*</controlURL>"));
static EXTERNAL_IP: LazyLock<Regex> =
    LazyLock::new(|| regex(r"<NewExternalIPAddress>(.*?)</NewExternalIPAddress>"));

/// Router con UPnP: URL de control y servicio.
#[derive(Clone, Debug)]
pub struct Gateway {
    pub control: String,
    pub service: String,
}

/// Busca un router durante 3 s (tarda al menos eso).
pub fn discover() -> Option<Gateway> {
    let message = [
        "M-SEARCH * HTTP/1.1",
        "HOST: 239.255.255.250:1900",
        "MAN: \"ssdp:discover\"",
        "MX: 2",
        "ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1",
        "",
        "",
    ]
    .join("\r\n");
    let mut locations: Vec<String> = Vec::new();
    if let Ok(socket) = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)) {
        let _ = socket.set_read_timeout(Some(Duration::from_secs(3)));
        if socket
            .send_to(message.as_bytes(), ("239.255.255.250", 1900))
            .is_ok()
        {
            let mut buf = vec![0u8; 65507];
            while let Ok((n, _)) = socket.recv_from(&mut buf) {
                let text = String::from_utf8_lossy(&buf[..n]);
                if let Some(location) = LOCATION.captures(&text).and_then(|c| c.get(1)) {
                    let location = location.as_str().to_owned();
                    if !locations.contains(&location) {
                        locations.push(location);
                    }
                }
            }
        }
    }
    for location in locations {
        let Ok(response) = http::request(Method::GET, &location, &[], None, 5) else {
            continue;
        };
        let Ok(body) = http::read_body(response) else {
            continue;
        };
        let xml = String::from_utf8_lossy(&body);
        for block in SERVICE.captures_iter(&xml).filter_map(|c| c.get(1)) {
            for service in SERVICES {
                if !block.as_str().contains(service) {
                    continue;
                }
                let control = CONTROL.captures(block.as_str()).and_then(|c| c.get(1));
                if let Some(control) = control
                    && let Ok(url) =
                        Url::parse(&location).and_then(|base| base.join(control.as_str()))
                {
                    return Some(Gateway {
                        control: url.to_string(),
                        service: service.to_owned(),
                    });
                }
            }
        }
    }
    None
}

fn soap(gateway: &Gateway, action: &str, arguments: &[(&str, String)]) -> anyhow::Result<String> {
    let args: String = arguments
        .iter()
        .map(|(name, value)| format!("<{name}>{value}</{name}>"))
        .collect();
    let body = format!(
        "<?xml version=\"1.0\"?><s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" \
         s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"><s:Body>\
         <u:{action} xmlns:u=\"{}\">{args}</u:{action}></s:Body></s:Envelope>",
        gateway.service
    );
    let soap_action = format!("\"{}#{action}\"", gateway.service);
    let headers = [
        ("Content-Type", "text/xml; charset=\"utf-8\""),
        ("SOAPAction", soap_action.as_str()),
    ];
    let response = http::request(
        Method::POST,
        &gateway.control,
        &headers,
        Some(body.into_bytes()),
        10,
    )?;
    Ok(String::from_utf8_lossy(&http::read_body(response)?).into_owned())
}

pub fn add(gateway: &Gateway, port: u16, protocol: &str, description: &str) -> anyhow::Result<()> {
    soap(
        gateway,
        "AddPortMapping",
        &[
            ("NewRemoteHost", String::new()),
            ("NewExternalPort", port.to_string()),
            ("NewProtocol", protocol.to_owned()),
            ("NewInternalPort", port.to_string()),
            ("NewInternalClient", lan_ip()),
            ("NewEnabled", "1".to_owned()),
            ("NewPortMappingDescription", description.to_owned()),
            ("NewLeaseDuration", "0".to_owned()),
        ],
    )
    .map(|_| ())
}

pub fn delete(gateway: &Gateway, port: u16, protocol: &str) {
    let _ = soap(
        gateway,
        "DeletePortMapping",
        &[
            ("NewRemoteHost", String::new()),
            ("NewExternalPort", port.to_string()),
            ("NewProtocol", protocol.to_owned()),
        ],
    );
}

pub fn external_ip(gateway: &Gateway) -> Option<String> {
    let reply = soap(gateway, "GetExternalIPAddress", &[]).ok()?;
    EXTERNAL_IP
        .captures(&reply)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_owned())
}
