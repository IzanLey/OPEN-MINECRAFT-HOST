//! `omh start`, `stop`, `restart`, `status`, `console`, `logs` y `cmd`.

use std::fs::File;
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use serde_json::Value;

use super::control::{is_running, run_file, send_command};
use super::{build_command, preflight, run, spawn_daemon};
use crate::cli::{CmdArgs, DirArgs, LogsArgs, StartArgs};
use crate::config::{self, RUN_DIR};
use crate::error::fail;
use crate::network::server_addresses;
use crate::platform::IS_WIN;
use crate::system::dir_size_mb;
use crate::ui::{self, CtrlC};
use crate::util::{fmt_mb, tail_lines};

pub fn cmd_start(args: &StartArgs) -> anyhow::Result<()> {
    let cfg = config::load_config(&config::resolve_dir(args.dir.as_deref())?)?;
    if !args.detach {
        return run(&cfg, args.daemon);
    }
    preflight(&cfg, true)?;
    build_command(&cfg)?; // descarga Java ahora si falta, para ver el progreso
    spawn_daemon(&cfg)?;
    for _ in 0..30 {
        thread::sleep(Duration::from_secs(1));
        if is_running(&cfg.dir) {
            ui::ok(&format!(
                "Servidor '{}' iniciado en segundo plano.",
                cfg.name
            ));
            for line in server_addresses(&cfg) {
                ui::info(&format!("  {line}"));
            }
            ui::info("Consola: omh console | Registro: omh logs | Parar: omh stop");
            return Ok(());
        }
    }
    Err(fail("El servidor no arranco. Revisa: omh logs"))
}

fn kill_pid(pid: u64) {
    if pid == 0 {
        return;
    }
    if IS_WIN {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    } else {
        #[cfg(unix)]
        if let Ok(pid) = libc::pid_t::try_from(pid) {
            // SAFETY: llamada sin punteros; si el proceso no existe falla sin mas.
            unsafe {
                libc::kill(pid, libc::SIGTERM);
            }
        }
    }
}

pub fn cmd_stop(args: &DirArgs) -> anyhow::Result<()> {
    let cfg = config::load_config(&config::resolve_dir(args.dir.as_deref())?)?;
    if !is_running(&cfg.dir) {
        ui::info(&format!("El servidor '{}' no esta en ejecucion.", cfg.name));
        return Ok(());
    }
    ui::info(&format!(
        "Deteniendo '{}' (guardando el mundo)...",
        cfg.name
    ));
    send_command(&cfg.dir, "stop");
    for _ in 0..90 {
        thread::sleep(Duration::from_secs(1));
        if !is_running(&cfg.dir) {
            ui::ok("Servidor detenido.");
            return Ok(());
        }
    }
    ui::warn("El servidor no respondio en 90 segundos. Se fuerza el cierre.");
    let path = run_file(&cfg.dir);
    if let Some(run) = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
    {
        kill_pid(run.get("pid").and_then(Value::as_u64).unwrap_or(0));
        kill_pid(run.get("wrapper_pid").and_then(Value::as_u64).unwrap_or(0));
        let _ = std::fs::remove_file(&path);
    }
    Ok(())
}

pub fn cmd_restart(args: &DirArgs) -> anyhow::Result<()> {
    cmd_stop(args)?;
    cmd_start(&StartArgs {
        dir: args.dir.clone(),
        detach: true,
        daemon: false,
    })
}

pub fn cmd_logs(args: &LogsArgs) -> anyhow::Result<()> {
    let cfg = config::load_config(&config::resolve_dir(args.dir.as_deref())?)?;
    for line in tail_lines(&cfg.dir.join(RUN_DIR).join("console.log"), args.lines) {
        println!("{line}");
    }
    Ok(())
}

pub fn cmd_cmd(args: &CmdArgs) -> anyhow::Result<()> {
    let cfg = config::load_config(&config::resolve_dir(args.dir.as_deref())?)?;
    let command = args.text.join(" ");
    if !send_command(&cfg.dir, &command) {
        return Err(fail("El servidor no esta en ejecucion."));
    }
    ui::ok(&format!("Enviado: {command}"));
    Ok(())
}

