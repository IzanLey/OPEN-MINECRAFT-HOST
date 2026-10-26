//! HTTP como en `legacy/omh.py`: peticiones con el User-Agent de omh, JSON
//! y descargas con progreso y verificacion SHA-1/SHA-256.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use anyhow::Context;
use reqwest::Method;
use reqwest::blocking::{Client, Response};
use reqwest::header::CONTENT_LENGTH;
use serde_json::Value;
use sha1::{Digest, Sha1};
use sha2::Sha256;

use crate::error::{OmhError, fail};
use crate::util::{MB, abspath, py_basename};

/// Timeouts por operacion (conectar y cada lectura), como los de urllib en Python.
pub const JSON_TIMEOUT: u64 = 30;
pub const DOWNLOAD_TIMEOUT: u64 = 60;

pub fn user_agent() -> String {
    format!(
        "{}/{} (https://github.com/{})",
        crate::APP_NAME,
        crate::APP_VERSION,
        crate::GITHUB_REPO
    )
}

static CLIENTS: LazyLock<Mutex<HashMap<u64, Client>>> = LazyLock::new(Default::default);
static JSON_CACHE: LazyLock<Mutex<HashMap<String, Arc<Value>>>> = LazyLock::new(Default::default);

fn client(timeout_secs: u64) -> anyhow::Result<Client> {
    let mut clients = CLIENTS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(client) = clients.get(&timeout_secs) {
        return Ok(client.clone());
    }
    let timeout = Duration::from_secs(timeout_secs);
    // En el cliente bloqueante `timeout` vale para cada operacion (enviar y cada
    // lectura), no para la descarga entera: reqwest lo espera aparte por cada
    // `read` (`Response::read`), como el timeout de socket de urllib. Por eso una
    // descarga grande y lenta no se corta (D10). El cliente bloqueante no expone
    // `read_timeout` (solo el asincrono), asi que aqui `.timeout` hace ese papel.
    let client = Client::builder()
        .user_agent(user_agent())
        .connect_timeout(timeout)
        .timeout(timeout)
        .build()
        .map_err(|err| fail(format!("No se pudo preparar la conexion HTTPS: {err}")))?;
    clients.insert(timeout_secs, client.clone());
    Ok(client)
}

fn network_error(err: &reqwest::Error) -> anyhow::Error {
    let reason = if err.is_timeout() {
        "tiempo de espera agotado".to_owned()
    } else {
        // El motivo util esta al final de la cadena (DNS, conexion rechazada, TLS...).
        let mut source: &dyn std::error::Error = err;
        while let Some(next) = source.source() {
            source = next;
        }
        source.to_string()
    };
    OmhError::Network(reason).into()
}

fn read_error(err: io::Error) -> anyhow::Error {
    let reason = if err.kind() == io::ErrorKind::TimedOut {
        "tiempo de espera agotado".to_owned()
    } else {
        err.to_string()
    };
    OmhError::Network(reason).into()
}

/// Peticion HTTP. Si el codigo no es 2xx (tras las redirecciones) devuelve un
/// error `Http`, como `HTTPError` de urllib.
pub fn request(
    method: Method,
    url: &str,
    headers: &[(&str, &str)],
    body: Option<Vec<u8>>,
    timeout_secs: u64,
) -> anyhow::Result<Response> {
    let mut builder = client(timeout_secs)?.request(method, url);
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    if let Some(body) = body {
        builder = builder.body(body);
    }
    let response = builder.send().map_err(|err| network_error(&err))?;
    let status = response.status();
    if !status.is_success() {
        return Err(OmhError::Http {
            code: status.as_u16(),
            url: response.url().to_string(),
        }
        .into());
    }
    Ok(response)
}

/// Cuerpo completo de una respuesta, leido a trozos (timeout por lectura).
pub fn read_body(mut response: Response) -> anyhow::Result<Vec<u8>> {
    let mut body = Vec::new();
    response.read_to_end(&mut body).map_err(read_error)?;
    Ok(body)
}

