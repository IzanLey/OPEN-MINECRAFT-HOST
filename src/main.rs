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

mod catalog;
mod cli;
mod config;
mod cores;
mod create;
mod error;
mod extract;
mod http;
mod java;
mod network;
mod platform;
mod players;
mod plugins;
mod pyjson;
mod resources;
mod runner;
mod servers;
mod system;
mod ui;
mod update;
mod util;

use std::process::ExitCode;

use cli::Command;

pub const APP_NAME: &str = "open-mc-host";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GITHUB_REPO: &str = "IzanLey/OPEN-MINECRAFT-HOST";

fn main() -> ExitCode {
    ui::install_ctrlc_handler();
    update::cleanup_old_binary();
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
    use runner::commands;
    match command {
        Command::Create(args) => create::cmd_create(&args),
        Command::Resources(args) => resources::cmd_resources(&args),
        Command::Network(args) => network::cmd_network(&args),
        Command::Start(args) => commands::cmd_start(&args),
        Command::Stop(args) => commands::cmd_stop(&args),
        Command::Restart(args) => commands::cmd_restart(&args),
        Command::Status(args) => commands::cmd_status(&args),
        Command::Console(args) => commands::cmd_console(&args),
        Command::Logs(args) => commands::cmd_logs(&args),
        Command::Cmd(args) => commands::cmd_cmd(&args),
        Command::Whitelist(args) => players::cmd_whitelist(&args),
        Command::Admin(args) => players::cmd_admin(&args),
        Command::Plugin(args) => plugins::cmd_plugin(&args),
        Command::Eula(args) => create::cmd_eula(&args),
        Command::Tunnel(_) => network::tunnel::cmd_tunnel(),
        Command::List => catalog::cmd_list(),
        Command::Servers => servers::cmd_servers(),
        Command::Use(args) => servers::cmd_use(&args.target),
        Command::Update => update::cmd_update(),
    }
}