/// Imprime lo que se va anadiendo al registro (como `tail -f`).
fn follow(path: std::path::PathBuf, stop: Arc<AtomicBool>) {
    let Ok(mut file) = File::open(&path) else {
        return;
    };
    let _ = file.seek(SeekFrom::End(0));
    let mut pending: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 8192];
    while !stop.load(Ordering::SeqCst) {
        match file.read(&mut chunk) {
            // Como `readline()` de Python al llegar al final del archivo: una
            // linea sin salto de linea pendiente tambien se imprime.
            Ok(0) => {
                if !pending.is_empty() {
                    let text = String::from_utf8_lossy(&pending);
                    let mut out = io::stdout();
                    let _ = writeln!(out, "{}", text.trim_end_matches(['\r', '\n']));
                    let _ = out.flush();
                    pending.clear();
                }
                thread::sleep(Duration::from_millis(300));
            }
            // Python ignora los errores de lectura y sigue esperando.
            Err(_) => thread::sleep(Duration::from_millis(300)),
            Ok(n) => {
                pending.extend_from_slice(&chunk[..n]);
                while let Some(end) = pending.iter().position(|&b| b == b'\n') {
                    let line: Vec<u8> = pending.drain(..=end).collect();
                    let text = String::from_utf8_lossy(&line);
                    let mut out = io::stdout();
                    let _ = writeln!(out, "{}", text.trim_end_matches(['\r', '\n']));
                    let _ = out.flush();
                }
            }
        }
    }
}

pub fn cmd_console(args: &DirArgs) -> anyhow::Result<()> {
    let cfg = config::load_config(&config::resolve_dir(args.dir.as_deref())?)?;
    if !is_running(&cfg.dir) {
        return Err(fail(
            "El servidor no esta en ejecucion. Inicialo con: omh start",
        ));
    }
    let path = cfg.dir.join(RUN_DIR).join("console.log");
    for line in tail_lines(&path, 30) {
        println!("{line}");
    }
    ui::info(&format!(
        "-- Consola de '{}'. Escribe comandos. Ctrl+C para salir (el servidor sigue). --",
        cfg.name
    ));
    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = Arc::clone(&stop);
        thread::spawn(move || follow(path, stop));
    }
    ui::set_ctrlc(CtrlC::QuietExit);
    let stdin = io::stdin();
    let mut line = String::new();
    loop {
        line.clear();
        match stdin.lock().read_line(&mut line) {
            Ok(0) | Err(_) => {
                println!();
                break;
            }
            Ok(_) => {
                if !send_command(&cfg.dir, line.trim_end_matches(['\n', '\r'])) {
                    ui::info("El servidor ya no esta en ejecucion.");
                    break;
                }
            }
        }
    }
    stop.store(true, Ordering::SeqCst);
    ui::set_ctrlc(CtrlC::Cancel);
    Ok(())
}

pub fn cmd_status(args: &DirArgs) -> anyhow::Result<()> {
    let cfg = config::load_config(&config::resolve_dir(args.dir.as_deref())?)?;
    let running = is_running(&cfg.dir);
    ui::header(&format!("Servidor {}", cfg.name));
    ui::info(&format!("Carpeta:   {}", cfg.dir.display()));
    let java = cfg
        .java
        .map(|java| format!(" | Java {java}"))
        .unwrap_or_default();
    ui::info(&format!(
        "Edicion:   {} | core {} | version {}{java}",
        cfg.edition, cfg.core, cfg.version
    ));
    match &cfg.resources {
        Some(res) => {
            ui::info(&format!(
                "Recursos:  RAM {} | almacenamiento {} | {} nucleos",
                fmt_mb(res.ram_mb),
                fmt_mb(res.storage_mb),
                res.cores
            ));
            ui::info(&format!(
                "Disco:     usa {} de {}",
                fmt_mb(dir_size_mb(&cfg.dir)),
                fmt_mb(res.storage_mb)
            ));
        }
        None => ui::info("Recursos:  sin configurar (omh resources)"),
    }
    // B1: una red a medias (solo admins) se muestra como sin configurar.
    match cfg.network() {
        Some(n) => {
            let method = n
                .public_method
                .as_deref()
                .filter(|m| !m.is_empty())
                .map(|m| format!(" ({m})"))
                .unwrap_or_default();
            ui::info(&format!(
                "Red:       modo {}{method} | puerto {} | whitelist {} | online-mode {}",
                n.mode,
                n.port,
                if n.whitelist { "on" } else { "off" },
                if n.online_mode { "on" } else { "off" }
            ));
            let admins = if n.admins.is_empty() {
                "ninguno".to_owned()
            } else {
                n.admins.join(", ")
            };
            ui::info(&format!("Admins:    {admins}"));
            for line in server_addresses(&cfg) {
                ui::info(&format!("           {line}"));
            }
        }
        None => ui::info("Red:       sin configurar (omh network)"),
    }
    ui::info(&format!(
        "Estado:    {}",
        if running { "EN EJECUCION" } else { "DETENIDO" }
    ));
    Ok(())
}
