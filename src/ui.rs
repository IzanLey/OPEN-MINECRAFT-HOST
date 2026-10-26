//! Lo que ve el usuario: titulo ASCII, guia, mensajes, preguntas y Ctrl+C.

use std::io::{self, BufRead, IsTerminal, Write};
use std::process;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

use crate::error::{fail, user_message};
use crate::util::py_strip;

// ---------------------------------------------------------------------------
// Titulo
// ---------------------------------------------------------------------------

/// Letras del titulo, en el estilo "standard" de figlet.
fn glyph(letter: char) -> [&'static str; 5] {
    match letter {
        'O' => [r"  ___", r" / _ \", r"| | | |", r"| |_| |", r" \___/"],
        'P' => [r" ____", r"|  _ \", r"| |_) |", r"|  __/", r"|_|"],
        'E' => [r" _____", r"| ____|", r"|  _|", r"| |___", r"|_____|"],
        'N' => [r" _   _", r"| \ | |", r"|  \| |", r"| |\  |", r"|_| \_|"],
        'M' => [
            r" __  __",
            r"|  \/  |",
            r"| |\/| |",
            r"| |  | |",
            r"|_|  |_|",
        ],
        'I' => [r" ___", r"|_ _|", r" | |", r" | |", r"|___|"],
        'C' => [r"  ____", r" / ___|", r"| |", r"| |___", r" \____|"],
        'R' => [r" ____", r"|  _ \", r"| |_) |", r"|  _ <", r"|_| \_\"],
        'A' => [r"    _", r"   / \", r"  / _ \", r" / ___ \", r"/_/   \_\"],
        'F' => [r" _____", r"|  ___|", r"| |_", r"|  _|", r"|_|"],
        'T' => [r" _____", r"|_   _|", r"  | |", r"  | |", r"  |_|"],
        'H' => [r" _   _", r"| | | |", r"| |_| |", r"|  _  |", r"|_| |_|"],
        'S' => [r" ____", r"/ ___|", r"\___ \", r" ___) |", r"|____/"],
        _ => [""; 5],
    }
}

fn render_word(word: &str) -> Vec<String> {
    let mut rows = vec![String::new(); 5];
    for letter in word.chars() {
        let lines = glyph(letter);
        let width = lines.iter().map(|line| line.len()).max().unwrap_or(0);
        if width == 0 {
            continue;
        }
        let mut glyph: Vec<String> = lines.iter().map(|line| format!("{line:<width$}")).collect();
        // Junta las letras una columna cuando no se pisan, como hace figlet.
        let overlap = !rows[0].is_empty()
            && rows
                .iter()
                .zip(&glyph)
                .all(|(row, g)| row.ends_with(' ') || g.starts_with(' '));
        if overlap {
            for (row, g) in rows.iter_mut().zip(glyph.iter_mut()) {
                let first = g.remove(0);
                if row.ends_with(' ') {
                    row.pop();
                    row.push(first);
                }
            }
        }
        for (row, g) in rows.iter_mut().zip(&glyph) {
            row.push_str(g);
        }
    }
    rows
}

pub fn print_title() {
    let blocks: Vec<Vec<String>> = ["OPEN", "MINECRAFT", "HOST"]
        .iter()
        .map(|word| render_word(word))
        .collect();
    let width = blocks.iter().flatten().map(String::len).max().unwrap_or(0);
    let mut out = String::from("\n");
    for block in &blocks {
        let pad = width.saturating_sub(block.first().map_or(0, String::len)) / 2;
        for row in block {
            out.push_str(&format!("  {}{}\n", " ".repeat(pad), row.trim_end()));
        }
        out.push('\n');
    }
    out.push_str(&format!(
        "  Convierte tu PC en un servidor de Minecraft  -  v{}\n",
        crate::APP_VERSION
    ));
    out.push_str(&format!("  {}\n", "-".repeat(width)));
    print!("{out}");
}

const GUIDE: &str = "
  PASO 1 de 3 - Crear el servidor (edicion, core y version):

    omh create --edition java --core paper --version 1.21.1 --dir ./mi-servidor

  Si ejecutas \"omh create\" sin opciones se abre un asistente que te pregunta
  cada valor. Para ver los cores y versiones disponibles: omh list

  Despues:
    PASO 2:  omh resources --ram 4G --storage 20G --cores 2
    PASO 3:  omh network --mode lan --port auto --whitelist on --admin TuNombre
    INICIAR: omh start

  Ayuda completa: omh --help  |  Ayuda de un comando: omh create --help
";

pub fn print_guide() {
    println!("{GUIDE}");
}

// ---------------------------------------------------------------------------
// Mensajes
// ---------------------------------------------------------------------------

pub fn info(msg: &str) {
    println!("  {msg}");
}

pub fn ok(msg: &str) {
    println!("  [OK] {msg}");
}

pub fn warn(msg: &str) {
    println!("  [AVISO] {msg}");
}

pub fn header(msg: &str) {
    println!();
    println!("  == {msg} ==");
}

pub fn print_error(err: &anyhow::Error) {
    println!("  [ERROR] {}", user_message(err));
}

// ---------------------------------------------------------------------------
// Preguntas
// ---------------------------------------------------------------------------

/// Hay alguien escribiendo en una terminal (no una tuberia ni un archivo).
pub fn interactive() -> bool {
    io::stdin().is_terminal()
}

/// Muestra la pregunta y lee una linea. `None` si se acaba la entrada.
fn read_answer(prompt: &str) -> Option<String> {
    let mut out = io::stdout();
    let _ = out.write_all(prompt.as_bytes());
    let _ = out.flush();
    let mut line = String::new();
    match io::stdin().lock().read_line(&mut line) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(line.trim_end_matches(['\n', '\r']).to_owned()),
    }
}

/// Pregunta un valor al usuario. Sin terminal interactiva usa el valor por defecto.
/// Con `choices`, la respuesta se devuelve en minusculas.
pub fn ask(question: &str, default: Option<&str>, choices: &[&str]) -> anyhow::Result<String> {
    let fallback = || {
        default
            .map(str::to_owned)
            .ok_or_else(|| fail(format!("Falta un valor obligatorio: {question}")))
    };
    if !interactive() {
        return fallback();
    }
    let mut hint = String::new();
    if !choices.is_empty() {
        hint.push_str(&format!(" [{}]", choices.join("/")));
    }
    if let Some(value) = default.filter(|value| !value.is_empty()) {
        hint.push_str(&format!(" (por defecto: {value})"));
    }
    loop {
        let Some(line) = read_answer(&format!("  {question}{hint}: ")) else {
            return fallback();
        };
        let answer = py_strip(&line);
        if answer.is_empty() {
            match default {
                Some(value) => return Ok(value.to_owned()),
                None => continue,
            }
        }
        if choices.is_empty() {
            return Ok(answer.to_owned());
        }
        let lower = answer.to_lowercase();
        if choices.contains(&lower.as_str()) {
            return Ok(lower);
        }
        info(&format!("Opciones validas: {}", choices.join(", ")));
    }
}

/// Pregunta de si o no. Sin terminal interactiva devuelve el valor por defecto.
pub fn confirm(question: &str, default: bool) -> bool {
    if !interactive() {
        return default;
    }
    let suffix = if default { " [S/n]: " } else { " [s/N]: " };
    let Some(line) = read_answer(&format!("  {question}{suffix}")) else {
        println!();
        return default;
    };
    let answer = py_strip(&line).to_lowercase();
    if answer.is_empty() {
        return default;
    }
    matches!(answer.as_str(), "s" | "si" | "y" | "yes")
}

// ---------------------------------------------------------------------------
// Ctrl+C
// ---------------------------------------------------------------------------

/// Que hace Ctrl+C en cada momento.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CtrlC {
    /// `Cancelado.` y codigo 130 (lo normal en cualquier comando).
    Cancel,
    /// Solo se anota: el codigo que espera decide que hacer (supervisor, tunel).
    Notify,
    /// Salto de linea y codigo 0 (`omh console`).
    QuietExit,
}

