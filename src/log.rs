use crate::style::s;
use crate::writer::{ansi_enabled, writeln as wln};

/// Structured CLI logging with icons and colors.
pub fn info(msg: &str) {
    let icon = if ansi_enabled() { s().blue().paint("ℹ") } else { "i".to_string() };
    wln(&format!("{} {}", icon, msg));
}

pub fn warn(msg: &str) {
    let icon = if ansi_enabled() { s().yellow().paint("⚠") } else { "!".to_string() };
    wln(&format!("{} {}", icon, msg));
}

pub fn error(msg: &str) {
    let icon = if ansi_enabled() { s().red().paint("✗") } else { "x".to_string() };
    wln(&format!("{} {}", icon, msg));
}

pub fn success(msg: &str) {
    let icon = if ansi_enabled() { s().green().paint("✓") } else { "v".to_string() };
    wln(&format!("{} {}", icon, msg));
}

pub fn debug(msg: &str) {
    let icon = if ansi_enabled() { s().dim().paint("●") } else { ".".to_string() };
    wln(&format!("{} {}", icon, msg));
}

pub fn step(msg: &str) {
    let icon = if ansi_enabled() { s().cyan().paint("→") } else { ">".to_string() };
    wln(&format!("{} {}", icon, msg));
}
