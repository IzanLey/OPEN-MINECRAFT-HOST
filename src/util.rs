//! Utilidades que reproducen funciones de `legacy/omh.py` con el mismo resultado.

use std::borrow::Cow;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::Context;
use regex::Regex;
use serde_json::Value;

use crate::error::fail;

pub const MB: u64 = 1024 * 1024;

fn regex(pattern: &str) -> Regex {
    Regex::new(pattern).expect("expresion regular fija y probada en los tests")
}

static DIGITS: LazyLock<Regex> = LazyLock::new(|| regex(r"[0-9]+"));
static RELEASE: LazyLock<Regex> = LazyLock::new(|| regex(r"^[0-9]+(\.[0-9]+)+$"));
static SIZE: LazyLock<Regex> =
    LazyLock::new(|| regex(r"(?i)^\s*([0-9]+(?:\.[0-9]+)?)\s*([KMGT]?)(?:I?B)?\s*$"));
static PLAYER: LazyLock<Regex> = LazyLock::new(|| regex(r"^[A-Za-z0-9_ ]{1,32}$"));

/// Convierte '1.20.4' en [1, 20, 4] para comparar versiones. Sin numeros, [0].
pub fn vt(version: &str) -> Vec<u64> {
    let nums: Vec<u64> = DIGITS
        .find_iter(version)
        .map(|m| m.as_str().parse().unwrap_or(u64::MAX))
        .collect();
    if nums.is_empty() { vec![0] } else { nums }
}

/// Version normal de Minecraft: al menos dos numeros separados por puntos.
pub fn is_release(version: &str) -> bool {
    RELEASE.is_match(version)
}

/// '4G', '4096M', '4GB' o '4' (GB) -> megabytes.
pub fn parse_size_mb(text: &str) -> anyhow::Result<i64> {
    let invalid = || {
        fail(format!(
            "Tamano no valido: '{text}'. Usa por ejemplo 4G o 4096M."
        ))
    };
    let caps = SIZE.captures(text).ok_or_else(invalid)?;
    let number: f64 = caps
        .get(1)
        .map_or("", |m| m.as_str())
        .parse()
        .map_err(|_| invalid())?;
    let factor = match caps.get(2).map_or("", |m| m.as_str()) {
        "K" | "k" => 1.0 / 1024.0,
        "M" | "m" => 1.0,
        "T" | "t" => 1024.0 * 1024.0,
        _ => 1024.0,
    };
    // Como int() de Python: trunca hacia cero.
    Ok((number * factor) as i64)
}

/// 4096 -> "4.0 GB", 512 -> "512 MB".
pub fn fmt_mb(mb: i64) -> String {
    if mb >= 1024 {
        format!("{:.1} GB", mb as f64 / 1024.0)
    } else {
        format!("{mb} MB")
    }
}

/// Nombre de jugador de Java o Bedrock (letras, numeros, `_` y espacios; 1 a 32).
pub fn valid_player(name: &str) -> anyhow::Result<String> {
    if !PLAYER.is_match(name) {
        return Err(fail(format!("Nombre de jugador no valido: '{name}'")));
    }
    Ok(py_strip(name).to_owned())
}

/// `str.isspace()` de Python, que ademas de los espacios de Unicode cuenta
/// los separadores ASCII 0x1C-0x1F.
pub fn py_is_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// `str.strip()` de Python.
pub fn py_strip(text: &str) -> &str {
    text.trim_matches(py_is_space)
}

