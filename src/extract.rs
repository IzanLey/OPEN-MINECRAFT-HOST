//! Extraccion segura de zip y tar(.gz), como `safe_extract` de `legacy/omh.py`:
//! antes de extraer nada se comprueba que ningun archivo ni enlace salga de la
//! carpeta destino.

use std::fs::{self, File, FileTimes};
use std::io::{self, BufReader, Read, Seek};
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use anyhow::Context;
use flate2::read::GzDecoder;
use tar::EntryType;
use zip::ZipArchive;

use crate::error::fail;
use crate::util::abspath;

pub fn safe_extract(archive: &Path, dest: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dest).with_context(|| format!("No se pudo crear {}", dest.display()))?;
    let file =
        File::open(archive).with_context(|| format!("No se pudo abrir {}", archive.display()))?;
    match ZipArchive::new(BufReader::new(file)) {
        Ok(zip) => extract_zip(zip, dest),
        Err(_) => extract_tar(archive, dest),
    }
}

/// `_inside` de Python: comparacion de rutas normalizadas sin tocar el disco.
fn inside(base: &Path, target: &Path) -> bool {
    abspath(target).starts_with(abspath(base))
}

fn unsafe_archive(name: &str) -> anyhow::Error {
    fail(format!("Archivo comprimido inseguro: {name}"))
}

// ---------------------------------------------------------------------------
// zip
// ---------------------------------------------------------------------------

fn extract_zip<R: Read + Seek>(mut zip: ZipArchive<R>, dest: &Path) -> anyhow::Result<()> {
    let names: Vec<String> = zip
        .file_names()
        .map(|name| name.map(|name| name.into_owned()))
        .collect::<Result<_, _>>()
        .map_err(|err| fail(format!("No se pudo leer el archivo comprimido ({err}).")))?;
    for name in &names {
        if !inside(dest, &dest.join(name)) {
            return Err(unsafe_archive(name));
        }
    }
    for index in 0..zip.len() {
        let mut entry = zip
            .by_index(index)
            .with_context(|| "No se pudo leer el archivo comprimido".to_owned())?;
        let name = entry
            .name()
            .map_err(|err| fail(format!("No se pudo leer el archivo comprimido ({err}).")))?
            .into_owned();
        let target = dest.join(zip_member_path(&name));
        if entry.is_dir() {
            fs::create_dir_all(&target)
                .with_context(|| format!("No se pudo crear {}", target.display()))?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("No se pudo crear {}", parent.display()))?;
        }
        let mut out = File::create(&target)
            .with_context(|| format!("No se pudo crear {}", target.display()))?;
        io::copy(&mut entry, &mut out)
            .with_context(|| format!("No se pudo extraer {}", target.display()))?;
    }
    Ok(())
}

/// Ruta relativa como la calcula `zipfile` al extraer: sin unidad, sin `.`/`..`
/// ni partes vacias y, en Windows, sin caracteres prohibidos ni puntos/espacios finales.
fn zip_member_path(name: &str) -> PathBuf {
    let seps: &[char] = if cfg!(windows) { &['/', '\\'] } else { &['/'] };
    let name = if cfg!(windows) && name.as_bytes().get(1) == Some(&b':') {
        &name[2..]
    } else {
        name
    };
    let mut path = PathBuf::new();
    for part in name.split(seps) {
        if part.is_empty() || part == "." || part == ".." {
            continue;
        }
        if cfg!(windows) {
            let cleaned: String = part
                .chars()
                .map(|c| if ":<>|\"?*".contains(c) { '_' } else { c })
                .collect();
            let cleaned = cleaned.trim_end_matches([' ', '.']);
            if !cleaned.is_empty() {
                path.push(cleaned);
            }
        } else {
            path.push(part);
        }
    }
    path
}

// ---------------------------------------------------------------------------
// tar
// ---------------------------------------------------------------------------

fn open_tar(archive: &Path) -> anyhow::Result<tar::Archive<Box<dyn Read>>> {
    let mut file =
        File::open(archive).with_context(|| format!("No se pudo abrir {}", archive.display()))?;
    let mut magic = [0u8; 2];
    let read = file.read(&mut magic)?;
    file.rewind()?;
    let reader: Box<dyn Read> = if read == 2 && magic == [0x1f, 0x8b] {
        Box::new(GzDecoder::new(BufReader::new(file)))
    } else {
        Box::new(BufReader::new(file))
    };
    Ok(tar::Archive::new(reader))
}

fn tar_error(err: io::Error) -> anyhow::Error {
    fail(format!("No se pudo leer el archivo comprimido ({err})."))
}

