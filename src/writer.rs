use std::io::{self, IsTerminal, Write};

/// Whether stdout is a real terminal (not piped)
pub fn is_tty() -> bool {
    io::stdout().is_terminal()
}

/// Whether both stdout AND stdin are TTY (interactive session)
pub fn interactive_tty() -> bool {
    is_tty() && io::stdin().is_terminal()
}

/// Whether ANSI escape codes should be emitted
pub fn ansi_enabled() -> bool {
    if std::env::var("FORCE_COLOR").as_deref() == Ok("1") {
        return true;
    }
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    is_tty()
}

/// Terminal width in columns (default 80)
pub fn term_width() -> u16 {
    crossterm::terminal::size().map(|(w, _)| w).unwrap_or(80)
}

/// Write raw text to stdout (no newline). Panics on write failure (same as println!).
pub fn write(text: &str) {
    let mut stdout = io::stdout().lock();
    stdout.write_all(text.as_bytes()).expect("failed to write to stdout");
    stdout.flush().expect("failed to flush stdout");
}

/// Write text + newline to stdout. Panics on write failure.
pub fn writeln(text: &str) {
    let mut stdout = io::stdout().lock();
    stdout.write_all(text.as_bytes()).expect("failed to write to stdout");
    stdout.write_all(b"\n").expect("failed to write to stdout");
    stdout.flush().expect("failed to flush stdout");
}

/// Write text + newline to stderr. Panics on write failure.
pub fn write_err(text: &str) {
    let mut stderr = io::stderr().lock();
    stderr.write_all(text.as_bytes()).expect("failed to write to stderr");
    stderr.write_all(b"\n").expect("failed to write to stderr");
    stderr.flush().expect("failed to flush stderr");
}

/// Strip ANSI codes if not TTY
pub fn pipe_aware(text: &str) -> String {
    if ansi_enabled() {
        text.to_string()
    } else {
        crate::ansi::strip_ansi(text)
    }
}

/// Calculate visual rows a line occupies (accounting for terminal wrapping)
pub fn visual_rows(line: &str, width: u16) -> u16 {
    let w = crate::ansi::measure_width(line);
    if w == 0 {
        return 1;
    }
    let cols = width as usize;
    ((w + cols - 1) / cols) as u16
}