pub fn get_json(url: &str, headers: &[(&str, &str)]) -> anyhow::Result<Value> {
    let body = read_body(request(Method::GET, url, headers, None, JSON_TIMEOUT)?)?;
    serde_json::from_slice(&body).map_err(|err| {
        fail(format!(
            "La respuesta de {url} no es un JSON valido ({err})."
        ))
    })
}

/// `get_json` guardando la respuesta en memoria durante la ejecucion.
pub fn cached_json(url: &str) -> anyhow::Result<Arc<Value>> {
    let cached = JSON_CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(url)
        .cloned();
    if let Some(value) = cached {
        return Ok(value);
    }
    let value = Arc::new(get_json(url, &[])?);
    JSON_CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(url.to_owned(), Arc::clone(&value));
    Ok(value)
}

pub fn github_latest_release(repo: &str) -> anyhow::Result<Value> {
    get_json(
        &format!("https://api.github.com/repos/{repo}/releases/latest"),
        &[("Accept", "application/vnd.github+json")],
    )
}

/// Campo de texto obligatorio de una respuesta JSON.
pub fn str_field<'a>(value: &'a Value, key: &str) -> anyhow::Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| fail(format!("Respuesta inesperada del servidor: falta '{key}'.")))
}

/// Hashes publicados por la fuente; los que falten no se comprueban.
#[derive(Clone, Copy, Debug, Default)]
pub struct Checksums<'a> {
    pub sha1: Option<&'a str>,
    pub sha256: Option<&'a str>,
}

impl<'a> Checksums<'a> {
    pub fn sha1(hash: Option<&'a str>) -> Self {
        Checksums {
            sha1: hash,
            sha256: None,
        }
    }