fn extract_tar(archive: &Path, dest: &Path) -> anyhow::Result<()> {
    // Primera pasada: comprobar todo, como hace Python antes de extraer.
    let mut tar = open_tar(archive)?;
    for entry in tar.entries().map_err(tar_error)? {
        let entry = entry.map_err(tar_error)?;
        let name = String::from_utf8_lossy(&entry.path_bytes()).into_owned();
        let target = dest.join(entry.path().map_err(tar_error)?);
        if !inside(dest, &target) {
            return Err(unsafe_archive(&name));
        }
        let kind = entry.header().entry_type();
        if kind.is_symlink() || kind.is_hard_link() {
            let link = entry.link_name().map_err(tar_error)?.unwrap_or_default();
            let resolved = if kind.is_hard_link() {
                dest.join(&link)
            } else {
                target.parent().unwrap_or(dest).join(&link)
            };
            if !inside(dest, &resolved) {
                return Err(fail(format!("Enlace inseguro en el archivo: {name}")));
            }
        }
    }

    // Segunda pasada: extraer con las reglas del filtro "data" de tarfile.
    let mut tar = open_tar(archive)?;
    for entry in tar.entries().map_err(tar_error)? {
        let mut entry = entry.map_err(tar_error)?;
        let name = String::from_utf8_lossy(&entry.path_bytes()).into_owned();
        let relative = entry.path().map_err(tar_error)?.into_owned();
        let target = dest.join(relative.strip_prefix("/").unwrap_or(&relative));
        let kind = entry.header().entry_type();
        match kind {
            EntryType::Directory => {
                fs::create_dir_all(&target)
                    .with_context(|| format!("No se pudo crear {}", target.display()))?;
            }
            EntryType::Regular | EntryType::Continuous | EntryType::GNUSparse => {
                prepare_parent(&target)?;
                let mut out = File::create(&target)
                    .with_context(|| format!("No se pudo crear {}", target.display()))?;
                io::copy(&mut entry, &mut out)
                    .with_context(|| format!("No se pudo extraer {}", target.display()))?;
                let mtime = UNIX_EPOCH + Duration::from_secs(entry.header().mtime().unwrap_or(0));
                let _ = out.set_times(FileTimes::new().set_accessed(mtime).set_modified(mtime));
                set_data_mode(&target, entry.header().mode().unwrap_or(0o644))?;
            }
            EntryType::Symlink => {
                let link = entry
                    .link_name()
                    .map_err(tar_error)?
                    .unwrap_or_default()
                    .into_owned();
                prepare_parent(&target)?;
                make_symlink(&link, &target)?;
            }
            EntryType::Link => {
                let link = entry
                    .link_name()
                    .map_err(tar_error)?
                    .unwrap_or_default()
                    .into_owned();
                prepare_parent(&target)?;
                let source = dest.join(link);
                if fs::hard_link(&source, &target).is_err() {
                    fs::copy(&source, &target)
                        .with_context(|| format!("No se pudo extraer {}", target.display()))?;
                }
                set_data_mode(&target, entry.header().mode().unwrap_or(0o644))?;
            }
            EntryType::Char | EntryType::Block | EntryType::Fifo => {
                return Err(unsafe_archive(&name));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Crea la carpeta del archivo y quita lo que hubiera en su lugar (como tarfile).
fn prepare_parent(target: &Path) -> anyhow::Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("No se pudo crear {}", parent.display()))?;
    }
    if fs::symlink_metadata(target).is_ok_and(|meta| !meta.is_dir()) {
        let _ = fs::remove_file(target);
    }
    Ok(())
}

/// Permisos del filtro "data": sin setuid ni escritura para grupo y otros; sin
/// ejecucion si el dueno no la tiene; lectura y escritura para el dueno.
#[cfg(unix)]
fn set_data_mode(path: &Path, mode: u32) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut mode = mode & 0o755;
    if mode & 0o100 == 0 {
        mode &= !0o111;
    }
    mode |= 0o600;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .with_context(|| format!("No se pudieron cambiar los permisos de {}", path.display()))
}

#[cfg(not(unix))]
fn set_data_mode(_path: &Path, _mode: u32) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn make_symlink(link: &Path, target: &Path) -> anyhow::Result<()> {
    std::os::unix::fs::symlink(link, target)
        .with_context(|| format!("No se pudo crear el enlace {}", target.display()))
}

