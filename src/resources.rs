//! `omh resources` (PASO 2): RAM, almacenamiento maximo y nucleos.

use serde_json::Map;

use crate::cli::ResourcesArgs;
use crate::config::{self, Resources};
use crate::error::fail;
use crate::system::{cpu_count, dir_size_mb, disk_free_mb, total_ram_mb};
use crate::ui;
use crate::util::{fmt_mb, parse_size_mb};

/// RAM sugerida: la mitad redondeada a 512 MB, entre 1 GB y 4 GB (2 GB si no se sabe).
fn suggested_ram(total: i64) -> i64 {
    if total > 0 {
        ((total / 2) / 512 * 512).clamp(1024, 4096)
    } else {
        2048
    }
}

fn given(value: &Option<String>) -> Option<&str> {
    value.as_deref().filter(|value| !value.is_empty())
}

pub fn cmd_resources(args: &ResourcesArgs) -> anyhow::Result<()> {
    let mut cfg = config::load_config(&config::resolve_dir(args.dir.as_deref())?)?;
    config::require_step(&cfg, 1)?;
    ui::header(&format!("PASO 2 de 3 - Recursos ({})", cfg.name));
    let total = total_ram_mb();
    let cpus = cpu_count();
    let used = dir_size_mb(&cfg.dir);
    let free = disk_free_mb(&cfg.dir)?;
    let ram_text = if total > 0 {
        fmt_mb(total)
    } else {
        "desconocida".to_owned()
    };
    ui::info(&format!(
        "Este equipo: RAM {ram_text} | {cpus} nucleos | disco libre {} | servidor ocupa {}",
        fmt_mb(free),
        fmt_mb(used)
    ));

    let default_ram = fmt_mb(suggested_ram(total))
        .replace(' ', "")
        .replace(".0", "");
    let ram = match given(&args.ram) {
        Some(ram) => ram.to_owned(),
        None => ui::ask("RAM para el servidor (ej. 4G)", Some(&default_ram), &[])?,
    };
    let ram = parse_size_mb(&ram)?;
    if ram < 512 {
        return Err(fail("La RAM minima es 512M."));
    }
    if total > 0 && ram > total - 512 {
        return Err(fail(format!(
            "No puedes asignar {}: el equipo tiene {} y necesita al menos 512 MB para el sistema.",
            fmt_mb(ram),
            fmt_mb(total)
        )));
    }
    if total > 0 && ram as f64 > total as f64 * 0.8 {
        ui::warn("Asignas mas del 80% de la RAM del equipo. El sistema puede ir lento.");
    }

    let storage = match given(&args.storage) {
        Some(storage) => storage.to_owned(),
        None => ui::ask("Almacenamiento maximo (ej. 20G)", Some("10G"), &[])?,
    };
    let storage = parse_size_mb(&storage)?;
    if storage < used + 256 {
        return Err(fail(format!(
            "El limite de almacenamiento debe ser mayor que lo que ya ocupa el servidor ({}).",
            fmt_mb(used)
        )));
    }
    if storage - used > free {
        return Err(fail(format!(
            "No hay espacio suficiente en disco: pides {} y hay {} libres.",
            fmt_mb(storage - used),
            fmt_mb(free)
        )));
    }

    // B2: un 0 explicito cuenta como valor (Python lo trataba como "no indicado").
    let cores = match args.cores {
        Some(cores) => cores,
        None => {
            let answer = ui::ask(
                &format!("Nucleos de CPU (1-{cpus})"),
                Some(&cpus.min(2).to_string()),
                &[],
            )?;
            answer
                .trim()
                .parse::<i64>()
                .map_err(|_| fail(format!("Numero de nucleos no valido: '{answer}'.")))?
        }
    };
    if !(1..=cpus).contains(&cores) {
        return Err(fail(format!("Los nucleos deben estar entre 1 y {cpus}.")));
    }

    cfg.resources = Some(Resources {
        ram_mb: ram,
        storage_mb: storage,
        cores,
        extra: Map::new(),
    });
    cfg.step = cfg.step.max(2);
    if cfg.is_bedrock_vanilla() {
        config::set_properties(
            &cfg.dir.join("server.properties"),
            &[("max-threads", cores.to_string())],
        )?;
        ui::info("Bedrock vanilla no permite limitar la RAM. Se limitan los nucleos y el disco.");
    } else if cfg.core == "pocketmine" {
        ui::info(
            "PocketMine (PHP) no permite fijar la RAM desde fuera. Se limitan los nucleos y el disco.",
        );
    }
    config::save_config(&cfg)?;
    ui::ok(&format!(
        "Recursos: RAM {} | almacenamiento {} | {cores} nucleos",
        fmt_mb(ram),
        fmt_mb(storage)
    ));
    print!(
        "\n  Siguiente paso:\n    PASO 3 de 3 - Red:  omh network --mode lan --port auto --whitelist on --admin TuNombre\n    Modos: local (solo este PC), lan (tu red de casa), public (internet)\n\n"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ram_sugerida_como_python() {
        assert_eq!(suggested_ram(0), 2048);
        assert_eq!(suggested_ram(1500), 1024);
        assert_eq!(suggested_ram(6000), 2560);
        assert_eq!(suggested_ram(15847), 4096);
        let text = |mb| fmt_mb(suggested_ram(mb)).replace(' ', "").replace(".0", "");
        assert_eq!(text(6000), "2.5GB");
        assert_eq!(text(15847), "4GB");
    }
}