/// `str.splitlines()` de Python: corta en `\n`, `\r\n`, `\r` y los demas saltos
/// de linea de Unicode, sin dejar una linea vacia al final.
pub fn py_splitlines(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let is_break = matches!(
            c,
            '\n' | '\r'
                | '\u{b}'
                | '\u{c}'
                | '\u{1c}'
                | '\u{1d}'
                | '\u{1e}'
                | '\u{85}'
                | '\u{2028}'
                | '\u{2029}'
        );
        if !is_break {
            continue;
        }
        lines.push(&text[start..i]);
        start = i + c.len_utf8();
        if c == '\r' && chars.peek().is_some_and(|&(_, next)| next == '\n') {
            chars.next();
            start += 1;
        }
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

/// Lineas de un archivo leido en modo texto por Python (`for linea in f`): solo
/// cuentan como salto `\n`, `\r\n` y `\r`.
pub fn text_lines(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                lines.push(&text[start..i]);
                start = i + 1;
            }
            b'\r' => {
                lines.push(&text[start..i]);
                if bytes.get(i + 1) == Some(&b'\n') {
                    i += 1;
                }
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

/// `lista[-n:]` de Python: con 0 devuelve todo y con negativos se salta el principio.
pub fn py_tail<T>(items: &[T], count: i64) -> &[T] {
    let len = i64::try_from(items.len()).unwrap_or(i64::MAX);
    let start = if count > 0 {
        (len - count).max(0)
    } else {
        count.saturating_neg().min(len)
    };
    &items[usize::try_from(start).unwrap_or(0)..]
}

/// Valor de verdad de Python para un valor JSON.
pub fn py_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// `os.path.abspath`: ruta absoluta normalizada sin resolver enlaces.
pub fn abspath(path: &Path) -> PathBuf {
    if path.as_os_str().is_empty() {
        return env::current_dir().unwrap_or_default();
    }
    #[cfg(windows)]
    {
        // Como Python: GetFullPathNameW.
        std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
    }
    #[cfg(not(windows))]
    {
        let joined = if path.is_absolute() {
            path.to_path_buf()
        } else {
            env::current_dir().unwrap_or_default().join(path)
        };
        normpath(&joined)
    }
}

/// `posixpath.normpath`: quita `.`, resuelve `..` y separadores repetidos sin tocar
/// el disco. Conserva exactamente dos barras iniciales, como POSIX.
#[cfg(not(windows))]
fn normpath(path: &Path) -> PathBuf {
    use std::ffi::OsString;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    let bytes = path.as_os_str().as_bytes();
    if bytes.is_empty() {
        return PathBuf::from(".");
    }
    let initial = if bytes.starts_with(b"//") && !bytes.starts_with(b"///") {
        2
    } else {
        usize::from(bytes.starts_with(b"/"))
    };
    let mut comps: Vec<&[u8]> = Vec::new();
    for comp in bytes.split(|&b| b == b'/') {
        if comp.is_empty() || comp == b"." {
            continue;
        }
        let keep_dotdot = (initial == 0 && comps.is_empty()) || comps.last() == Some(&&b".."[..]);
        if comp != b".." || keep_dotdot {
            comps.push(comp);
        } else {
            comps.pop();
        }
    }
    let mut out = vec![b'/'; initial];
    out.extend(comps.join(&b'/'));
    if out.is_empty() {
        out.push(b'.');
    }
    PathBuf::from(OsString::from_vec(out))
}

/// Texto con los saltos de linea que usa Python al escribir en modo texto:
/// `\r\n` en Windows y `\n` en Linux.
pub fn native_newlines(text: &str) -> Cow<'_, str> {
    if cfg!(windows) {
        Cow::Owned(text.replace('\n', "\r\n"))
    } else {
        Cow::Borrowed(text)
    }
}

/// Escribe un archivo de texto como `open(ruta, "w", encoding="utf-8")` de Python.
pub fn write_text(path: &Path, text: &str) -> anyhow::Result<()> {
    fs::write(path, native_newlines(text).as_bytes())
        .with_context(|| format!("No se pudo escribir {}", path.display()))
}

/// Lee un archivo en UTF-8 cambiando los bytes no validos por U+FFFD
/// (`errors="replace"` de Python).
pub fn read_text_lossy(path: &Path) -> io::Result<String> {
    fs::read(path).map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

/// Fecha y hora local como `time.strftime("%Y-%m-%d %H:%M:%S")`.
pub fn local_timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

// ---------------------------------------------------------------------------
// Rutas como os.path de Python
// ---------------------------------------------------------------------------

fn is_sep(c: char) -> bool {
    c == '/' || (cfg!(windows) && c == '\\')
}

/// Quita la letra de unidad (`C:`) como `ntpath.splitdrive`; en Linux no hace nada.
fn without_drive(path: &str) -> &str {
    if cfg!(windows) && path.as_bytes().get(1) == Some(&b':') {
        &path[2..]
    } else {
        path
    }
}

/// `os.path.basename`: en Windows tambien corta por `\` y quita la unidad.
pub fn py_basename(path: &str) -> &str {
    let path = without_drive(path);
    match path.rfind(is_sep) {
        Some(i) => &path[i + 1..],
        None => path,
    }
}

/// `os.path.relpath` para una ruta que esta dentro de `base` (con el separador del sistema).
pub fn relpath(path: &Path, base: &Path) -> String {
    let path = abspath(path);
    match path.strip_prefix(abspath(base)) {
        Ok(rel) if rel.as_os_str().is_empty() => ".".to_owned(),
        Ok(rel) => rel.to_string_lossy().into_owned(),
        Err(_) => path.to_string_lossy().into_owned(),
    }
}

/// `os.path.expanduser`: `~` y `~usuario` al principio de la ruta.
pub fn expanduser(path: &str) -> PathBuf {
    if !path.starts_with('~') {
        return PathBuf::from(path);
    }
    let end = path[1..].find(is_sep).map_or(path.len(), |i| i + 1);
    match user_home(&path[1..end]) {
        Some(home) => join_home(home, &path[end..]),
        None => PathBuf::from(path),
    }
}

#[cfg(windows)]
fn user_home(user: &str) -> Option<PathBuf> {
    let mut home = match env::var_os("USERPROFILE") {
        Some(profile) => PathBuf::from(profile),
        None => {
            let homepath = env::var_os("HOMEPATH")?;
            PathBuf::from(env::var_os("HOMEDRIVE").unwrap_or_default()).join(homepath)
        }
    };
    if !user.is_empty() {
        let current = env::var("USERNAME").ok();
        if current.as_deref() != Some(user) {
            // Como Python: se supone que los perfiles estan todos en la misma carpeta.
            let home_name = home.file_name().map(|n| n.to_string_lossy().into_owned());
            if current != home_name {
                return None;
            }
            home = home.parent()?.join(user);
        }
    }
    Some(home)
}

#[cfg(windows)]
fn join_home(home: PathBuf, rest: &str) -> PathBuf {
    let mut joined = home.into_os_string();
    joined.push(rest);
    PathBuf::from(joined)
}

#[cfg(not(windows))]
fn user_home(user: &str) -> Option<PathBuf> {
    if user.is_empty()
        && let Some(home) = env::var_os("HOME")
    {
        return Some(PathBuf::from(home));
    }
    crate::platform::passwd_home(user)
}

#[cfg(not(windows))]
fn join_home(home: PathBuf, rest: &str) -> PathBuf {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let mut bytes = home.into_os_string().into_vec();
    while bytes.last() == Some(&b'/') {
        bytes.pop();
    }
    bytes.extend_from_slice(rest.as_bytes());
    if bytes.is_empty() {
        bytes.push(b'/');
    }
    PathBuf::from(OsString::from_vec(bytes))
}

/// Compara nombres de archivo como `fnmatch`: sin distinguir mayusculas en Windows.
pub fn name_key(name: &str) -> Cow<'_, str> {
    if cfg!(windows) {
        Cow::Owned(name.to_lowercase())
    } else {
        Cow::Borrowed(name)
    }
}

/// Recorre `root` como `os.walk` (de arriba abajo, en el orden del sistema de
/// archivos y sin entrar en enlaces a carpetas) hasta que `found` devuelve algo
/// para una carpeta y sus archivos.
pub fn walk_find<T, F>(root: &Path, found: &mut F) -> Option<T>
where
    F: FnMut(&Path, &[String]) -> Option<T>,
{
    let entries = fs::read_dir(root).ok()?;
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.path().is_dir() {
            let is_link = entry.file_type().is_ok_and(|kind| kind.is_symlink());
            dirs.push((name, is_link));
        } else {
            files.push(name);
        }
    }
    if let Some(hit) = found(root, &files) {
        return Some(hit);
    }
    dirs.into_iter()
        .filter(|(_, is_link)| !is_link)
        .find_map(|(name, _)| walk_find(&root.join(name), found))
}

