//! Linea de comandos: los mismos subcomandos, opciones y textos de ayuda que
//! `legacy/omh.py`. La ayuda la genera clap, con las cabeceras en espanol.

use std::process;

use clap::error::ErrorKind;
use clap::{Arg, ArgAction, Args, ColorChoice, CommandFactory, FromArgMatches, Parser, Subcommand};

const DIR_HELP: &str = "carpeta del servidor (por defecto el servidor activo)";

#[derive(Parser, Debug)]
#[command(
    name = crate::APP_NAME,
    bin_name = "omh",
    version = crate::APP_VERSION,
    about = "OPEN MINECRAFT HOST - convierte tu PC en un servidor de Minecraft.",
    after_help = "Orden: 1) omh create  2) omh resources  3) omh network  4) omh start",
    subcommand_value_name = "COMANDO",
    subcommand_help_heading = "Comandos"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(about = "PASO 1: elegir edicion, core y version e instalar el servidor")]
    Create(CreateArgs),
    #[command(about = "PASO 2: asignar RAM, almacenamiento y nucleos")]
    Resources(ResourcesArgs),
    #[command(about = "PASO 3: configurar red, puerto, whitelist y administradores")]
    Network(NetworkArgs),
    #[command(about = "iniciar el servidor")]
    Start(StartArgs),
    #[command(about = "detener el servidor guardando el mundo")]
    Stop(DirArgs),
    #[command(about = "reiniciar el servidor en segundo plano")]
    Restart(DirArgs),
    #[command(about = "ver la configuracion y el estado del servidor")]
    Status(DirArgs),
    #[command(about = "abrir la consola de un servidor en segundo plano")]
    Console(DirArgs),
    #[command(about = "ver las ultimas lineas del registro")]
    Logs(LogsArgs),
    #[command(about = "enviar un comando a la consola del servidor")]
    Cmd(CmdArgs),
    #[command(about = "gestionar la whitelist")]
    Whitelist(WhitelistArgs),
    #[command(about = "gestionar los administradores (op)")]
    Admin(AdminArgs),
    #[command(about = "buscar, instalar, listar o quitar plugins y mods")]
    Plugin(PluginArgs),
    #[command(about = "aceptar el EULA de Minecraft (servidores Java)")]
    Eula(EulaArgs),
    #[command(about = "configurar o ver el tunel gratuito de playit.gg")]
    Tunnel(DirArgs),
    #[command(about = "ver ediciones, cores y versiones disponibles")]
    List,
    #[command(about = "ver todos los servidores creados")]
    Servers,
    #[command(about = "elegir el servidor activo")]
    Use(UseArgs),
    #[command(about = "actualizar open-mc-host desde GitHub")]
    Update,
}

#[derive(Args, Debug)]
pub struct DirArgs {
    #[arg(long, help = DIR_HELP)]
    pub dir: Option<String>,
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    #[arg(long, short = 'e', value_parser = ["java", "bedrock"], value_name = "java|bedrock",
          hide_possible_values = true, help = "edicion de Minecraft")]
    pub edition: Option<String>,
    #[arg(
        long,
        short = 'c',
        help = "java: vanilla, paper, purpur, fabric, forge, neoforge | bedrock: vanilla, pocketmine, nukkit"
    )]
    pub core: Option<String>,
    #[arg(long, short = 'v', help = "version de Minecraft o 'latest'")]
    pub version: Option<String>,
    #[arg(long, short = 'd', help = "carpeta donde se instala el servidor")]
    pub dir: Option<String>,
    #[arg(
        long,
        short = 'n',
        help = "nombre del servidor (por defecto el de la carpeta)"
    )]
    pub name: Option<String>,
    #[arg(
        long,
        help = "aceptar el EULA de Minecraft (https://aka.ms/MinecraftEULA)"
    )]
    pub eula: bool,
    #[arg(long, help = "reinstalar si ya existe un servidor en la carpeta")]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct ResourcesArgs {
    #[arg(long, help = DIR_HELP)]
    pub dir: Option<String>,
    #[arg(long, short = 'r', help = "RAM, por ejemplo 4G o 4096M")]
    pub ram: Option<String>,
    #[arg(long, short = 's', help = "almacenamiento maximo, por ejemplo 20G")]
    pub storage: Option<String>,
    #[arg(
        long,
        short = 'c',
        allow_negative_numbers = true,
        help = "numero de nucleos de CPU"
    )]
    pub cores: Option<i64>,
}

