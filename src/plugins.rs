//! `omh plugin`: buscar, instalar, listar o quitar plugins y mods
//! (Modrinth, Poggit, una URL o un archivo local).

use std::collections::HashSet;
use std::fs::{self, File, FileTimes};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::Context;
use regex::Regex;
use serde_json::{Value, json};

use crate::catalog::{self, Addons};
use crate::cli::PluginArgs;
use crate::config::{self, ServerConfig};
use crate::error::{fail, http_status};
use crate::http::{self, Checksums};
use crate::pyjson;
use crate::ui;
use crate::util::{py_basename, py_is_space, py_quote, py_truthy, urlencode, vt};

static UNSAFE_CHARS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^A-Za-z0-9_.+-]").expect("expresion regular fija y probada"));

/// Carpeta de complementos del core (`plugins/` o `mods/`), creandola si falta.
pub fn addons_dir(cfg: &ServerConfig) -> anyhow::Result<(PathBuf, Addons)> {
    let Some(kind) = catalog::core(&cfg.edition, &cfg.core).and_then(|core| core.addons) else {
        return Err(fail(format!(
            "El core {} no admite plugins ni mods. Usa paper/purpur (plugins) o fabric/forge/neoforge (mods).",
            cfg.core
        )));
    };
    let path = cfg.dir.join(if kind == Addons::Plugins {
        "plugins"
    } else {
        "mods"
    });
    fs::create_dir_all(&path).with_context(|| format!("No se pudo crear {}", path.display()))?;
    Ok((path, kind))
}