/// `glob.glob(carpeta/patron)`: entradas no ocultas de `dir` cuyo nombre cumple
/// `matches`, en el orden del sistema de archivos.
pub fn glob_names(dir: &Path, matches: &dyn Fn(&str) -> bool) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            !name.starts_with('.') && matches(&name)
        })
        .map(|entry| entry.path())
        .collect()
}

/// `glob.glob(carpeta/**/patron, recursive=True)`: la carpeta y todas sus
/// subcarpetas no ocultas, de arriba abajo.
pub fn glob_recursive(dir: &Path, matches: &dyn Fn(&str) -> bool) -> Vec<PathBuf> {
    let mut found = glob_names(dir, matches);
    for sub in glob_names(dir, &|_| true) {
        let is_link = fs::symlink_metadata(&sub).is_ok_and(|meta| meta.file_type().is_symlink());
        if sub.is_dir() && !is_link {
            found.extend(glob_recursive(&sub, matches));
        }
    }
    found
}

// ---------------------------------------------------------------------------
// URLs como urllib.parse
// ---------------------------------------------------------------------------

/// `urllib.parse.quote(texto, safe)`: todo como `%XX` salvo letras, numeros,
/// `_.-~` y los caracteres de `safe`.
pub fn py_quote(text: &str, safe: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        let keep = byte.is_ascii_alphanumeric()
            || b"_.-~".contains(&byte)
            || (byte.is_ascii() && safe.contains(char::from(byte)));
        if keep {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// `urllib.parse.urlencode` con los pares en orden (espacios como `+`).
pub fn urlencode(pairs: &[(&str, &str)]) -> String {
    let quote_plus = |text: &str| py_quote(text, " ").replace(' ', "+");
    pairs
        .iter()
        .map(|(key, value)| format!("{}={}", quote_plus(key), quote_plus(value)))
        .collect::<Vec<_>>()
        .join("&")
}

/// Ultimas lineas de un archivo de texto (`tail_lines` de Python, con la misma
/// semantica de `lista[-n:]`). Si no se puede leer, ninguna.
pub fn tail_lines(path: &Path, count: i64) -> Vec<String> {
    read_text_lossy(path)
        .map(|text| {
            py_tail(&py_splitlines(&text), count)
                .iter()
                .map(|line| (*line).to_owned())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Los valores esperados salen de ejecutar las mismas funciones de legacy/omh.py.

    #[test]
    fn tamanos_como_python() {
        let cases = [
            ("4G", 4096),
            ("4096M", 4096),
            ("4GB", 4096),
            ("4", 4096),
            ("1.5G", 1536),
            ("2GiB", 2048),
            ("512K", 0),
            (" 8 g ", 8192),
            ("1T", 1048576),
            ("0.5", 512),
            ("10gb", 10240),
            ("3mb", 3),
            ("1.9999k", 0),
            ("7tib", 7340032),
        ];
        for (text, mb) in cases {
            assert_eq!(parse_size_mb(text).unwrap(), mb, "{text}");
        }
    }

    #[test]
    fn tamanos_no_validos_como_python() {
        for text in ["abc", "4X", "4 G B", "", "-1G", "1e3"] {
            let err = parse_size_mb(text).unwrap_err();
            assert_eq!(
                err.to_string(),
                format!("Tamano no valido: '{text}'. Usa por ejemplo 4G o 4096M.")
            );
        }
    }

    #[test]
    fn fmt_mb_como_python() {
        let cases = [
            (0, "0 MB"),
            (1, "1 MB"),
            (512, "512 MB"),
            (1023, "1023 MB"),
            (1024, "1.0 GB"),
            (1100, "1.1 GB"),
            (1280, "1.2 GB"),
            (1536, "1.5 GB"),
            (1792, "1.8 GB"),
            (2304, "2.2 GB"),
            (2816, "2.8 GB"),
            (4096, "4.0 GB"),
            (10240, "10.0 GB"),
            (15847, "15.5 GB"),
            (20480, "20.0 GB"),
            (1048576, "1024.0 GB"),
        ];
        for (mb, text) in cases {
            assert_eq!(fmt_mb(mb), text, "{mb}");
        }
    }

    #[test]
    fn vt_como_python() {
        assert_eq!(vt("1.20.4"), [1, 20, 4]);
        assert_eq!(vt("1.21.1-rc1"), [1, 21, 1, 1]);
        assert_eq!(vt("abc"), [0]);
        assert_eq!(vt(""), [0]);
        assert_eq!(vt("26.1"), [26, 1]);
        assert_eq!(vt("21.1.77-beta"), [21, 1, 77]);
        // Como las tuplas de Python: la mas corta va antes si empiezan igual.
        assert!(vt("1.20") < vt("1.20.0"));
        assert!(vt("1.9.4") < vt("1.10"));
        assert!(vt("1.21.11") > vt("1.21.4"));
    }

    #[test]
    fn is_release_como_python() {
        for (v, ok) in [
            ("1.21", true),
            ("26.1.2", true),
            ("1", false),
            ("1.21-pre1", false),
            ("1.21.", false),
            (".1", false),
            ("1..2", false),
            ("1.2\n", false),
        ] {
            assert_eq!(is_release(v), ok, "{v:?}");
        }
    }

    #[test]
    fn jugadores_como_python() {
        assert_eq!(valid_player("Steve").unwrap(), "Steve");
        assert_eq!(valid_player("Steve Jobs").unwrap(), "Steve Jobs");
        assert_eq!(valid_player(" Steve ").unwrap(), "Steve");
        assert_eq!(valid_player(&"a".repeat(32)).unwrap(), "a".repeat(32));
        assert_eq!(valid_player("under_score").unwrap(), "under_score");
        for bad in ["", "x-y", "\u{d1}and\u{fa}"] {
            assert_eq!(
                valid_player(bad).unwrap_err().to_string(),
                format!("Nombre de jugador no valido: '{bad}'")
            );
        }
        assert!(valid_player(&"a".repeat(33)).is_err());
    }

    #[test]
    fn splitlines_como_python() {
        let text = "a\r\nb\rc\nd\u{b}e\u{c}f\u{1c}g\u{1d}h\u{1e}i\u{85}j\u{2028}k\u{2029}l\n";
        assert_eq!(
            py_splitlines(text),
            ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l"]
        );
        assert!(py_splitlines("").is_empty());
        assert_eq!(py_splitlines("\n"), [""]);
        assert_eq!(py_splitlines("a\n\nb"), ["a", "", "b"]);
        assert_eq!(py_splitlines("x\r\n\r\ny"), ["x", "", "y"]);
        assert_eq!(py_splitlines("\r"), [""]);
        assert_eq!(py_splitlines("a\u{1f}b"), ["a\u{1f}b"]);
    }

    #[test]
    fn lineas_de_archivo_como_python() {
        assert_eq!(
            text_lines("a\r\nb\rc\nd\u{c}e\n"),
            ["a", "b", "c", "d\u{c}e"]
        );
        assert!(text_lines("").is_empty());
        assert_eq!(text_lines("x"), ["x"]);
    }

    #[test]
    fn strip_como_python() {
        assert_eq!(py_strip("\u{1c} a \u{1f}"), "a");
        assert_eq!(py_strip("\u{3000}x\u{a0}"), "x");
        assert_eq!(py_strip("\t\n y \u{b}\u{c}"), "y");
        assert_eq!(py_strip("\u{1d}"), "");
    }

    #[test]
    fn tail_como_slices_de_python() {
        let items: Vec<i32> = (1..=10).collect();
        assert_eq!(py_tail(&items, 3), [8, 9, 10]);
        assert_eq!(py_tail(&items, 0), &items[..]);
        assert_eq!(py_tail(&items, -3), [4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(py_tail(&items, 20), &items[..]);
        assert!(py_tail(&items, -20).is_empty());
        assert_eq!(py_tail(&items, 1), [10]);
        assert!(py_tail(&items, i64::MIN).is_empty());
    }

    #[test]
    fn truthy_como_python() {
        use serde_json::json;
        assert!(py_truthy(&json!(true)));
        assert!(py_truthy(&json!(1)));
        assert!(py_truthy(&json!("x")));
        for falsy in [
            json!(null),
            json!(false),
            json!(0),
            json!(""),
            json!([]),
            json!({}),
        ] {
            assert!(!py_truthy(&falsy), "{falsy}");
        }
    }

    #[test]
    fn abspath_normaliza_sin_tocar_el_disco() {
        let cwd = env::current_dir().unwrap();
        assert_eq!(abspath(Path::new("")), cwd);
        assert_eq!(abspath(Path::new("a/./b/../c")), cwd.join("a").join("c"));
    }

    #[cfg(not(windows))]
    #[test]
    fn normpath_como_posixpath() {
        for (input, expected) in [
            ("/a/b/../c", "/a/c"),
            ("/../x", "/x"),
            ("//srv/x/", "//srv/x"),
            ("///srv", "/srv"),
            ("/a//b/./c/..", "/a/b"),
            ("/", "/"),
        ] {
            assert_eq!(
                normpath(Path::new(input)),
                PathBuf::from(expected),
                "{input}"
            );
        }
    }

    #[test]
    fn saltos_de_linea_del_sistema() {
        let expected = if cfg!(windows) {
            "a\r\nb\r\n"
        } else {
            "a\nb\n"
        };
        assert_eq!(native_newlines("a\nb\n"), expected);
    }

    // Valores de os.path y urllib.parse de Python en cada sistema.
    #[test]
    fn basename_como_python() {
        assert_eq!(py_basename("y.jar"), "y.jar");
        assert_eq!(py_basename("/srv/x/"), "");
        if cfg!(windows) {
            assert_eq!(py_basename(r"C:\x\y.jar"), "y.jar");
            assert_eq!(py_basename(r"a/b\c.jar"), "c.jar");
            assert_eq!(py_basename("C:"), "");
        } else {
            assert_eq!(py_basename(r"C:\x\y.jar"), r"C:\x\y.jar");
            assert_eq!(py_basename(r"a/b\c.jar"), r"b\c.jar");
            assert_eq!(py_basename("C:"), "C:");
        }
    }

    #[test]
    fn relpath_con_el_separador_del_sistema() {
        let base = env::current_dir().unwrap();
        let expected = if cfg!(windows) { r"a\b\c" } else { "a/b/c" };
        assert_eq!(
            relpath(&base.join("a").join("b").join("c"), &base),
            expected
        );
        assert_eq!(relpath(&base, &base), ".");
    }

    #[test]
    fn expanduser_como_python() {
        assert_eq!(expanduser("x/~"), PathBuf::from("x/~"));
        assert_ne!(expanduser("~"), PathBuf::from("~"));
        assert_ne!(expanduser("~/a"), PathBuf::from("~/a"));
        // Un usuario que no existe: Linux deja la ruta igual; Windows supone que su
        // perfil esta junto al del usuario actual.
        let unknown = "~nadie_que_exista_123/a";
        assert_eq!(expanduser(unknown) == Path::new(unknown), !cfg!(windows));
    }

    #[test]
    fn quote_y_urlencode_como_python() {
        assert_eq!(py_quote("worldedit", "/"), "worldedit");
        assert_eq!(py_quote("foo bar/baz", "/"), "foo%20bar/baz");
        assert_eq!(py_quote("caf\u{e9}", "/"), "caf%C3%A9");
        assert_eq!(py_quote("a+b&c=d", "/"), "a%2Bb%26c%3Dd");
        assert_eq!(py_quote("~x_y.z-", "/"), "~x_y.z-");
        assert_eq!(
            urlencode(&[
                ("loaders", "[\"paper\", \"spigot\", \"bukkit\", \"purpur\"]"),
                ("game_versions", "[\"1.21.1\"]")
            ]),
            "loaders=%5B%22paper%22%2C+%22spigot%22%2C+%22bukkit%22%2C+%22purpur%22%5D&game_versions=%5B%221.21.1%22%5D"
        );
    }

    #[test]
    fn recorridos_como_os_walk_y_glob() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("jre").join("bin")).unwrap();
        fs::create_dir_all(root.join(".oculta")).unwrap();
        fs::write(root.join("jre").join("bin").join("java"), "").unwrap();
        fs::write(root.join("a.jar"), "").unwrap();
        fs::write(root.join(".oculta").join("b.jar"), "").unwrap();
        fs::write(root.join("jre").join("c.jar"), "").unwrap();

        let found = walk_find(root, &mut |dir: &Path, files: &[String]| {
            (dir.file_name().is_some_and(|n| n == "bin") && files.iter().any(|f| f == "java"))
                .then(|| dir.join("java"))
        });
        assert_eq!(found, Some(root.join("jre").join("bin").join("java")));

        let jars = glob_recursive(root, &|name: &str| name.ends_with(".jar"));
        assert_eq!(jars, [root.join("a.jar"), root.join("jre").join("c.jar")]);
    }
}
