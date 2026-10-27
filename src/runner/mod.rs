//! Supervisor (`omh start`): lanza el servidor, reenvia la consola, vigila el disco
//! y atiende el canal de control. Es el `Runner` de `legacy/omh.py`.

pub mod commands;
pub mod control;

use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, LazyLock, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Context;
use regex::Regex;
use serde_json::{Map, Value};

use crate::config::{self, RUN_DIR, ServerConfig, StartSpec};
use crate::cores::{eula_accepted, write_eula};
use crate::error::{fail, user_message};
use crate::java::ensure_java;
use crate::network::{self, port_free, public_ip, server_addresses, tunnel, upnp};
use crate::platform::{self, IS_WIN};
use crate::pyjson;
use crate::system::{cpu_count, dir_size_mb};
use crate::ui::{self, CtrlC};
use crate::util::{MB, fmt_mb, local_timestamp, native_newlines, py_strip, tail_lines};

static READY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Done \(|Server started\.|Done!").expect("expresion regular fija y probada")
});
static BEDROCK_JOIN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Player connected: (.+?), xuid").expect("expresion regular fija y probada")
});

#[cfg(windows)]
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Linea de comandos del servidor segun `start` (como `build_command` de Python).
/// Si falta Java, lo descarga.
pub fn build_command(cfg: &ServerConfig) -> anyhow::Result<Command> {
    let start = cfg.start.as_ref().ok_or_else(|| {
        fail("El servidor no sabe como arrancar. Reinstalalo con: omh create --force")
    })?;
    let res = cfg
        .resources
        .as_ref()
        .ok_or_else(|| fail("Falta el paso 2. Ejecuta primero: omh resources"))?;
    let mut command = match start {
        StartSpec::Jar { .. } | StartSpec::Argsfile { .. } => {
            let major = cfg
                .java
                .ok_or_else(|| fail("Este servidor no tiene version de Java."))?;
            let mut command = Command::new(ensure_java(major)?);
            command.args([
                format!("-Xms{}M", res.ram_mb),
                format!("-Xmx{}M", res.ram_mb),
                "-XX:+UseG1GC".to_owned(),
                "-XX:+ParallelRefProcEnabled".to_owned(),
                "-XX:MaxGCPauseMillis=200".to_owned(),
                format!("-XX:ActiveProcessorCount={}", res.cores),
            ]);
            match start {
                StartSpec::Jar { jar } => {
                    command.arg("-jar").arg(jar);
                    if cfg.is_java() {
                        command.arg("nogui");
                    }
                }
                StartSpec::Argsfile { dir } => {
                    let file = Path::new(dir).join(if IS_WIN {
                        "win_args.txt"
                    } else {
                        "unix_args.txt"
                    });
                    command.arg(format!("@{}", file.display())).arg("nogui");
                }
                _ => {}
            }
            command
        }
        StartSpec::Native { exe } => {
            let mut command = Command::new(cfg.dir.join(exe));
            if !IS_WIN {
                command.env("LD_LIBRARY_PATH", &cfg.dir);
            }
            command
        }
        StartSpec::Php { php, phar } => {
            let mut command = Command::new(cfg.dir.join(php));
            command.arg(cfg.dir.join(phar)).arg("--no-wizard");
            command
        }
    };
    command.current_dir(&cfg.dir);
    Ok(command)
}

/// Comprobaciones antes de arrancar.
pub fn preflight(cfg: &ServerConfig, daemon: bool) -> anyhow::Result<()> {
    config::require_step(cfg, 3)?;
    if control::is_running(&cfg.dir) {
        return Err(fail(
            "El servidor ya esta en ejecucion. Usa: omh console  u  omh stop",
        ));
    }
    if cfg.is_java() && !eula_accepted(&cfg.dir) {
        ui::info("Debes aceptar el EULA de Minecraft: https://aka.ms/MinecraftEULA");
        if daemon || !ui::confirm("Aceptas el EULA de Minecraft?", false) {
            return Err(fail("EULA no aceptado. Usa: omh eula --accept"));
        }
        write_eula(&cfg.dir)?;
    }
    let used = dir_size_mb(&cfg.dir);
    let limit = cfg.resources.as_ref().map_or(0, |res| res.storage_mb);
    if used >= limit {
        return Err(fail(format!(
            "El servidor ocupa {} y su limite es {}. Amplialo con: omh resources",
            fmt_mb(used),
            fmt_mb(limit)
        )));
    }
    let network = cfg
        .network()
        .ok_or_else(|| fail("Falta el paso 3. Ejecuta primero: omh network"))?;
    if !port_free(network.port, cfg.edition == "bedrock") {
        return Err(fail(format!(
            "El puerto {} esta ocupado por otro programa. Cambialo con: omh network",
            network.port
        )));
    }
    Ok(())
}