/// `urllib.parse.unquote`: cambia los `%XX` por sus bytes y los lee como UTF-8.
fn unquote(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes
            .get(i + 1..i + 3)
            .filter(|pair| pair.iter().all(u8::is_ascii_hexdigit))
            .and_then(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok());
        match (bytes[i], hex) {
            (b'%', Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Nombre de archivo seguro a partir de una URL o un nombre.
pub fn safe_filename(name: &str) -> String {
    let without_query = name.split('?').next().unwrap_or_default();
    let decoded = unquote(without_query);
    let cleaned = UNSAFE_CHARS.replace_all(py_basename(&decoded), "_");
    if cleaned.is_empty() {
        "addon.jar".to_owned()
    } else {
        cleaned.into_owned()
    }
}

/// `shutil.copy2`: copia el contenido y las fechas.
fn copy2(source: &Path, dest: &Path) -> anyhow::Result<()> {
    fs::copy(source, dest).with_context(|| format!("No se pudo copiar {}", source.display()))?;
    if let Ok(meta) = fs::metadata(source) {
        let mut times = FileTimes::new();
        if let Ok(modified) = meta.modified() {
            times = times.set_modified(modified);
        }
        if let Ok(accessed) = meta.accessed() {
            times = times.set_accessed(accessed);
        }
        if let Ok(file) = File::options().write(true).open(dest) {
            let _ = file.set_times(times);
        }
    }
    Ok(())
}

fn modrinth_install(
    cfg: &ServerConfig,
    folder: &Path,
    project: &str,
    seen: &mut HashSet<String>,
) -> anyhow::Result<()> {
    if !seen.insert(project.to_owned()) {
        return Ok(());
    }
    let loaders = catalog::modrinth_loaders(&cfg.core).unwrap_or_default();
    let params = urlencode(&[
        ("loaders", &pyjson::to_compact(&loaders)?),
        ("game_versions", &pyjson::to_compact(&[&cfg.version])?),
    ]);
    let url = format!(
        "https://api.modrinth.com/v2/project/{}/version?{params}",
        py_quote(project, "/")
    );
    let versions = match http::get_json(&url, &[]) {
        Ok(versions) => versions,
        Err(err) if http_status(&err) == Some(404) => {
            return Err(fail(format!(
                "No existe el proyecto '{project}' en Modrinth. Prueba: omh plugin search {project}"
            )));
        }
        Err(err) => return Err(err),
    };
    let Some(version) = versions.as_array().and_then(|list| list.first()) else {
        return Err(fail(format!(
            "'{project}' no tiene version para {} {}.",
            cfg.core, cfg.version
        )));
    };
    let files = version
        .get("files")
        .and_then(Value::as_array)
        .filter(|files| !files.is_empty())
        .ok_or_else(|| fail("Respuesta inesperada de Modrinth: la version no tiene archivos."))?;
    let file = files
        .iter()
        .find(|file| file.get("primary").is_some_and(py_truthy))
        .unwrap_or(&files[0]);
    let filename = http::str_field(file, "filename")?;
    let sha1 = file
        .get("hashes")
        .and_then(|hashes| hashes.get("sha1"))
        .and_then(Value::as_str);
    http::download(
        http::str_field(file, "url")?,
        &folder.join(safe_filename(filename)),
        Checksums::sha1(sha1),
        Some(filename),
    )?;
    for dep in version
        .get("dependencies")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let required = dep.get("dependency_type").and_then(Value::as_str) == Some("required");
        if required
            && let Some(id) = dep
                .get("project_id")
                .and_then(Value::as_str)
                .filter(|id| !id.is_empty())
        {
            ui::info("Instalando dependencia necesaria...");
            modrinth_install(cfg, folder, id, seen)?;
        }
    }
    Ok(())
}

fn search(cfg: &ServerConfig, kind: Addons, target: &str) -> anyhow::Result<()> {
    let Some(loaders) = catalog::modrinth_loaders(&cfg.core) else {
        return Err(fail(
            "La busqueda usa Modrinth y solo funciona en servidores de Java. Para PocketMine busca en https://poggit.pmmp.io",
        ));
    };
    let project_type = if kind == Addons::Plugins {
        "plugin"
    } else {
        "mod"
    };
    let categories: Vec<String> = loaders.iter().map(|l| format!("categories:{l}")).collect();
    let facets = json!([
        [format!("project_type:{project_type}")],
        categories,
        [format!("versions:{}", cfg.version)]
    ]);
    let query = urlencode(&[
        ("query", target),
        ("facets", &pyjson::to_compact(&facets)?),
        ("limit", "15"),
    ]);
    let data = http::get_json(&format!("https://api.modrinth.com/v2/search?{query}"), &[])?;
    let hits = data
        .get("hits")
        .and_then(Value::as_array)
        .ok_or_else(|| fail("Respuesta inesperada de Modrinth: falta 'hits'."))?;
    if hits.is_empty() {
        ui::info(&format!(
            "Sin resultados para '{target}' en {} {}.",
            cfg.core, cfg.version
        ));
    }
    for hit in hits {
        let slug = hit.get("slug").and_then(Value::as_str).unwrap_or_default();
        let downloads = hit.get("downloads").and_then(Value::as_i64).unwrap_or(0);
        let description: String = hit
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .split(py_is_space)
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(60)
            .collect();
        ui::info(&format!(
            "{slug:<28} {downloads:>8} descargas  {description}"
        ));
    }
    ui::info("Instala con: omh plugin add <nombre de la primera columna>");
    Ok(())
}

fn remove(folder: &Path, target: &str) -> anyhow::Result<()> {
    let wanted = target.to_lowercase();
    let matches: Vec<String> = fs::read_dir(folder)
        .with_context(|| format!("No se pudo leer {}", folder.display()))?
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.to_lowercase().contains(&wanted))
        .collect();
    match matches.as_slice() {
        [] => Err(fail(format!(
            "No hay ningun archivo que coincida con '{target}'."
        ))),
        [only] => {
            let path = folder.join(only);
            fs::remove_file(&path)
                .with_context(|| format!("No se pudo borrar {}", path.display()))?;
            ui::ok(&format!(
                "Eliminado {only}. Reinicia el servidor para aplicarlo."
            ));
            Ok(())
        }
        many => Err(fail(format!(
            "Coinciden varios archivos: {}. Se mas especifico.",
            many.join(", ")
        ))),
    }
}

fn add_from_poggit(folder: &Path, target: &str) -> anyhow::Result<()> {
    let releases = http::get_json(
        &format!(
            "https://poggit.pmmp.io/releases.min.json?name={}",
            py_quote(target, "/")
        ),
        &[],
    )?;
    let list = releases.as_array().map(Vec::as_slice).unwrap_or_default();
    let mut best: Option<&Value> = None;
    for release in list {
        let version = |r: &Value| vt(r.get("version").and_then(Value::as_str).unwrap_or("0"));
        if best.is_none_or(|b| version(release) > version(b)) {
            best = Some(release);
        }
    }
    let Some(release) = best else {
        return Err(fail(format!("No existe el plugin '{target}' en Poggit.")));
    };
    let file = format!("{}.phar", safe_filename(target));
    let url = format!("{}/{file}", http::str_field(release, "artifact_url")?);
    http::download(&url, &folder.join(&file), Checksums::default(), None)
}

pub fn cmd_plugin(args: &PluginArgs) -> anyhow::Result<()> {
    let cfg = config::load_config(&config::resolve_dir(args.dir.as_deref())?)?;
    config::require_step(&cfg, 1)?;
    let (folder, kind) = addons_dir(&cfg)?;
    let action = args.action.as_str();

    if action == "list" {
        let mut files: Vec<String> = fs::read_dir(&folder)
            .with_context(|| format!("No se pudo leer {}", folder.display()))?
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| {
                [".jar", ".phar", ".zip"]
                    .iter()
                    .any(|ext| name.ends_with(ext))
            })
            .collect();
        files.sort();
        let label = if kind == Addons::Plugins {
            "Plugins"
        } else {
            "Mods"
        };
        ui::info(&format!("{label} en {}:", folder.display()));
        for file in &files {
            ui::info(&format!("  {file}"));
        }
        if files.is_empty() {
            ui::info("  (ninguno)");
        }
        return Ok(());
    }

    if args.target.is_empty() {
        return Err(fail(format!(
            "Indica el plugin. Ejemplo: omh plugin {action} nombre"
        )));
    }
    let target = args.target.join(" ");
    match action {
        "search" => return search(&cfg, kind, &target),
        "remove" => return remove(&folder, &target),
        _ => {}
    }

    // add
    if target.starts_with("https://") || target.starts_with("http://") {
        http::download(
            &target,
            &folder.join(safe_filename(&target)),
            Checksums::default(),
            None,
        )?;
    } else if Path::new(&target).is_file() {
        copy2(Path::new(&target), &folder.join(py_basename(&target)))?;
    } else if catalog::modrinth_loaders(&cfg.core).is_some() {
        modrinth_install(&cfg, &folder, &target, &mut HashSet::new())?;
    } else if cfg.core == "pocketmine" {
        add_from_poggit(&folder, &target)?;
    } else {
        return Err(fail(format!(
            "Para {} indica una URL directa o un archivo .jar local.",
            cfg.core
        )));
    }
    ui::ok(&format!(
        "Instalado en {}. Reinicia el servidor para cargarlo.",
        folder.display()
    ));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Resultados de safe_filename en legacy/omh.py.
    #[test]
    fn nombres_seguros_como_python() {
        assert_eq!(
            safe_filename(
                "https://cdn.modrinth.com/data/abc/versions/1.0/WorldEdit%20Bukkit%207.3.jar?x=1"
            ),
            "WorldEdit_Bukkit_7.3.jar"
        );
        assert_eq!(safe_filename("https://x.y/z/"), "addon.jar");
        assert_eq!(safe_filename("plugin name (1).jar"), "plugin_name__1_.jar");
        assert_eq!(safe_filename(".."), "..");
        assert_eq!(safe_filename("caf\u{e9}.jar"), "caf_.jar");
        let expected = if cfg!(windows) { "c.jar" } else { "b_c.jar" };
        assert_eq!(safe_filename(r"a/b\c.jar"), expected);
    }

    #[test]
    fn consulta_de_modrinth_como_python() {
        let loaders = catalog::modrinth_loaders("purpur").unwrap();
        let categories: Vec<String> = loaders.iter().map(|l| format!("categories:{l}")).collect();
        let facets = json!([["project_type:plugin"], categories, ["versions:1.20.4"]]);
        let query = urlencode(&[
            ("query", "world edit"),
            ("facets", &pyjson::to_compact(&facets).unwrap()),
            ("limit", "15"),
        ]);
        assert_eq!(
            query,
            "query=world+edit&facets=%5B%5B%22project_type%3Aplugin%22%5D%2C+%5B%22categories%3Apurpur%22%2C+%22categories%3Apaper%22%2C+%22categories%3Aspigot%22%2C+%22categories%3Abukkit%22%5D%2C+%5B%22versions%3A1.20.4%22%5D%5D&limit=15"
        );
    }

    #[test]
    fn quitar_pide_ser_especifico() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("WorldEdit.jar"), "").unwrap();
        fs::write(tmp.path().join("WorldGuard.jar"), "").unwrap();
        let err = remove(tmp.path(), "world").unwrap_err();
        assert!(err.to_string().starts_with("Coinciden varios archivos: "));
        assert!(remove(tmp.path(), "nada").is_err());
        remove(tmp.path(), "worldedit").unwrap();
        assert!(!tmp.path().join("WorldEdit.jar").exists());
    }
}
