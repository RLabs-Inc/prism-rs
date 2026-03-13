use crate::writer::ansi_enabled;

pub const RESET: &str = "\x1b[0m";

#[derive(Debug, Clone)]
pub enum Color {
    Rgb(u8, u8, u8),
    Hex(u32),
}

pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

pub fn hex(value: u32) -> Color {
    Color::Hex(value)
}

impl Color {
    fn to_rgb(&self) -> (u8, u8, u8) {
        match *self {
            Color::Rgb(r, g, b) => (r, g, b),
            Color::Hex(v) => (
                ((v >> 16) & 0xFF) as u8,
                ((v >> 8) & 0xFF) as u8,
                (v & 0xFF) as u8,
            ),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Style {
    open: String,
    close: String,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    fn push(mut self, open: &str, close: &str) -> Self {
        self.open.push_str(open);
        // Prepend close code so they nest correctly (LIFO)
        self.close = format!("{}{}", close, self.close);
        self
    }

    // Modifiers
    pub fn bold(self) -> Self { self.push("\x1b[1m", "\x1b[22m") }
    pub fn dim(self) -> Self { self.push("\x1b[2m", "\x1b[22m") }
    pub fn italic(self) -> Self { self.push("\x1b[3m", "\x1b[23m") }
    pub fn underline(self) -> Self { self.push("\x1b[4m", "\x1b[24m") }
    pub fn inverse(self) -> Self { self.push("\x1b[7m", "\x1b[27m") }
    pub fn strikethrough(self) -> Self { self.push("\x1b[9m", "\x1b[29m") }

    // Foreground ANSI 16 (terminal-themed)
    pub fn black(self) -> Self { self.push("\x1b[30m", "\x1b[39m") }
    pub fn red(self) -> Self { self.push("\x1b[31m", "\x1b[39m") }
    pub fn green(self) -> Self { self.push("\x1b[32m", "\x1b[39m") }
    pub fn yellow(self) -> Self { self.push("\x1b[33m", "\x1b[39m") }
    pub fn blue(self) -> Self { self.push("\x1b[34m", "\x1b[39m") }
    pub fn magenta(self) -> Self { self.push("\x1b[35m", "\x1b[39m") }
    pub fn cyan(self) -> Self { self.push("\x1b[36m", "\x1b[39m") }
    pub fn white(self) -> Self { self.push("\x1b[37m", "\x1b[39m") }
    pub fn gray(self) -> Self { self.push("\x1b[90m", "\x1b[39m") }

    // Bright foreground ANSI 16
    pub fn bright_red(self) -> Self { self.push("\x1b[91m", "\x1b[39m") }
    pub fn bright_green(self) -> Self { self.push("\x1b[92m", "\x1b[39m") }
    pub fn bright_yellow(self) -> Self { self.push("\x1b[93m", "\x1b[39m") }
    pub fn bright_blue(self) -> Self { self.push("\x1b[94m", "\x1b[39m") }
    pub fn bright_magenta(self) -> Self { self.push("\x1b[95m", "\x1b[39m") }
    pub fn bright_cyan(self) -> Self { self.push("\x1b[96m", "\x1b[39m") }
    pub fn bright_white(self) -> Self { self.push("\x1b[97m", "\x1b[39m") }

    // Background ANSI 16
    pub fn bg_black(self) -> Self { self.push("\x1b[40m", "\x1b[49m") }
    pub fn bg_red(self) -> Self { self.push("\x1b[41m", "\x1b[49m") }
    pub fn bg_green(self) -> Self { self.push("\x1b[42m", "\x1b[49m") }
    pub fn bg_yellow(self) -> Self { self.push("\x1b[43m", "\x1b[49m") }
    pub fn bg_blue(self) -> Self { self.push("\x1b[44m", "\x1b[49m") }
    pub fn bg_magenta(self) -> Self { self.push("\x1b[45m", "\x1b[49m") }
    pub fn bg_cyan(self) -> Self { self.push("\x1b[46m", "\x1b[49m") }
    pub fn bg_white(self) -> Self { self.push("\x1b[47m", "\x1b[49m") }

    // Exact colors via RGB/Hex
    pub fn fg(self, color: Color) -> Self {
        let (r, g, b) = color.to_rgb();
        self.push(&format!("\x1b[38;2;{};{};{}m", r, g, b), "\x1b[39m")
    }

    pub fn bg_color(self, color: Color) -> Self {
        let (r, g, b) = color.to_rgb();
        self.push(&format!("\x1b[48;2;{};{};{}m", r, g, b), "\x1b[49m")
    }

    /// Terminal method: wrap text in accumulated ANSI codes (respects ansi_enabled())
    pub fn paint(&self, text: &str) -> String {
        if !ansi_enabled() {
            return crate::ansi::strip_ansi(text);
        }
        self.render(text)
    }

    /// Always apply ANSI codes regardless of TTY state (for testing, logging to files, etc.)
    pub fn render(&self, text: &str) -> String {
        if self.open.is_empty() {
            return text.to_string();
        }
        format!("{}{}{}", self.open, text, self.close)
    }
}

/// Start a style chain
pub fn s() -> Style {
    Style::new()
}

/// Alias for s()
pub fn style() -> Style {
    Style::new()
}

/// Convenience: apply exact fg (and optional bg) color
pub fn color(text: &str, fg: Color, bg: Option<Color>) -> String {
    if !ansi_enabled() {
        return crate::ansi::strip_ansi(text);
    }
    let mut result = String::new();
    let (r, g, b) = fg.to_rgb();
    result.push_str(&format!("\x1b[38;2;{};{};{}m", r, g, b));
    if let Some(ref bg_color) = bg {
        let (r, g, b) = bg_color.to_rgb();
        result.push_str(&format!("\x1b[48;2;{};{};{}m", r, g, b));
    }
    result.push_str(text);
    if bg.is_some() {
        result.push_str("\x1b[49m");
    }
    result.push_str("\x1b[39m");
    result
}