/// Lo compartido entre los hilos del supervisor.
struct Shared {
    cfg: ServerConfig,
    daemon: bool,
    log: Mutex<File>,
    stdin: Mutex<Option<ChildStdin>>,
    exited: AtomicBool,
    ready: AtomicBool,
    pid: AtomicU32,
    token: String,
}

impl Shared {
    fn log_line(&self, line: &str) {
        let mut log = self
            .log
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = log.write_all(native_newlines(&format!("{line}\n")).as_bytes());
        let _ = log.flush();
    }

    fn say(&self, message: &str) {
        let mut out = io::stdout();
        let _ = writeln!(out, "  [omh] {message}");
        let _ = out.flush();
        self.log_line(&format!("[omh] {message}"));
    }

    /// Escribe un comando en la consola del servidor.
    fn send(&self, command: &str) -> bool {
        let command = py_strip(&command.replace('\r', "").replace('\n', " ")).to_owned();
        if command.is_empty() || self.exited.load(Ordering::SeqCst) {
            return false;
        }
        let mut stdin = self
            .stdin
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(stdin) = stdin.as_mut() else {
            return false;
        };
        stdin
            .write_all(format!("{command}\n").as_bytes())
            .and_then(|()| stdin.flush())
            .is_ok()
    }
}

/// UPnP abierto y procesos auxiliares (agente de playit) del arranque.
#[derive(Default)]
struct NetworkState {
    gateway: Option<upnp::Gateway>,
    extra: Vec<Child>,
}

fn ports_to_open(cfg: &ServerConfig) -> Vec<u16> {
    cfg.network()
        .map(|n| ports_of(n.port, n.port_v6))
        .unwrap_or_default()
}

/// Puertos que se abren en el router. `port_v6` a 0 significa "elige uno libre",
/// no un puerto que abrir, igual que en Python.
fn ports_of(port: u16, port_v6: Option<u16>) -> Vec<u16> {
    [Some(port), port_v6]
        .into_iter()
        .flatten()
        .filter(|&p| p != 0)
        .collect()
}

fn protocol(cfg: &ServerConfig) -> &'static str {
    if cfg.edition == "bedrock" {
        "UDP"
    } else {
        "TCP"
    }
}

fn start_network(shared: &Shared) -> anyhow::Result<NetworkState> {
    let cfg = &shared.cfg;
    let mut state = NetworkState::default();
    let Some(n) = cfg.network().filter(|n| n.mode == "public") else {
        return Ok(state);
    };
    match n.public_method.as_deref() {
        Some("upnp") => {
            let Some(gateway) = upnp::discover() else {
                shared.say("UPnP no disponible: el puerto no se abrio en el router.");
                return Ok(state);
            };
            let description = format!("open-mc-host {}", cfg.name);
            let opened = ports_to_open(cfg)
                .into_iter()
                .try_for_each(|port| upnp::add(&gateway, port, protocol(cfg), &description));
            match opened {
                Ok(()) => {
                    // Un router sin WAN puede devolver ""; como en Python, cuenta
                    // como vacio y se prueba la IP publica.
                    let ip = upnp::external_ip(&gateway)
                        .filter(|ip| !ip.is_empty())
                        .or_else(public_ip)
                        .unwrap_or_else(|| "?".to_owned());
                    shared.say(&format!(
                        "UPnP: puerto {}/{} abierto. IP publica: {ip}",
                        n.port,
                        protocol(cfg)
                    ));
                    state.gateway = Some(gateway);
                }
                Err(err) => shared.say(&format!(
                    "UPnP: el router rechazo abrir el puerto ({}).",
                    user_message(&err)
                )),
            }
        }
        Some("playit") => {
            let exe = tunnel::ensure_playit()?;
            let log_path = cfg.dir.join(RUN_DIR).join("playit.log");
            let log = File::create(&log_path)
                .with_context(|| format!("No se pudo crear {}", log_path.display()))?;
            let mut command = tunnel::playit_command(&exe);
            command
                .stdin(Stdio::null())
                .stdout(log.try_clone()?)
                .stderr(log);
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                command.creation_flags(CREATE_NO_WINDOW);
                keep_std_handles_private();
            }
            state.extra.push(
                command
                    .spawn()
                    .with_context(|| format!("No se pudo ejecutar {}", exe.display()))?,
            );
            shared.say("Tunel playit.gg iniciado (registro en .omh/playit.log).");
            thread::sleep(Duration::from_secs(4));
            if let Some(claim) = network::playit_claim(&tail_lines(&log_path, 50).join("\n")) {
                shared.say(&format!(
                    "playit.gg aun no esta vinculado. Abre {claim} o ejecuta: omh tunnel"
                ));
            }
        }
        _ => {}
    }
    Ok(state)
}

