//! OPEN MINECRAFT HOST (omh)
//!
//! Herramienta de terminal para convertir tu PC en un servidor de Minecraft.
//! Funciona en Windows y Linux. Es la version en Rust de `legacy/omh.py` y se
//! comporta igual que ella: mismos comandos, textos y archivos.
//!
//! Flujo basico (en este orden):
//!
//! ```text
//! 1. omh create     -> edicion, core y version
//! 2. omh resources  -> RAM, almacenamiento y nucleos
//! 3. omh network    -> red, puerto, whitelist y administradores
//! Despues: omh start
//! ```

// Durante la migracion por fases hay piezas ya portadas que todavia no usa ningun
// comando. Se quita en la fase 5, cuando estan conectados todos.
#![allow(dead_code)]

mod cli;
mod config;
mod error;
mod platform;
mod pyjson;
mod runner;
mod servers;
mod ui;
mod util;

use std::process::ExitCode;

use cli::Command;

pub const APP_NAME: &str = "open-mc-host";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GITHUB_REPO: &str = "IzanLey/OPEN-MINECRAFT-HOST";

fn main() -> ExitCode {
    ui::install_ctrlc_handler();
    let Some(command) = cli::parse().command else {
        ui::print_title();
        ui::print_guide();
        return ExitCode::SUCCESS;
    };
    match run(command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            ui::print_error(&err);
            ExitCode::FAILURE
        }
    }
}

fn run(command: Command) -> anyhow::Result<()> {
    match command {
        Command::Servers => servers::cmd_servers(),
        Command::Use(args) => servers::cmd_use(&args.target),
        _ => Err(error::fail("Este comando aun no esta portado a Rust.")),
    }
}