#[derive(Args, Debug)]
pub struct NetworkArgs {
    #[arg(long, help = DIR_HELP)]
    pub dir: Option<String>,
    #[arg(long, short = 'm', value_parser = ["local", "lan", "public"], value_name = "local|lan|public",
          hide_possible_values = true, help = "local: solo este PC | lan: tu red | public: internet")]
    pub mode: Option<String>,
    #[arg(long, value_parser = ["playit", "upnp", "manual"], value_name = "playit|upnp|manual",
          hide_possible_values = true,
          help = "como abrirlo a internet (modo public). playit = direccion gratis sin tocar el router")]
    pub public_method: Option<String>,
    #[arg(
        long,
        short = 'p',
        allow_negative_numbers = true,
        help = "puerto o 'auto' para buscar uno libre"
    )]
    pub port: Option<String>,
    #[arg(
        long,
        help = "IP local donde escuchar (por defecto todas; en modo local 127.0.0.1)"
    )]
    pub ip: Option<String>,
    #[arg(long, short = 'w', value_parser = ["on", "off"], value_name = "on|off",
          hide_possible_values = true, help = "activar o desactivar la whitelist")]
    pub whitelist: Option<String>,
    #[arg(long, short = 'a', action = ArgAction::Append, help = "jugador administrador (se puede repetir)")]
    pub admin: Vec<String>,
    #[arg(long, value_parser = ["on", "off"], value_name = "on|off", hide_possible_values = true,
          help = "comprobar cuentas oficiales (on recomendado)")]
    pub online_mode: Option<String>,
    #[arg(
        long,
        allow_negative_numbers = true,
        help = "numero maximo de jugadores"
    )]
    pub max_players: Option<i64>,
    #[arg(long, help = "nombre o mensaje del servidor")]
    pub motd: Option<String>,
}

#[derive(Args, Debug)]
pub struct StartArgs {
    #[arg(long, help = DIR_HELP)]
    pub dir: Option<String>,
    #[arg(long, help = "iniciar en segundo plano")]
    pub detach: bool,
    /// Proceso en segundo plano lanzado por `omh start --detach` (oculta).
    #[arg(long, hide = true)]
    pub daemon: bool,
}

#[derive(Args, Debug)]
pub struct LogsArgs {
    #[arg(long, help = DIR_HELP)]
    pub dir: Option<String>,
    #[arg(
        long,
        short = 'l',
        default_value_t = 50,
        hide_default_value = true,
        allow_negative_numbers = true,
        help = "numero de lineas"
    )]
    pub lines: i64,
}

#[derive(Args, Debug)]
pub struct CmdArgs {
    #[arg(long, help = DIR_HELP)]
    pub dir: Option<String>,
    #[arg(required = true, num_args = 1.., allow_negative_numbers = true,
          help = "comando, por ejemplo: say hola")]
    pub text: Vec<String>,
}

#[derive(Args, Debug)]
pub struct WhitelistArgs {
    #[arg(long, help = DIR_HELP)]
    pub dir: Option<String>,
    #[arg(value_parser = ["add", "remove", "list", "on", "off"], value_name = "add|remove|list|on|off",
          hide_possible_values = true)]
    pub action: String,
    #[arg(help = "nombre del jugador")]
    pub player: Vec<String>,
}

#[derive(Args, Debug)]
pub struct AdminArgs {
    #[arg(long, help = DIR_HELP)]
    pub dir: Option<String>,
    #[arg(value_parser = ["add", "remove", "list"], value_name = "add|remove|list", hide_possible_values = true)]
    pub action: String,
    #[arg(help = "nombre del jugador")]
    pub player: Vec<String>,
}

#[derive(Args, Debug)]
pub struct PluginArgs {
    #[arg(long, help = DIR_HELP)]
    pub dir: Option<String>,
    #[arg(value_parser = ["search", "add", "remove", "list"], value_name = "search|add|remove|list",
          hide_possible_values = true)]
    pub action: String,
    #[arg(help = "nombre en Modrinth/Poggit, URL o archivo local")]
    pub target: Vec<String>,
}

#[derive(Args, Debug)]
pub struct EulaArgs {
    #[arg(long, help = DIR_HELP)]
    pub dir: Option<String>,
    #[arg(long, help = "aceptar el EULA")]
    pub accept: bool,
}

#[derive(Args, Debug)]
pub struct UseArgs {
    #[arg(help = "nombre o carpeta del servidor")]
    pub target: String,
}

/// El analizador completo, con la ayuda en espanol.
pub fn command() -> clap::Command {
    localize(Cli::command(), true)
}

/// Lee los argumentos. Ante un error de uso, clap muestra el error y sale con codigo 2,
/// como argparse.
pub fn parse() -> Cli {
    let matches = match command().try_get_matches() {
        Ok(matches) => matches,
        Err(err) if err.kind() == ErrorKind::DisplayHelp => {
            print!(
                "{}",
                err.render().to_string().replace("[OPTIONS]", "[OPCIONES]")
            );
            process::exit(0);
        }
        Err(err) => err.exit(),
    };
    Cli::from_arg_matches(&matches).unwrap_or_else(|err| err.exit())
}