fn stop_network(cfg: &ServerConfig, state: &mut NetworkState) {
    if let Some(gateway) = &state.gateway {
        for port in ports_to_open(cfg) {
            upnp::delete(gateway, port, protocol(cfg));
        }
    }
    for child in &mut state.extra {
        // terminate() de Python: SIGTERM en Linux, TerminateProcess en Windows.
        #[cfg(unix)]
        // SAFETY: senal a un proceso hijo nuestro.
        unsafe {
            libc::kill(child.id() as libc::pid_t, libc::SIGTERM);
        }
        #[cfg(windows)]
        let _ = child.kill();
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline && matches!(child.try_wait(), Ok(None)) {
            thread::sleep(Duration::from_millis(100));
        }
    }
}

/// Compara el token en tiempo constante.
fn same_token(given: &str, expected: &str) -> bool {
    given.len() == expected.len()
        && given
            .bytes()
            .zip(expected.bytes())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
}

fn handle_client(shared: &Shared, mut stream: TcpStream) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    let mut data = Vec::new();
    let mut chunk = [0u8; 4096];
    while !data.ends_with(b"\n") && data.len() < 65536 {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => data.extend_from_slice(&chunk[..n]),
            Err(_) => return,
        }
    }
    let Ok(request) = serde_json::from_slice::<Value>(&data) else {
        return;
    };
    let token = match request.get("token") {
        Some(Value::String(token)) => token.clone(),
        Some(other) => other.to_string(),
        None => String::new(),
    };
    let mut reply = Map::new();
    if !same_token(&token, &shared.token) {
        reply.insert("ok".into(), false.into());
        reply.insert("error".into(), "token".into());
    } else {
        match request.get("action").and_then(Value::as_str) {
            Some("ping") => {
                reply.insert("ok".into(), true.into());
                reply.insert("pid".into(), shared.pid.load(Ordering::SeqCst).into());
                reply.insert("ready".into(), shared.ready.load(Ordering::SeqCst).into());
            }
            Some("cmd") => {
                let cmd = match request.get("cmd") {
                    Some(Value::String(cmd)) => cmd.clone(),
                    Some(other) => other.to_string(),
                    None => String::new(),
                };
                reply.insert("ok".into(), shared.send(&cmd).into());
            }
            _ => {
                reply.insert("ok".into(), false.into());
            }
        }
    }
    if let Ok(text) = pyjson::to_compact(&reply) {
        let _ = stream.write_all(format!("{text}\n").as_bytes());
    }
}

fn pump_output(shared: &Shared, output: impl Read) {
    let admins: HashSet<String> = shared
        .cfg
        .admins()
        .iter()
        .map(|a| a.to_lowercase())
        .collect();
    let mut opped: HashSet<String> = HashSet::new();
    for raw in BufReader::new(output).split(b'\n') {
        let Ok(raw) = raw else {
            break;
        };
        let text = String::from_utf8_lossy(&raw);
        let line = text.trim_end_matches(['\r', '\n']);
        if !shared.daemon {
            let mut out = io::stdout();
            let _ = writeln!(out, "{line}");
            let _ = out.flush();
        }
        shared.log_line(line);
        if !shared.ready.load(Ordering::SeqCst) && READY.is_match(line) {
            shared.ready.store(true, Ordering::SeqCst);
            shared.say("Servidor listo. Direcciones:");
            for address in server_addresses(&shared.cfg) {
                shared.say(&format!("  {address}"));
            }
        }
        if let Some(name) = BEDROCK_JOIN.captures(line).and_then(|c| c.get(1)) {
            let lower = name.as_str().to_lowercase();
            if admins.contains(&lower) && opped.insert(lower) {
                shared.send(&format!("op \"{}\"", name.as_str()));
            }
        }
    }
}

