use crate::style::s;
use crate::writer::ansi_enabled;

pub enum BadgeVariant {
    Bracket,
    Dot,
    Pill,
}

/// Render an inline status badge.
/// color_fn: optional styling function for the badge text (default: white)
pub fn badge(text: &str, variant: BadgeVariant, color_fn: Option<fn(&str) -> String>) -> String {
    let color = color_fn.unwrap_or(|t| s().white().paint(t));

    if !ansi_enabled() {
        return match variant {
            BadgeVariant::Dot => format!("* {}", text),
            _ => format!("[{}]", text),
        };
    }

    match variant {
        BadgeVariant::Bracket => format!(
            "{}{}{}",
            s().dim().paint("["),
            color(text),
            s().dim().paint("]")
        ),
        BadgeVariant::Dot => format!("{} {}", color("●"), text),
        BadgeVariant::Pill => color(&format!(" {} ", text)),
    }
}
