//! Lo que cambia entre Windows y Linux.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

use anyhow::Context;

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

/// Carpeta de un usuario del sistema (`~usuario`); sin nombre, la del usuario actual.
#[cfg(not(windows))]
pub fn passwd_home(user: &str) -> Option<PathBuf> {
    use std::ffi::{CStr, CString, OsStr};
    use std::os::unix::ffi::OsStrExt;

    let mut entry: libc::passwd = unsafe { std::mem::zeroed() };
    let mut result: *mut libc::passwd = std::ptr::null_mut();
    let mut buf = vec![0 as libc::c_char; 16 * 1024];
    let rc = if user.is_empty() {
        // SAFETY: todos los punteros son validos y `buf` vive durante la llamada.
        unsafe {
            libc::getpwuid_r(
                libc::getuid(),
                &mut entry,
                buf.as_mut_ptr(),
                buf.len(),
                &mut result,
            )
        }
    } else {
        let name = CString::new(user).ok()?;
        // SAFETY: igual que arriba; `name` es una cadena C valida.
        unsafe {
            libc::getpwnam_r(
                name.as_ptr(),
                &mut entry,
                buf.as_mut_ptr(),
                buf.len(),
                &mut result,
            )
        }
    };
    if rc != 0 || result.is_null() || entry.pw_dir.is_null() {
        return None;
    }
    // SAFETY: getpw*_r ha dejado en `pw_dir` una cadena terminada en cero dentro de `buf`.
    let dir = unsafe { CStr::from_ptr(entry.pw_dir) };
    Some(PathBuf::from(OsStr::from_bytes(dir.to_bytes())))
}

/// `platform_ids()` de Python: (`windows` | `mac` | `linux`, `x64` | `aarch64` | otra).
pub fn platform_ids() -> (&'static str, String) {
    let machine = machine().to_lowercase();
    let arch = match machine.as_str() {
        "x86_64" | "amd64" => "x64".to_owned(),
        "aarch64" | "arm64" => "aarch64".to_owned(),
        _ => machine,
    };
    let os = if IS_WIN {
        "windows"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "linux"
    };
    (os, arch)
}

/// `platform.machine()`: la arquitectura del sistema, no la del programa.
#[cfg(windows)]
fn machine() -> String {
    env::var("PROCESSOR_ARCHITEW6432")
        .ok()
        .filter(|arch| !arch.is_empty())
        .or_else(|| env::var("PROCESSOR_ARCHITECTURE").ok())
        .unwrap_or_else(|| env::consts::ARCH.to_owned())
}

#[cfg(not(windows))]
fn machine() -> String {
    let mut info: libc::utsname = unsafe { std::mem::zeroed() };
    // SAFETY: `info` es una estructura valida que uname rellena.
    if unsafe { libc::uname(&mut info) } != 0 {
        return env::consts::ARCH.to_owned();
    }
    let bytes: Vec<u8> = info
        .machine
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Da permiso de ejecucion (`chmod +x`) en Linux; en Windows no hace nada.
pub fn make_executable(path: &Path) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if let Ok(meta) = fs::metadata(path) {
            let mode = meta.permissions().mode() | 0o755;
            fs::set_permissions(path, fs::Permissions::from_mode(mode)).with_context(|| {
                format!("No se pudo dar permiso de ejecucion a {}", path.display())
            })?;
        }
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/// Codigo de salida como lo muestra Python: sin signo en Windows y negativo
/// (la senal) en Linux si el proceso murio por una senal.
pub fn exit_code_text(status: &ExitStatus) -> String {
    #[cfg(windows)]
    {
        status
            .code()
            .map_or_else(String::new, |code| (code as u32).to_string())
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;

        match (status.code(), status.signal()) {
            (Some(code), _) => code.to_string(),
            (None, Some(signal)) => (-signal).to_string(),
            _ => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plataforma_como_python() {
        let (os, arch) = platform_ids();
        assert_eq!(os, if IS_WIN { "windows" } else { "linux" });
        assert!(["x64", "aarch64"].contains(&arch.as_str()), "{arch}");
    }

    #[cfg(not(windows))]
    #[test]
    fn carpeta_del_usuario_actual() {
        assert!(passwd_home("").is_some());
        assert!(passwd_home("nadie_que_exista_123").is_none());
    }
}