fn storage_watch(shared: &Shared) {
    let limit = shared
        .cfg
        .resources
        .as_ref()
        .map_or(i64::MAX, |res| res.storage_mb);
    loop {
        for _ in 0..1200 {
            if shared.exited.load(Ordering::SeqCst) {
                return;
            }
            thread::sleep(Duration::from_millis(100));
        }
        let used = dir_size_mb(&shared.cfg.dir);
        if used >= limit {
            shared.say(&format!(
                "Limite de almacenamiento alcanzado ({} de {}). Deteniendo.",
                fmt_mb(used),
                fmt_mb(limit)
            ));
            shared.send("say Limite de almacenamiento alcanzado. El servidor se detiene.");
            thread::sleep(Duration::from_secs(2));
            shared.send("stop");
            return;
        }
        if used as f64 >= limit as f64 * 0.9 {
            shared.say(&format!(
                "Aviso: el servidor usa {} de {} de almacenamiento.",
                fmt_mb(used),
                fmt_mb(limit)
            ));
        }
    }
}

/// Primeras `count` CPUs que el proceso tiene permitidas (B4).
#[cfg(target_os = "linux")]
fn affinity_set(count: usize) -> Option<libc::cpu_set_t> {
    // SAFETY: cpu_set_t es una estructura de bits que se inicializa a cero.
    let mut allowed: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<libc::cpu_set_t>();
    if unsafe { libc::sched_getaffinity(0, size, &mut allowed) } != 0 {
        return None;
    }
    let mut chosen: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    let mut taken = 0;
    for cpu in 0..libc::CPU_SETSIZE as usize {
        if taken == count {
            break;
        }
        if unsafe { libc::CPU_ISSET(cpu, &allowed) } {
            unsafe { libc::CPU_SET(cpu, &mut chosen) };
            taken += 1;
        }
    }
    (taken > 0).then_some(chosen)
}

/// Lanza el servidor como lo hace Python: nueva sesion (Linux) o nuevo grupo de
/// procesos (Windows) para que Ctrl+C no lo mate sin guardar, y afinidad de CPU.
fn configure_server_process(command: &mut Command, cores: i64, cpus: i64) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        #[cfg(target_os = "linux")]
        let affinity = (cores < cpus)
            .then(|| affinity_set(usize::try_from(cores).unwrap_or(1)))
            .flatten();
        // SAFETY: solo llamadas al sistema seguras tras fork (setsid, sched_setaffinity).
        unsafe {
            command.pre_exec(move || {
                if libc::setsid() == -1 {
                    return Err(io::Error::last_os_error());
                }
                #[cfg(target_os = "linux")]
                if let Some(set) = &affinity {
                    libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), set);
                }
                Ok(())
            });
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = (cores, cpus);
        command.creation_flags(CREATE_NEW_PROCESS_GROUP);
    }
}

/// Afinidad en Windows: lo mismo que `(Get-Process).ProcessorAffinity = mascara`.
#[cfg(windows)]
fn apply_windows_affinity(child: &Child, cores: i64, cpus: i64) {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::Threading::SetProcessAffinityMask;

    if cores < cpus && (1..64).contains(&cores) {
        let mask = (1usize << cores) - 1;
        // SAFETY: el handle es el del proceso hijo, valido mientras viva `child`.
        unsafe {
            SetProcessAffinityMask(child.as_raw_handle(), mask);
        }
    }
}

fn write_run_file(cfg: &ServerConfig, port: u16, token: &str, pid: u32) -> anyhow::Result<()> {
    let mut run = Map::new();
    run.insert("port".into(), port.into());
    run.insert("token".into(), token.into());
    run.insert("pid".into(), pid.into());
    run.insert("wrapper_pid".into(), std::process::id().into());
    run.insert("started".into(), local_timestamp().into());
    let path = control::run_file(&cfg.dir);
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&path)
        .with_context(|| format!("No se pudo escribir {}", path.display()))?;
    file.write_all(pyjson::to_compact(&run)?.as_bytes())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Python hace chmod siempre: `mode` solo se aplica al crear el archivo.
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .with_context(|| {
                format!("No se pudieron ajustar los permisos de {}", path.display())
            })?;
    }
    Ok(())
}

