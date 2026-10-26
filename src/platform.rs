//! Lo que cambia entre Windows y Linux.

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

pub const IS_WIN: bool = cfg!(windows);

/// Carpeta de datos de omh: `~/.open-mc-host`.
pub fn app_home() -> PathBuf {
    home_dir().join(".open-mc-host")
}

/// Carpeta de usuario como `os.path.expanduser("~")` de Python: `%USERPROFILE%`
/// en Windows y `$HOME` en Linux, con la del sistema como respaldo.
pub fn home_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Some(profile) = env_nonempty("USERPROFILE") {
            return PathBuf::from(profile);
        }
        if let Some(path) = env_nonempty("HOMEPATH") {
            let drive = env::var_os("HOMEDRIVE").unwrap_or_default();
            return PathBuf::from(drive).join(path);
        }
    }
    #[cfg(not(windows))]
    {
        if let Some(home) = env_nonempty("HOME") {
            return PathBuf::from(home);
        }
    }
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn env_nonempty(name: &str) -> Option<OsString> {
    env::var_os(name).filter(|value| !value.is_empty())
}