    pub fn sha256(hash: Option<&'a str>) -> Self {
        Checksums {
            sha1: None,
            sha256: hash,
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn megabytes(bytes: u64) -> f64 {
    bytes as f64 / MB as f64
}

/// Descarga un archivo mostrando el progreso y verifica el hash si se conoce.
/// Escribe primero en `<destino>.part` y solo lo renombra si todo va bien.
pub fn download(
    url: &str,
    dest: &Path,
    sums: Checksums,
    label: Option<&str>,
) -> anyhow::Result<()> {
    if let Some(parent) = abspath(dest).parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("No se pudo crear {}", parent.display()))?;
    }
    let mut tmp = dest.as_os_str().to_owned();
    tmp.push(".part");
    let tmp = PathBuf::from(tmp);
    let label = label.map_or_else(
        || py_basename(&dest.to_string_lossy()).to_owned(),
        str::to_owned,
    );

    let mut response = request(Method::GET, url, &[], None, DOWNLOAD_TIMEOUT)?;
    let total = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let mut out =
        File::create(&tmp).with_context(|| format!("No se pudo crear {}", tmp.display()))?;
    let mut sha1 = Sha1::new();
    let mut sha256 = Sha256::new();
    let mut done: u64 = 0;
    let mut last: Option<Instant> = None;
    let mut buf = vec![0u8; 256 * 1024];
    let mut stdout = io::stdout();
    loop {
        let n = response.read(&mut buf).map_err(read_error)?;
        if n == 0 {
            break;
        }
        let chunk = &buf[..n];
        out.write_all(chunk)
            .with_context(|| format!("No se pudo escribir {}", tmp.display()))?;
        sha1.update(chunk);
        sha256.update(chunk);
        done += n as u64;
        if last.is_none_or(|at| at.elapsed() > Duration::from_millis(250)) {
            last = Some(Instant::now());
            let line = if let Some(percent) = (done * 100).checked_div(total) {
                format!(
                    "\r  Descargando {label}: {percent:3}% ({:.1}/{:.1} MB)",
                    megabytes(done),
                    megabytes(total)
                )
            } else {
                format!("\r  Descargando {label}: {:.1} MB", megabytes(done))
            };
            let _ = stdout.write_all(line.as_bytes());
            let _ = stdout.flush();
        }
    }
    let _ = write!(
        stdout,
        "\r  Descargando {label}: completado ({:.1} MB)          \n",
        megabytes(done)
    );
    let _ = stdout.flush();
    out.flush()
        .with_context(|| format!("No se pudo escribir {}", tmp.display()))?;
    drop(out);

    let checks = [
        ("SHA-1", sums.sha1, hex(&sha1.finalize())),
        ("SHA-256", sums.sha256, hex(&sha256.finalize())),
    ];
    for (name, expected, actual) in checks {
        if let Some(expected) = expected.filter(|hash| !hash.is_empty())
            && actual != expected.to_lowercase()
        {
            let _ = fs::remove_file(&tmp);
            return Err(fail(format!(
                "El hash {name} de {label} no coincide. Descarga cancelada."
            )));
        }
    }
    fs::rename(&tmp, dest).with_context(|| format!("No se pudo crear {}", dest.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{http_status, user_message};
    use std::io::{BufRead, BufReader};
    use std::net::TcpListener;
    use std::thread;

    /// Servidor HTTP minimo: responde una vez con `status` y `body`.
    fn serve_once(status: &'static str, body: &'static [u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(&stream);
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap() > 0 && line != "\r\n" {
                line.clear();
            }
            let head = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            (&stream).write_all(head.as_bytes()).unwrap();
            (&stream).write_all(body).unwrap();
        });
        format!("http://127.0.0.1:{port}/archivo.jar")
    }

    #[test]
    fn user_agent_como_python() {
        assert_eq!(
            user_agent(),
            format!(
                "open-mc-host/{} (https://github.com/IzanLey/OPEN-MINECRAFT-HOST)",
                crate::APP_VERSION
            )
        );
    }

    #[test]
    fn descarga_y_comprueba_el_hash() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("sub").join("server.jar");
        let url = serve_once("200 OK", b"hola");
        // sha1("hola") y sha256("hola")
        let sums = Checksums {
            sha1: Some("99800B85D3383E3A2FB45EB7D0066A4879A9DAD0"),
            sha256: Some("b221d9dbb083a7f33428d7c2a3c3198ae925614d70210e28716ccaa7cd4ddb79"),
        };
        download(&url, &dest, sums, None).unwrap();
        assert_eq!(fs::read(&dest).unwrap(), b"hola");
        assert!(!tmp.path().join("sub").join("server.jar.part").exists());
    }

    #[test]
    fn hash_distinto_cancela_y_borra_el_part() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("server.jar");
        let url = serve_once("200 OK", b"hola");
        let err = download(
            &url,
            &dest,
            Checksums::sha256(Some("00")),
            Some("paper 1.21.1"),
        )
        .unwrap_err();
        assert_eq!(
            user_message(&err),
            "El hash SHA-256 de paper 1.21.1 no coincide. Descarga cancelada."
        );
        assert!(!dest.exists());
        assert!(!tmp.path().join("server.jar.part").exists());
    }

    #[test]
    fn error_http_como_urllib() {
        let tmp = tempfile::tempdir().unwrap();
        let url = serve_once("404 Not Found", b"no");
        let err =
            download(&url, &tmp.path().join("x.jar"), Checksums::default(), None).unwrap_err();
        assert_eq!(http_status(&err), Some(404));
        assert_eq!(
            user_message(&err),
            format!("El servidor remoto respondio 404 ({url}).")
        );
        assert!(
            !tmp.path().join("x.jar.part").exists(),
            "sin respuesta valida no se crea nada"
        );
    }

    #[test]
    fn json_no_valido_da_error_claro() {
        let url = serve_once("200 OK", b"<html>");
        let err = get_json(&url, &[]).unwrap_err();
        assert!(
            user_message(&err).starts_with(&format!("La respuesta de {url} no es un JSON valido"))
        );
    }

    #[test]
    fn sin_servidor_es_error_de_red() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let err = get_json(&format!("http://127.0.0.1:{port}/"), &[]).unwrap_err();
        assert!(user_message(&err).starts_with("Error de red: "));
        assert!(user_message(&err).ends_with(". Comprueba tu conexion a internet."));
    }
}