fn localize(cmd: clap::Command, root: bool) -> clap::Command {
    let mut cmd = cmd
        .help_template("{about-with-newline}\nUso: {usage}\n\n{all-args}{after-help}")
        .color(ColorChoice::Never)
        // argparse acepta abreviaturas de las opciones largas (--ed = --edition).
        .infer_long_args(true)
        .disable_help_subcommand(true)
        .disable_help_flag(true)
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .action(ArgAction::Help)
                .help("muestra esta ayuda y sale"),
        );
    if root {
        cmd = cmd.disable_version_flag(true).arg(
            Arg::new("version")
                .long("version")
                .action(ArgAction::Version)
                .help("muestra la version y sale"),
        );
    }
    cmd.mut_args(|arg| {
        if arg.get_help_heading().is_some() {
            return arg;
        }
        let heading = if arg.is_positional() {
            "Argumentos"
        } else {
            "Opciones"
        };
        arg.help_heading(heading)
    })
    .mut_subcommands(|sub| localize(sub, false))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_from(args: &[&str]) -> Result<Cli, clap::Error> {
        let matches = command().try_get_matches_from(args)?;
        Cli::from_arg_matches(&matches)
    }

    #[test]
    fn definicion_valida() {
        command().debug_assert();
    }

    #[test]
    fn create_con_opciones_cortas() {
        let cli = parse_from(&[
            "omh", "create", "-e", "java", "-c", "paper", "-v", "1.21.1", "-d", "x", "--eula",
        ])
        .unwrap();
        let Some(Command::Create(args)) = cli.command else {
            panic!("se esperaba create");
        };
        assert_eq!(args.edition.as_deref(), Some("java"));
        assert_eq!(args.core.as_deref(), Some("paper"));
        assert_eq!(args.version.as_deref(), Some("1.21.1"));
        assert_eq!(args.dir.as_deref(), Some("x"));
        assert!(args.eula && !args.force);
    }

    #[test]
    fn abreviaturas_como_argparse() {
        let cli = parse_from(&["omh", "create", "--ed", "bedrock"]).unwrap();
        let Some(Command::Create(args)) = cli.command else {
            panic!("se esperaba create");
        };
        assert_eq!(args.edition.as_deref(), Some("bedrock"));
        // --e es ambigua (--edition, --eula): error de uso.
        assert!(parse_from(&["omh", "create", "--e", "java"]).is_err());
    }

    #[test]
    fn valor_no_permitido_sale_con_codigo_2() {
        let err = parse_from(&["omh", "create", "-e", "pc"]).unwrap_err();
        assert_eq!(err.exit_code(), 2);
        assert!(
            parse_from(&["omh", "help"]).is_err(),
            "no hay subcomando help"
        );
    }

    #[test]
    fn admin_repetible_y_numeros_negativos() {
        let cli = parse_from(&[
            "omh",
            "network",
            "-a",
            "Steve",
            "--admin",
            "Alex",
            "--max-players",
            "-1",
        ])
        .unwrap();
        let Some(Command::Network(args)) = cli.command else {
            panic!("se esperaba network");
        };
        assert_eq!(args.admin, ["Steve", "Alex"]);
        assert_eq!(args.max_players, Some(-1));

        let cli = parse_from(&["omh", "logs", "-l", "-5"]).unwrap();
        let Some(Command::Logs(args)) = cli.command else {
            panic!("se esperaba logs");
        };
        assert_eq!(args.lines, -5);
    }

    #[test]
    fn cmd_necesita_texto_y_admite_opciones_despues() {
        assert!(parse_from(&["omh", "cmd"]).is_err());
        let cli = parse_from(&["omh", "cmd", "say", "hola", "--dir", "x"]).unwrap();
        let Some(Command::Cmd(args)) = cli.command else {
            panic!("se esperaba cmd");
        };
        assert_eq!(args.text, ["say", "hola"]);
        assert_eq!(args.dir.as_deref(), Some("x"));
    }

    #[test]
    fn version_como_python() {
        let err = parse_from(&["omh", "--version"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::DisplayVersion);
        assert_eq!(
            err.render().to_string(),
            format!("open-mc-host {}\n", crate::APP_VERSION)
        );
    }

    #[test]
    fn ayuda_en_espanol() {
        let err = parse_from(&["omh", "create", "--help"]).unwrap_err();
        let help = err.render().to_string();
        assert!(
            help.starts_with("PASO 1: elegir edicion, core y version e instalar el servidor\n")
        );
        assert!(help.contains("Uso: omh create"));
        assert!(help.contains("Opciones:"));
        assert!(help.contains("muestra esta ayuda y sale"));
        assert!(!help.contains("Options:"));
    }
}
