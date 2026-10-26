//! Canal de control: `run.json` y el socket local con token por el que `omh stop`,
//! `omh cmd`, `omh console`, `omh status`... hablan con el supervisor.
//!
//! El protocolo es el mismo que el de la version Python (una linea JSON por peticion
//! y por respuesta), asi que las dos versiones pueden controlarse entre si.

use std::fs;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Map, Value};

use crate::config::RUN_DIR;
use crate::pyjson;
use crate::util::py_truthy;

pub fn run_file(dir: &Path) -> PathBuf {
    dir.join(RUN_DIR).join("run.json")
}

/// Envia una peticion al supervisor del servidor de `dir`. `None` si no hay
/// supervisor o no responde.
pub fn control_request(
    dir: &Path,
    action: &str,
    cmd: Option<&str>,
    timeout: Duration,
) -> Option<Value> {
    let run: Value = serde_json::from_str(&fs::read_to_string(run_file(dir)).ok()?).ok()?;
    let port = u16::try_from(run.get("port")?.as_u64()?).ok()?;
    let token = run.get("token")?.as_str()?;

    let mut payload = Map::new();
    payload.insert("action".into(), action.into());
    if let Some(cmd) = cmd {
        payload.insert("cmd".into(), cmd.into());
    }
    payload.insert("token".into(), token.into());
    let request = pyjson::to_compact(&payload).ok()? + "\n";

    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let mut stream = TcpStream::connect_timeout(&address, timeout).ok()?;
    stream.set_read_timeout(Some(timeout)).ok()?;
    stream.set_write_timeout(Some(timeout)).ok()?;
    stream.write_all(request.as_bytes()).ok()?;
    let mut data = Vec::new();
    let mut chunk = [0u8; 4096];
    while !data.ends_with(b"\n") {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => data.extend_from_slice(&chunk[..n]),
            Err(_) => return None,
        }
    }
    let text = String::from_utf8(data).ok()?;
    serde_json::from_str(if text.is_empty() { "null" } else { &text }).ok()
}

fn answered_ok(response: Option<Value>) -> bool {
    response.is_some_and(|response| response.get("ok").is_some_and(py_truthy))
}

/// El servidor esta en marcha si su supervisor responde al ping.
pub fn is_running(dir: &Path) -> bool {
    answered_ok(control_request(dir, "ping", None, Duration::from_secs(2)))
}

/// Escribe un comando en la consola del servidor.
pub fn send_command(dir: &Path, command: &str) -> bool {
    answered_ok(control_request(
        dir,
        "cmd",
        Some(command),
        Duration::from_secs(5),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::net::TcpListener;
    use std::thread;

    /// Supervisor falso: atiende una conexion, devuelve la peticion recibida y
    /// responde `reply`.
    fn fake_supervisor(dir: &Path, reply: &'static str) -> thread::JoinHandle<String> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        fs::create_dir_all(dir.join(RUN_DIR)).unwrap();
        fs::write(
            run_file(dir),
            format!("{{\"port\": {port}, \"token\": \"abc\", \"pid\": 1, \"wrapper_pid\": 2, \"started\": \"x\"}}"),
        )
        .unwrap();
        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut line = String::new();
            BufReader::new(&stream).read_line(&mut line).unwrap();
            (&stream).write_all(reply.as_bytes()).unwrap();
            line
        })
    }

    #[test]
    fn sin_run_json_no_esta_en_marcha() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!is_running(tmp.path()));
        assert!(!send_command(tmp.path(), "stop"));
    }

    #[test]
    fn ping_y_comando_con_el_formato_de_python() {
        let tmp = tempfile::tempdir().unwrap();
        let server = fake_supervisor(tmp.path(), "{\"ok\": true, \"pid\": 1, \"ready\": false}\n");
        assert!(is_running(tmp.path()));
        assert_eq!(
            server.join().unwrap(),
            "{\"action\": \"ping\", \"token\": \"abc\"}\n"
        );

        let server = fake_supervisor(tmp.path(), "{\"ok\": true}\n");
        assert!(send_command(tmp.path(), "say hola"));
        assert_eq!(
            server.join().unwrap(),
            "{\"action\": \"cmd\", \"cmd\": \"say hola\", \"token\": \"abc\"}\n"
        );
    }

    #[test]
    fn respuesta_negativa_o_rota() {
        let tmp = tempfile::tempdir().unwrap();
        let server = fake_supervisor(tmp.path(), "{\"ok\": false, \"error\": \"token\"}\n");
        assert!(!is_running(tmp.path()));
        server.join().unwrap();

        let server = fake_supervisor(tmp.path(), "no es json\n");
        assert!(!is_running(tmp.path()));
        server.join().unwrap();
    }
}