fn new_token() -> anyhow::Result<String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes)
        .map_err(|err| fail(format!("No se pudo generar el token: {err}")))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// Ejecuta el servidor y se queda supervisandolo hasta que termina.
pub fn run(cfg: &ServerConfig, daemon: bool) -> anyhow::Result<()> {
    preflight(cfg, daemon)?;
    let mut command = build_command(cfg)?;
    let run_dir = cfg.dir.join(RUN_DIR);
    fs::create_dir_all(&run_dir)
        .with_context(|| format!("No se pudo crear {}", run_dir.display()))?;
    let log_path = run_dir.join("console.log");
    if fs::metadata(&log_path).is_ok_and(|meta| meta.len() > 5 * MB) {
        fs::rename(&log_path, run_dir.join("console.old.log"))
            .with_context(|| format!("No se pudo rotar {}", log_path.display()))?;
    }
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("No se pudo abrir {}", log_path.display()))?;
    let shared = Arc::new(Shared {
        cfg: cfg.clone(),
        daemon,
        log: Mutex::new(log),
        stdin: Mutex::new(None),
        exited: AtomicBool::new(false),
        ready: AtomicBool::new(false),
        pid: AtomicU32::new(0),
        token: new_token()?,
    });
    shared.say(&format!(
        "Iniciando {} ({} {} {})...",
        cfg.name, cfg.edition, cfg.core, cfg.version
    ));
    let cores = cfg.resources.as_ref().map_or(1, |res| res.cores);
    let cpus = cpu_count();

    // B5: si se pulsa Ctrl+C mientras se prepara la red, se limpia antes de salir.
    ui::set_ctrlc(CtrlC::Notify);
    let mut network_state = start_network(&shared)?;
    if ui::take_ctrlc() {
        stop_network(cfg, &mut network_state);
        println!("\n  Cancelado.");
        std::process::exit(130);
    }

    let listener = match TcpListener::bind((Ipv4Addr::LOCALHOST, 0)) {
        Ok(listener) => listener,
        Err(err) => {
            stop_network(cfg, &mut network_state);
            return Err(err).context("No se pudo abrir el canal de control");
        }
    };
    let control_port = listener.local_addr()?.port();
    let (reader, writer) = match io::pipe() {
        Ok(pipe) => pipe,
        Err(err) => {
            stop_network(cfg, &mut network_state);
            return Err(err).context("No se pudo preparar la consola del servidor");
        }
    };
    command
        .stdin(Stdio::piped())
        .stdout(writer.try_clone()?)
        .stderr(writer);
    configure_server_process(&mut command, cores, cpus);
    let spawned = command.spawn();
    drop(command); // cierra nuestra copia del extremo de escritura de la tuberia
    let mut child = match spawned {
        Ok(child) => child,
        Err(err) => {
            stop_network(cfg, &mut network_state);
            return Err(err).context("No se pudo iniciar el servidor");
        }
    };
    #[cfg(windows)]
    apply_windows_affinity(&child, cores, cpus);
    shared.pid.store(child.id(), Ordering::SeqCst);
    *shared
        .stdin
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = child.stdin.take();

    let result = supervise(&shared, &mut child, listener, control_port, reader);

    if matches!(child.try_wait(), Ok(None)) {
        let _ = child.kill();
        let _ = child.wait();
    }
    stop_network(cfg, &mut network_state);
    let _ = fs::remove_file(control::run_file(&cfg.dir));
    ui::set_ctrlc(CtrlC::Cancel);
    result
}

