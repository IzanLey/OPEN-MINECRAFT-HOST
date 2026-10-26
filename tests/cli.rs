//! Pruebas del binario `omh` completo, con una carpeta de usuario aislada.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn omh(home: &Path, args: &[&str]) -> Output {
    let mut attempts = 0;
    loop {
        let result = Command::new(env!("CARGO_BIN_EXE_omh"))
            .args(args)
            .env("USERPROFILE", home)
            .env("HOME", home)
            .output();
        match result {
            Ok(output) => return output,
            // En Windows el antivirus puede bloquear un momento el .exe recien compilado.
            Err(err) if attempts < 5 => {
                attempts += 1;
                eprintln!("reintentando omh: {err}");
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            Err(err) => panic!("no se pudo ejecutar omh: {err}"),
        }
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Como escribe Python en modo texto: `\r\n` en Windows.
fn native(text: &str) -> String {
    if cfg!(windows) {
        text.replace('\n', "\r\n")
    } else {
        text.to_owned()
    }
}

fn json_string(text: &str) -> String {
    serde_json::to_string(text).unwrap()
}

#[test]
fn sin_comando_muestra_el_titulo_y_la_guia_de_python() {
    let home = tempfile::tempdir().unwrap();
    let out = omh(home.path(), &[]);
    assert!(out.status.success());
    // Salida de `python legacy/omh.py` con APP_VERSION = 0.1.0.
    assert_eq!(stdout(&out), include_str!("fixtures/omh_sin_comando.txt"));
}

#[test]
fn version() {
    let home = tempfile::tempdir().unwrap();
    let out = omh(home.path(), &["--version"]);
    assert!(out.status.success());
    assert_eq!(
        stdout(&out),
        format!("open-mc-host {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn error_de_uso_sale_con_codigo_2() {
    let home = tempfile::tempdir().unwrap();
    let out = omh(home.path(), &["create", "--edition", "pc"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn servers_sin_servidores() {
    let home = tempfile::tempdir().unwrap();
    let out = omh(home.path(), &["servers"]);
    assert!(out.status.success());
    assert_eq!(
        stdout(&out),
        "  No hay servidores. Crea uno con: omh create\n"
    );
}

#[test]
fn use_registra_el_servidor_como_python_y_servers_lo_marca() {
    let home = tempfile::tempdir().unwrap();
    let server = tempfile::tempdir().unwrap();
    let dir = server.path().canonicalize().unwrap();
    let dir = dir.to_string_lossy().trim_start_matches(r"\\?\").to_owned();
    fs::write(
        Path::new(&dir).join("omh.json"),
        "{\n  \"name\": \"mi-servidor\",\n  \"dir\": \"x\",\n  \"edition\": \"java\",\n  \"core\": \"paper\",\n  \"version\": \"1.21.1\",\n  \"java\": 21,\n  \"step\": 1,\n  \"created\": \"2026-10-26 10:00:00\",\n  \"start\": {\n    \"type\": \"jar\",\n    \"jar\": \"server.jar\"\n  }\n}",
    )
    .unwrap();

    let out = omh(home.path(), &["use", &dir]);
    assert!(out.status.success(), "{}", stdout(&out));
    assert_eq!(
        stdout(&out),
        format!("  [OK] Servidor activo: mi-servidor ({dir})\n")
    );

    let state = fs::read_to_string(home.path().join(".open-mc-host").join("state.json")).unwrap();
    let expected = format!(
        "{{\n  \"current\": {d},\n  \"servers\": {{\n    \"mi-servidor\": {d}\n  }}\n}}",
        d = json_string(&dir)
    );
    assert_eq!(state, native(&expected));

    let out = omh(home.path(), &["servers"]);
    assert_eq!(
        stdout(&out),
        format!(
            "  * mi-servidor          detenido      {dir}\n  * = servidor activo. Cambia con: omh use <nombre o carpeta>\n"
        )
    );

    // Tambien por nombre.
    let out = omh(home.path(), &["use", "mi-servidor"]);
    assert!(out.status.success());
}

#[test]
fn comando_sin_servidor_da_error_claro() {
    let home = tempfile::tempdir().unwrap();
    let out = omh(home.path(), &["use", "no-existe-este-servidor"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout(&out).starts_with("  [ERROR] No hay ningun servidor en "));
}