/// En Windows crear enlaces necesita permisos especiales; como tarfile, si no se
/// puede se copia el archivo al que apunta.
#[cfg(windows)]
fn make_symlink(link: &Path, target: &Path) -> anyhow::Result<()> {
    if std::os::windows::fs::symlink_file(link, target).is_ok() {
        return Ok(());
    }
    let source = target.parent().unwrap_or(Path::new(".")).join(link);
    if source.is_file() {
        fs::copy(&source, target)
            .with_context(|| format!("No se pudo extraer {}", target.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::user_message;
    use std::io::Write;

    fn zip_with(path: &Path, files: &[(&str, &[u8])]) {
        let mut zip = zip::ZipWriter::new(File::create(path).unwrap());
        for (name, data) in files {
            if name.ends_with('/') {
                zip.add_directory(*name, zip::write::SimpleFileOptions::default())
                    .unwrap();
            } else {
                zip.start_file(*name, zip::write::SimpleFileOptions::default())
                    .unwrap();
                zip.write_all(data).unwrap();
            }
        }
        zip.finish().unwrap();
    }

    fn tar_gz_with(
        path: &Path,
        build: impl FnOnce(&mut tar::Builder<flate2::write::GzEncoder<File>>),
    ) {
        let gz = flate2::write::GzEncoder::new(
            File::create(path).unwrap(),
            flate2::Compression::default(),
        );
        let mut builder = tar::Builder::new(gz);
        build(&mut builder);
        builder.into_inner().unwrap().finish().unwrap();
    }

    fn file_header(size: u64, mode: u32) -> tar::Header {
        let mut header = tar::Header::new_gnu();
        header.set_size(size);
        header.set_mode(mode);
        header.set_entry_type(EntryType::Regular);
        header
    }

    /// Escribe una entrada sin las comprobaciones de `append_data` (para poder
    /// probar nombres peligrosos).
    fn raw_entry(
        builder: &mut tar::Builder<impl Write>,
        mut header: tar::Header,
        name: &str,
        data: &[u8],
    ) {
        let bytes = name.as_bytes();
        header.as_old_mut().name[..bytes.len()].copy_from_slice(bytes);
        header.set_cksum();
        builder.append(&header, data).unwrap();
    }

    #[test]
    fn zip_normal() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("a.zip");
        zip_with(
            &archive,
            &[("bin/", b""), ("bin/java", b"x"), ("lib/a/b.txt", b"hola")],
        );
        let dest = tmp.path().join("dest");
        safe_extract(&archive, &dest).unwrap();
        assert_eq!(
            fs::read(dest.join("lib").join("a").join("b.txt")).unwrap(),
            b"hola"
        );
        assert!(dest.join("bin").join("java").is_file());
    }

    #[test]
    fn zip_que_sale_de_la_carpeta_no_extrae_nada() {
        let tmp = tempfile::tempdir().unwrap();
        for bad in ["../fuera.txt", "a/../../fuera.txt", "/abs.txt"] {
            let archive = tmp.path().join("malo.zip");
            zip_with(&archive, &[("bueno.txt", b"x"), (bad, b"x")]);
            let dest = tmp.path().join("dest");
            let err = safe_extract(&archive, &dest).unwrap_err();
            assert_eq!(
                user_message(&err),
                format!("Archivo comprimido inseguro: {bad}")
            );
            assert!(!dest.join("bueno.txt").exists(), "{bad}: no se extrae nada");
        }
        assert!(!tmp.path().join("fuera.txt").exists());
    }

    #[test]
    fn zip_sin_puntos_como_zipfile() {
        assert_eq!(
            zip_member_path("a/./b/../c.txt"),
            Path::new("a").join("b").join("c.txt")
        );
    }

    #[test]
    fn tar_gz_normal_con_enlace_interno() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("jre.tar.gz");
        tar_gz_with(&archive, |b| {
            raw_entry(b, file_header(4, 0o755), "jre/bin/java", b"java");
            raw_entry(b, file_header(2, 0o666), "jre/lib/x.txt", b"ok");
            let mut link = tar::Header::new_gnu();
            link.set_entry_type(EntryType::Symlink);
            link.set_size(0);
            link.set_link_name("../lib/x.txt").unwrap();
            raw_entry(b, link, "jre/bin/x.txt", b"");
        });
        let dest = tmp.path().join("dest");
        safe_extract(&archive, &dest).unwrap();
        assert_eq!(
            fs::read(dest.join("jre").join("bin").join("java")).unwrap(),
            b"java"
        );
        assert_eq!(
            fs::read(dest.join("jre").join("bin").join("x.txt")).unwrap(),
            b"ok"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = |p: &Path| fs::metadata(p).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode(&dest.join("jre/bin/java")), 0o755);
            assert_eq!(
                mode(&dest.join("jre/lib/x.txt")),
                0o644,
                "sin escritura para grupo ni otros"
            );
        }
    }

    #[test]
    fn tar_con_ruta_o_enlace_fuera() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("malo.tar.gz");
        tar_gz_with(&archive, |b| {
            raw_entry(b, file_header(1, 0o644), "../fuera.txt", b"x")
        });
        let err = safe_extract(&archive, &tmp.path().join("d1")).unwrap_err();
        assert_eq!(
            user_message(&err),
            "Archivo comprimido inseguro: ../fuera.txt"
        );

        tar_gz_with(&archive, |b| {
            let mut link = tar::Header::new_gnu();
            link.set_entry_type(EntryType::Symlink);
            link.set_size(0);
            link.set_link_name("../../../etc/passwd").unwrap();
            raw_entry(b, link, "a/enlace", b"");
        });
        let err = safe_extract(&archive, &tmp.path().join("d2")).unwrap_err();
        assert_eq!(
            user_message(&err),
            "Enlace inseguro en el archivo: a/enlace"
        );

        tar_gz_with(&archive, |b| {
            let mut link = tar::Header::new_gnu();
            link.set_entry_type(EntryType::Link);
            link.set_size(0);
            link.set_link_name("../fuera").unwrap();
            raw_entry(b, link, "duro", b"");
        });
        let err = safe_extract(&archive, &tmp.path().join("d3")).unwrap_err();
        assert_eq!(user_message(&err), "Enlace inseguro en el archivo: duro");
        assert!(!tmp.path().join("d3").join("duro").exists());
    }
}
