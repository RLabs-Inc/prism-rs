//! Single-line status bar with left/right alignment.
//!
//! Ports `statusbar.ts` — renders a left-aligned segment list and an optional
//! right-aligned segment, separated and padded to fill the terminal width.

use crate::ansi::measure_width;
use crate::style::s;
use crate::text::truncate;
use crate::writer;

/// A segment of the status bar: either plain text or styled text.
#[derive(Debug, Clone)]
pub enum Segment {
    /// Plain text segment.
    Text(String),
    /// Text with an optional color/style function.
    Styled {
        text: String,
        color: Option<fn(&str) -> String>,
    },
}

impl Segment {
    /// Resolve the segment into its final rendered string.
    pub fn resolve(&self) -> String {
        match self {
            Segment::Text(t) => t.clone(),
            Segment::Styled { text, color } => {
                if let Some(f) = color {
                    f(text)
                } else {
                    text.clone()
                }
            }
        }
    }

    /// Resolve the segment as plain text (no color applied), for non-TTY output.
    pub fn resolve_plain(&self) -> String {
        match self {
            Segment::Text(t) => t.clone(),
            Segment::Styled { text, .. } => text.clone(),
        }
    }
}

/// Configuration for rendering a status bar.
#[derive(Debug, Clone)]
pub struct StatusBarConfig {
    /// Left-aligned segments, joined by the separator.
    pub left: Vec<Segment>,
    /// Optional right-aligned segment.
    pub right: Option<Segment>,
    /// Separator string between left segments (default `" │ "`).
    pub separator: Option<String>,
    /// Left indent in spaces (default 2).
    pub indent: Option<usize>,
    /// Optional color function for the separator.
    pub separator_color: Option<fn(&str) -> String>,
}

impl Default for StatusBarConfig {
    fn default() -> Self {
        Self {
            left: Vec::new(),
            right: None,
            separator: None,
            indent: None,
            separator_color: None,
        }
    }
}

/// Render a status bar using the current terminal state (TTY detection + width).
///
/// For non-TTY output, segments are joined plainly without styling.
pub fn statusbar(config: &StatusBarConfig) -> String {
    if !writer::is_tty() {
        return statusbar_plain(config);
    }
    let width = writer::term_width() as usize;
    statusbar_render(config, width)
}

/// Pure rendering function with an explicit width. Use this in tests to avoid
/// TTY-dependent behaviour.
pub fn statusbar_render(config: &StatusBarConfig, total_width: usize) -> String {
    let indent = config.indent.unwrap_or(2);
    let separator = config.separator.as_deref().unwrap_or(" \u{2502} "); // " │ "
    let sep_color: fn(&str) -> String = config
        .separator_color
        .unwrap_or(|t| s().dim().render(t));

    let pad = " ".repeat(indent);
    let styled_sep = sep_color(separator);

    let left_parts: Vec<String> = config.left.iter().map(|seg| seg.resolve()).collect();
    let left_str = left_parts.join(&styled_sep);

    let right_str = config.right.as_ref().map(|seg| seg.resolve()).unwrap_or_default();

    let available_width = total_width.saturating_sub(indent);

    if right_str.is_empty() {
        return format!("{}{}", pad, truncate(&left_str, available_width, "…"));
    }

    let right_width = measure_width(&right_str);
    if right_width >= available_width {
        return format!("{}{}", pad, truncate(&right_str, available_width, "…"));
    }

    let max_left_width = available_width.saturating_sub(right_width + 1);
    let left_display = truncate(&left_str, max_left_width, "…");
    let left_width = measure_width(&left_display);
    let fill_width = available_width.saturating_sub(left_width + right_width).max(1);

    format!("{}{}{}{}", pad, left_display, " ".repeat(fill_width), right_str)
}

/// Plain-text rendering for non-TTY output (no styling, no fill).
fn statusbar_plain(config: &StatusBarConfig) -> String {
    let indent = config.indent.unwrap_or(2);
    let separator = config.separator.as_deref().unwrap_or(" \u{2502} "); // " │ "

    let left_parts: Vec<String> = config.left.iter().map(|seg| seg.resolve_plain()).collect();
    let left_str = left_parts.join(separator);

    let right_str = config
        .right
        .as_ref()
        .map(|seg| seg.resolve_plain())
        .unwrap_or_default();

    let pad = " ".repeat(indent);
    if right_str.is_empty() {
        format!("{}{}", pad, left_str)
    } else {
        format!("{}{}  {}", pad, left_str, right_str)
    }
}