static CTRLC_MODE: AtomicU8 = AtomicU8::new(CtrlC::Cancel as u8);
static CTRLC_PRESSES: AtomicUsize = AtomicUsize::new(0);

pub fn install_ctrlc_handler() {
    // Si el sistema no deja instalarlo, Ctrl+C cierra el programa sin mensaje.
    let _ = ctrlc::set_handler(|| {
        let mode = CTRLC_MODE.load(Ordering::SeqCst);
        if mode == CtrlC::Notify as u8 {
            CTRLC_PRESSES.fetch_add(1, Ordering::SeqCst);
        } else if mode == CtrlC::QuietExit as u8 {
            exit_now("\n", 0);
        } else {
            exit_now("\n  Cancelado.\n", 130);
        }
    });
}

pub fn set_ctrlc(mode: CtrlC) {
    CTRLC_MODE.store(mode as u8, Ordering::SeqCst);
}

/// Si se ha pulsado Ctrl+C desde la ultima vez que se pregunto.
pub fn take_ctrlc() -> bool {
    CTRLC_PRESSES.swap(0, Ordering::SeqCst) > 0
}

fn exit_now(text: &str, code: i32) -> ! {
    let mut out = io::stdout();
    let _ = out.write_all(text.as_bytes());
    let _ = out.flush();
    process::exit(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palabras_del_titulo() {
        let rows = render_word("OPEN");
        let trimmed: Vec<&str> = rows.iter().map(|row| row.trim_end()).collect();
        assert_eq!(
            trimmed,
            [
                r"  ___   ____   _____  _   _",
                r" / _ \ |  _ \ | ____|| \ | |",
                r"| | | || |_) ||  _|  |  \| |",
                r"| |_| ||  __/ | |___ | |\  |",
                r" \___/ |_|    |_____||_| \_|",
            ]
        );
    }
}
