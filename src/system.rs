//! Recursos del equipo como los mide Python: RAM total, CPUs logicas del sistema,
//! espacio libre de una carpeta y lo que ocupa una carpeta.

use std::fs;
use std::path::Path;

use anyhow::Context;

use crate::util::MB;

/// RAM fisica total en MB (0 si no se puede saber).
#[cfg(windows)]
pub fn total_ram_mb() -> i64 {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    // SAFETY: estructura inicializada con su tamano, como pide la API.
    let mut status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    if unsafe { GlobalMemoryStatusEx(&mut status) } == 0 {
        return 0;
    }
    i64::try_from(status.ullTotalPhys / MB).unwrap_or(0)
}

#[cfg(not(windows))]
pub fn total_ram_mb() -> i64 {
    let Ok(text) = fs::read_to_string("/proc/meminfo") else {
        return 0;
    };
    text.lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))
        .and_then(|rest| rest.split_whitespace().next()?.parse::<i64>().ok())
        .map_or(0, |kb| kb / 1024)
}

/// CPUs logicas del sistema (`os.cpu_count()`), no las permitidas por afinidad.
#[cfg(windows)]
pub fn cpu_count() -> i64 {
    use windows_sys::Win32::System::Threading::{ALL_PROCESSOR_GROUPS, GetActiveProcessorCount};

    // SAFETY: llamada sin punteros.
    let count = unsafe { GetActiveProcessorCount(ALL_PROCESSOR_GROUPS) };
    i64::from(count).max(1)
}

#[cfg(not(windows))]
pub fn cpu_count() -> i64 {
    // SAFETY: llamada sin punteros.
    let count = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
    count.max(1)
}

/// Espacio libre en MB del disco donde esta `path` (`shutil.disk_usage(path).free`).
#[cfg(windows)]
pub fn disk_free_mb(path: &Path) -> anyhow::Result<i64> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let (mut available, mut total, mut free) = (0u64, 0u64, 0u64);
    // SAFETY: `wide` termina en cero y los punteros apuntan a variables locales.
    let ok = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut available, &mut total, &mut free) };
    if ok == 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("No se pudo leer el espacio libre de {}", path.display()));
    }
    Ok(i64::try_from(free / MB).unwrap_or(i64::MAX))
}

#[cfg(not(windows))]
pub fn disk_free_mb(path: &Path) -> anyhow::Result<i64> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let cpath = CString::new(path.as_os_str().as_bytes())?;
    // SAFETY: `cpath` es una cadena C valida y `stat` una estructura local.
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(cpath.as_ptr(), &mut stat) } != 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("No se pudo leer el espacio libre de {}", path.display()));
    }
    let free = stat.f_bavail.saturating_mul(stat.f_frsize);
    Ok(i64::try_from(free / MB).unwrap_or(i64::MAX))
}

/// Lo que ocupa una carpeta en MB, como `dir_size_mb` (sin entrar en enlaces a carpetas).
pub fn dir_size_mb(path: &Path) -> i64 {
    i64::try_from(dir_size(path) / MB).unwrap_or(i64::MAX)
}

fn dir_size(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    let mut total = 0;
    for entry in entries.flatten() {
        let child = entry.path();
        if child.is_dir() {
            if !entry.file_type().is_ok_and(|kind| kind.is_symlink()) {
                total += dir_size(&child);
            }
        } else if let Ok(meta) = fs::metadata(&child) {
            total += meta.len();
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mide_el_equipo() {
        assert!(total_ram_mb() > 0);
        assert!(cpu_count() >= 1);
        assert!(disk_free_mb(Path::new(".")).unwrap() > 0);
    }

    #[test]
    fn tamano_de_carpeta() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("a")).unwrap();
        fs::write(
            tmp.path().join("a").join("x"),
            vec![0u8; (MB as usize) * 2 + 10],
        )
        .unwrap();
        fs::write(tmp.path().join("y"), vec![0u8; MB as usize]).unwrap();
        assert_eq!(dir_size_mb(tmp.path()), 3);
    }
}
