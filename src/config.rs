//! Configuracion de cada servidor (`omh.json`) y estado global
//! (`~/.open-mc-host/state.json`), con el mismo formato que la version Python.

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::error::fail;
use crate::platform;
use crate::pyjson;
use crate::util::{abspath, py_splitlines, py_strip, read_text_lossy, write_text};

pub const CONFIG_FILE: &str = "omh.json";
pub const RUN_DIR: &str = ".omh";

// ---------------------------------------------------------------------------
// Estado global
// ---------------------------------------------------------------------------

pub fn state_file() -> PathBuf {
    platform::app_home().join("state.json")
}

/// `{"current": <carpeta o null>, "servers": {<nombre>: <carpeta>}}`
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    #[serde(default)]
    pub current: Option<String>,
    #[serde(default)]
    pub servers: Map<String, Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl State {
    /// Servidores registrados (nombre, carpeta) en el orden del archivo.
    pub fn servers(&self) -> Vec<(String, String)> {
        self.servers
            .iter()
            .filter_map(|(name, dir)| dir.as_str().map(|dir| (name.clone(), dir.to_owned())))
            .collect()
    }
}

/// Si no existe o no se puede leer, un estado vacio (como Python).
pub fn load_state() -> State {
    fs::read_to_string(state_file())
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_state(state: &State) -> anyhow::Result<()> {
    let home = platform::app_home();
    fs::create_dir_all(&home).with_context(|| format!("No se pudo crear {}", home.display()))?;
    write_text(&state_file(), &pyjson::to_pretty(state)?)
}

/// Apunta el servidor en la lista y lo deja como activo.
pub fn register_server(cfg: &ServerConfig) -> anyhow::Result<()> {
    let mut state = load_state();
    let dir = cfg.dir_string();
    state
        .servers
        .insert(cfg.name.clone(), Value::String(dir.clone()));
    state.current = Some(dir);
    save_state(&state)
}

// ---------------------------------------------------------------------------
// Configuracion de un servidor
// ---------------------------------------------------------------------------

/// Contenido de `omh.json`. Las claves desconocidas se conservan al guardar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    #[serde(default)]
    pub dir: PathBuf,
    pub edition: String,
    pub core: String,
    pub version: String,
    #[serde(default)]
    pub java: Option<u32>,
    #[serde(default)]
    pub step: i64,
    #[serde(default)]
    pub created: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<StartSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<Resources>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<NetworkConfig>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Como arrancar el servidor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum StartSpec {
    /// `java -jar <jar>`
    Jar { jar: String },
    /// `java @<dir>/{win,unix}_args.txt` (Forge y NeoForge modernos)
    Argsfile { dir: String },
    /// Ejecutable propio (Bedrock Dedicated Server)
    Native { exe: String },
    /// `php PocketMine-MP.phar`
    Php { php: String, phar: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resources {
    pub ram_mb: i64,
    pub storage_mb: i64,
    pub cores: i64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// La red completa (paso 3) o solo la lista de administradores, que es lo que queda
/// si se usa `omh admin add` antes de `omh network`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NetworkConfig {
    Full(Network),
    Partial(PartialNetwork),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub mode: String,
    pub public_method: Option<String>,
    pub port: u16,
    pub port_v6: Option<u16>,
    pub bind_ip: String,
    pub whitelist: bool,
    pub online_mode: bool,
    pub online_mode_raw: String,
    pub max_players: i64,
    pub motd: String,
    pub admins: Vec<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartialNetwork {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admins: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl ServerConfig {
    pub fn dir_string(&self) -> String {
        self.dir.to_string_lossy().into_owned()
    }

    pub fn is_java(&self) -> bool {
        self.edition == "java"
    }

    pub fn is_bedrock_vanilla(&self) -> bool {
        self.edition == "bedrock" && self.core == "vanilla"
    }

    /// La red configurada en el paso 3 (`None` si falta o esta a medias).
    pub fn network(&self) -> Option<&Network> {
        match &self.network {
            Some(NetworkConfig::Full(network)) => Some(network),
            _ => None,
        }
    }

    pub fn network_mut(&mut self) -> Option<&mut Network> {
        match &mut self.network {
            Some(NetworkConfig::Full(network)) => Some(network),
            _ => None,
        }
    }

    /// Administradores guardados en la configuracion.
    pub fn admins(&self) -> &[String] {
        match &self.network {
            Some(NetworkConfig::Full(network)) => &network.admins,
            Some(NetworkConfig::Partial(partial)) => partial.admins.as_deref().unwrap_or_default(),
            None => &[],
        }
    }

    /// `cfg.setdefault("network", {}).setdefault("admins", [])`
    pub fn admins_mut(&mut self) -> &mut Vec<String> {
        match self
            .network
            .get_or_insert_with(|| NetworkConfig::Partial(PartialNetwork::default()))
        {
            NetworkConfig::Full(network) => &mut network.admins,
            NetworkConfig::Partial(partial) => partial.admins.get_or_insert_with(Vec::new),
        }
    }
}

pub fn load_config(directory: &Path) -> anyhow::Result<ServerConfig> {
    let path = directory.join(CONFIG_FILE);
    if !path.exists() {
        return Err(fail(format!(
            "No hay ningun servidor en {}. Crea uno con: omh create",
            directory.display()
        )));
    }
    let text =
        fs::read_to_string(&path).with_context(|| format!("No se pudo leer {}", path.display()))?;
    let mut cfg: ServerConfig = serde_json::from_str(&text)
        .map_err(|err| fail(format!("El archivo {} no es valido: {err}", path.display())))?;
    cfg.dir = abspath(directory);
    Ok(cfg)
}

pub fn save_config(cfg: &ServerConfig) -> anyhow::Result<()> {
    write_text(&cfg.dir.join(CONFIG_FILE), &pyjson::to_pretty(cfg)?)
}

/// Carpeta del servidor: `--dir`, la carpeta actual si es un servidor o el activo.
pub fn resolve_dir(dir: Option<&str>) -> anyhow::Result<PathBuf> {
    if let Some(dir) = dir.filter(|dir| !dir.is_empty()) {
        return Ok(abspath(Path::new(dir)));
    }
    let cwd = env::current_dir().context("No se pudo leer la carpeta actual")?;
    if cwd.join(CONFIG_FILE).exists() {
        return Ok(cwd);
    }
    if let Some(current) = load_state().current.filter(|current| !current.is_empty()) {
        let current = PathBuf::from(current);
        if current.join(CONFIG_FILE).exists() {
            return Ok(current);
        }
    }
    Err(fail(
        "No se encontro ningun servidor. Crea uno con 'omh create' o indica --dir.",
    ))
}

pub fn require_step(cfg: &ServerConfig, step: i64) -> anyhow::Result<()> {
    let command = match step {
        1 => "omh create",
        2 => "omh resources",
        _ => "omh network",
    };
    if cfg.step < step {
        return Err(fail(format!(
            "Falta el paso {step}. Ejecuta primero: {command}"
        )));
    }
    Ok(())
}

/// Puertos de los demas servidores de omh, para no repetirlos.
pub fn other_ports(cfg: &ServerConfig) -> HashSet<u16> {
    let mut ports = HashSet::new();
    for (_name, dir) in load_state().servers() {
        if abspath(Path::new(&dir)) == cfg.dir {
            continue;
        }
        let Ok(other) = load_config(Path::new(&dir)) else {
            continue;
        };
        if let Some(network) = other.network() {
            ports.extend(
                [Some(network.port), network.port_v6]
                    .into_iter()
                    .flatten()
                    .filter(|&p| p != 0),
            );
        }
    }
    ports
}

// ---------------------------------------------------------------------------
// server.properties
// ---------------------------------------------------------------------------

/// Cambia claves de un `server.properties` sin perder el resto del archivo.
pub fn set_properties(path: &Path, values: &[(&str, String)]) -> anyhow::Result<()> {
    let text = match read_text_lossy(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(err).with_context(|| format!("No se pudo leer {}", path.display())),
    };
    let wanted: HashMap<&str, &String> = values.iter().map(|(key, value)| (*key, value)).collect();
    let mut done = HashSet::new();
    let mut out: Vec<String> = Vec::new();
    for line in py_splitlines(&text) {
        let stripped = py_strip(line);
        if !stripped.is_empty()
            && !stripped.starts_with('#')
            && let Some((key, _)) = stripped.split_once('=')
            && let Some(value) = wanted.get(py_strip(key))
        {
            let key = py_strip(key);
            out.push(format!("{key}={value}"));
            done.insert(key.to_owned());
            continue;
        }
        out.push(line.to_owned());
    }
    for (key, value) in values {
        if !done.contains(*key) {
            out.push(format!("{key}={value}"));
        }
    }
    write_text(path, &(out.join("\n") + "\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::native_newlines;

    // Escrito por json.dump(cfg, indent=2) de Python, con tildes, un caracter no BMP y controles.
    const FULL: &str = r#"{
  "name": "servidor-jose",
  "dir": "C:\\Servidores\\Jos\u00e9",
  "edition": "java",
  "core": "forge",
  "version": "1.20.1",
  "java": 17,
  "step": 3,
  "created": "2026-10-25 23:56:50",
  "start": {
    "type": "argsfile",
    "dir": "libraries\net\\minecraftforge\forge\u0001.20.1-47.2.0"
  },
  "resources": {
    "ram_mb": 4096,
    "storage_mb": 20480,
    "cores": 2
  },
  "network": {
    "mode": "public",
    "public_method": null,
    "port": 25565,
    "port_v6": null,
    "bind_ip": "",
    "whitelist": true,
    "online_mode": true,
    "online_mode_raw": "on",
    "max_players": 20,
    "motd": "Servidor de Jos\u00e9 \ud834\udd1e",
    "admins": [
      "izan",
      "Steve"
    ]
  }
}"#;

    // Red a medias: `omh admin add` antes de `omh network`.
    const PARTIAL: &str = r#"{
  "name": "b",
  "dir": "/srv/b",
  "edition": "bedrock",
  "core": "vanilla",
  "version": "1.21.111.1",
  "java": null,
  "step": 1,
  "created": "2026-10-26 10:00:00",
  "start": {
    "type": "native",
    "exe": "bedrock_server"
  },
  "network": {
    "admins": [
      "Alex"
    ]
  }
}"#;

    #[test]
    fn omh_json_de_python_se_guarda_igual() {
        for text in [FULL, PARTIAL] {
            let cfg: ServerConfig = serde_json::from_str(text).unwrap();
            assert_eq!(pyjson::to_pretty(&cfg).unwrap(), text);
        }
    }

    #[test]
    fn red_completa_y_red_a_medias() {
        let full: ServerConfig = serde_json::from_str(FULL).unwrap();
        let network = full.network().unwrap();
        assert_eq!(
            (network.port, network.public_method.as_deref()),
            (25565, None)
        );
        assert_eq!(full.admins(), ["izan", "Steve"]);
        assert_eq!(
            full.start,
            Some(StartSpec::Argsfile {
                dir: "libraries\net\\minecraftforge\u{c}orge\u{1}.20.1-47.2.0".into()
            })
        );

        let mut partial: ServerConfig = serde_json::from_str(PARTIAL).unwrap();
        assert!(
            partial.network().is_none(),
            "una red a medias cuenta como sin configurar"
        );
        assert_eq!(partial.admins(), ["Alex"]);
        partial.admins_mut().push("Steve".into());
        assert_eq!(partial.admins(), ["Alex", "Steve"]);
        assert!(partial.resources.is_none());
    }

    #[test]
    fn claves_desconocidas_se_conservan() {
        let text = "{\n  \"name\": \"x\",\n  \"dir\": \"/x\",\n  \"edition\": \"java\",\n  \"core\": \"paper\",\n  \"version\": \"1.21.1\",\n  \"java\": 21,\n  \"step\": 1,\n  \"created\": \"\",\n  \"futuro\": [\n    1\n  ]\n}";
        let cfg: ServerConfig = serde_json::from_str(text).unwrap();
        assert_eq!(pyjson::to_pretty(&cfg).unwrap(), text);
    }

    #[test]
    fn estado_vacio_como_python() {
        assert_eq!(
            pyjson::to_pretty(&State::default()).unwrap(),
            "{\n  \"current\": null,\n  \"servers\": {}\n}"
        );
    }

    #[test]
    fn falta_un_paso() {
        let mut cfg: ServerConfig = serde_json::from_str(PARTIAL).unwrap();
        assert!(require_step(&cfg, 1).is_ok());
        let err = require_step(&cfg, 2).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Falta el paso 2. Ejecuta primero: omh resources"
        );
        cfg.step = 2;
        let err = require_step(&cfg, 3).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Falta el paso 3. Ejecuta primero: omh network"
        );
    }

    #[test]
    fn carga_la_carpeta_real_y_avisa_si_no_hay_servidor() {
        let tmp = tempfile::tempdir().unwrap();
        let err = load_config(tmp.path()).unwrap_err();
        assert_eq!(
            err.to_string(),
            format!(
                "No hay ningun servidor en {}. Crea uno con: omh create",
                tmp.path().display()
            )
        );
        fs::write(tmp.path().join(CONFIG_FILE), PARTIAL).unwrap();
        let cfg = load_config(tmp.path()).unwrap();
        assert_eq!(cfg.dir, abspath(tmp.path()));
        fs::write(tmp.path().join(CONFIG_FILE), "{no es json").unwrap();
        assert!(
            load_config(tmp.path())
                .unwrap_err()
                .to_string()
                .contains("no es valido")
        );
    }

    // Resultado de set_properties de legacy/omh.py sobre el mismo archivo.
    #[test]
    fn server_properties_como_python() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("server.properties");
        fs::write(
            &path,
            "#Minecraft server properties\r\n#date\r\nserver-port=25565\r\nmotd=A Minecraft Server\r\n  white-list = false\r\n=nokey\r\n!comment=x\r\nmotd=dup\r\na=1\u{c}b=2\r\n\r\n# fin",
        )
        .unwrap();
        let values = [
            ("server-port", "25566".to_owned()),
            ("motd", "Hola".to_owned()),
            ("white-list", "true".to_owned()),
            ("new-key", "v".to_owned()),
            ("b", "3".to_owned()),
        ];
        set_properties(&path, &values).unwrap();
        let expected = "#Minecraft server properties\n#date\nserver-port=25566\nmotd=Hola\nwhite-list=true\n=nokey\n!comment=x\nmotd=Hola\na=1\nb=3\n\n# fin\nnew-key=v\n";
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            native_newlines(expected)
        );

        let new = tmp.path().join("nuevo.properties");
        set_properties(
            &new,
            &[
                ("max-threads", "4".to_owned()),
                ("server-ip", String::new()),
            ],
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(&new).unwrap(),
            native_newlines("max-threads=4\nserver-ip=\n")
        );
    }
}