fn supervise(
    shared: &Arc<Shared>,
    child: &mut Child,
    listener: TcpListener,
    control_port: u16,
    output: io::PipeReader,
) -> anyhow::Result<()> {
    write_run_file(&shared.cfg, control_port, &shared.token, child.id())?;

    let stopping_control = Arc::new(AtomicBool::new(false));
    {
        let shared = Arc::clone(shared);
        let stopping = Arc::clone(&stopping_control);
        thread::spawn(move || {
            for stream in listener.incoming() {
                if stopping.load(Ordering::SeqCst) {
                    break;
                }
                if let Ok(stream) = stream {
                    let shared = Arc::clone(&shared);
                    thread::spawn(move || handle_client(&shared, stream));
                }
            }
        });
    }
    {
        let shared = Arc::clone(shared);
        thread::spawn(move || storage_watch(&shared));
    }
    let (done_tx, done_rx) = mpsc::channel();
    {
        let shared = Arc::clone(shared);
        thread::spawn(move || {
            pump_output(&shared, output);
            let _ = done_tx.send(());
        });
    }
    if !shared.daemon {
        ui::info("Escribe comandos del servidor aqui. 'stop' o Ctrl+C para detenerlo.");
        let shared = Arc::clone(shared);
        thread::spawn(move || {
            for line in io::stdin().lock().lines() {
                let Ok(line) = line else {
                    break;
                };
                shared.send(&line);
            }
        });
    }

    let mut stopping = false;
    let status = loop {
        if let Some(status) = child.try_wait().context("No se pudo vigilar el servidor")? {
            break status;
        }
        if ui::take_ctrlc() && !stopping {
            stopping = true;
            shared.say("Deteniendo el servidor de forma segura...");
            shared.send("stop");
        }
        thread::sleep(Duration::from_millis(300));
    };
    shared.exited.store(true, Ordering::SeqCst);
    let _ = done_rx.recv_timeout(Duration::from_secs(5));
    shared.say(&format!(
        "Servidor detenido (codigo {}).",
        platform::exit_code_text(&status)
    ));
    // Cierra el canal de control: se marca y se despierta al hilo que espera conexiones.
    stopping_control.store(true, Ordering::SeqCst);
    let _ = TcpStream::connect((Ipv4Addr::LOCALHOST, control_port));
    Ok(())
}

/// Lanza `omh start --daemon` desacoplado de la terminal (para `--detach`).
pub fn spawn_daemon(cfg: &ServerConfig) -> anyhow::Result<()> {
    let run_dir = cfg.dir.join(RUN_DIR);
    fs::create_dir_all(&run_dir)?;
    let log_path = run_dir.join("wrapper.log");
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("No se pudo abrir {}", log_path.display()))?;
    let exe = std::env::current_exe().context("No se pudo localizar el ejecutable de omh")?;
    let mut command = Command::new(exe);
    command
        .arg("start")
        .arg("--dir")
        .arg(&cfg.dir)
        .arg("--daemon")
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log)
        .current_dir(&cfg.dir);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // SAFETY: setsid es seguro tras fork.
        unsafe {
            command.pre_exec(|| {
                if libc::setsid() == -1 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
        keep_std_handles_private();
    }
    command
        .spawn()
        .context("No se pudo iniciar omh en segundo plano")?;
    Ok(())
}

/// En Windows el hijo hereda todos los handles heredables del padre, no solo los
/// que recibe como entrada y salida. Si la salida de `omh start --detach` va a una
/// tuberia (un script, `| Select-Object`...), el supervisor la mantendria abierta y
/// quien lee no terminaria nunca. Python lo evita con `close_fds=True` en `Popen`.
#[cfg(windows)]
fn keep_std_handles_private() {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Foundation::{HANDLE_FLAG_INHERIT, SetHandleInformation};

    for handle in [
        io::stdin().as_raw_handle(),
        io::stdout().as_raw_handle(),
        io::stderr().as_raw_handle(),
    ] {
        if !handle.is_null() {
            // SAFETY: son los handles estandar de este proceso; si no admiten el
            // cambio (por ejemplo, una consola antigua), la llamada falla sin mas.
            unsafe {
                SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_en_tiempo_constante() {
        assert!(same_token("abc", "abc"));
        assert!(!same_token("abd", "abc"));
        assert!(!same_token("ab", "abc"));
        assert_eq!(new_token().unwrap().len(), 32);
    }

    #[test]
    fn puertos_del_router_sin_el_cero() {
        assert_eq!(ports_of(25565, None), vec![25565]);
        assert_eq!(ports_of(25565, Some(25566)), vec![25565, 25566]);
        assert_eq!(ports_of(25565, Some(0)), vec![25565]);
        assert!(ports_of(0, Some(0)).is_empty());
    }

    #[test]
    fn deteccion_de_listo_y_de_bedrock() {
        assert!(
            READY.is_match("[12:00:00] [Server thread/INFO]: Done (3.2s)! For help, type \"help\"")
        );
        assert!(READY.is_match("[INFO] Server started."));
        assert!(!READY.is_match("Starting minecraft server"));
        let caps = BEDROCK_JOIN
            .captures("[INFO] Player connected: Steve Jobs, xuid: 123")
            .unwrap();
        assert_eq!(&caps[1], "Steve Jobs");
    }
}
