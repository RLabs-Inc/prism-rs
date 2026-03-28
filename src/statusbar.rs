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
#[derive(Debug, Clone, Default)]
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
    let sep_color: fn(&str) -> String = config.separator_color.unwrap_or(|t| s().dim().render(t));

    let pad = " ".repeat(indent);
    let styled_sep = sep_color(separator);

    let left_parts: Vec<String> = config.left.iter().map(|seg| seg.resolve()).collect();
    let left_str = left_parts.join(&styled_sep);

    let right_str = config
        .right
        .as_ref()
        .map(|seg| seg.resolve())
        .unwrap_or_default();

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
    let fill_width = available_width
        .saturating_sub(left_width + right_width)
        .max(1);

    format!(
        "{}{}{}{}",
        pad,
        left_display,
        " ".repeat(fill_width),
        right_str
    )
}

/// Render a status bar that wraps to multiple lines when content doesn't fit.
///
/// Returns a `Vec<String>` where each element is one line. The right-aligned
/// segment always appears on the first line. If left segments overflow, they
/// wrap to subsequent lines with the same indent and separator style.
///
/// This is the preferred rendering function when the status bar content may
/// exceed terminal width — instead of truncating with "…", the content flows
/// to the next line so no information is lost.
pub fn statusbar_render_wrapped(config: &StatusBarConfig, total_width: usize) -> Vec<String> {
    let indent = config.indent.unwrap_or(2);
    let separator = config.separator.as_deref().unwrap_or(" \u{2502} "); // " │ "
    let sep_width = measure_width(separator);
    let sep_color: fn(&str) -> String = config.separator_color.unwrap_or(|t| s().dim().render(t));
    let styled_sep = sep_color(separator);
    let pad = " ".repeat(indent);

    // Resolve all segments
    let left_resolved: Vec<String> = config.left.iter().map(|seg| seg.resolve()).collect();
    let left_widths: Vec<usize> = left_resolved.iter().map(|s| measure_width(s)).collect();

    let right_str = config.right.as_ref().map(|seg| seg.resolve()).unwrap_or_default();
    let right_width = measure_width(&right_str);

    let available_width = total_width.saturating_sub(indent);

    // Try single-line first
    let total_left_width: usize = left_widths.iter().sum::<usize>()
        + if left_widths.len() > 1 { (left_widths.len() - 1) * sep_width } else { 0 };
    let total_needed = total_left_width + if right_width > 0 { 1 + right_width } else { 0 };

    if total_needed <= available_width {
        // Fits in one line — use standard render
        return vec![statusbar_render(config, total_width)];
    }

    // Multi-line: greedily pack segments into lines
    let mut lines = Vec::new();
    let mut current_parts: Vec<String> = Vec::new();
    let mut current_width: usize = 0;

    // First line reserves space for right segment
    let first_line_max = if right_width > 0 {
        available_width.saturating_sub(right_width + 1)
    } else {
        available_width
    };
    let mut line_max = first_line_max;

    for (i, seg) in left_resolved.iter().enumerate() {
        let seg_width = left_widths[i];
        let with_sep = if current_parts.is_empty() { seg_width } else { sep_width + seg_width };

        if current_width + with_sep > line_max && !current_parts.is_empty() {
            // Flush current line
            let left_str = current_parts.join(&styled_sep);
            let left_w = measure_width(&left_str);

            if lines.is_empty() && !right_str.is_empty() {
                // First line with right alignment
                let fill = available_width.saturating_sub(left_w + right_width).max(1);
                lines.push(format!("{}{}{}{}", pad, left_str, " ".repeat(fill), right_str));
            } else {
                lines.push(format!("{}{}", pad, left_str));
            }

            current_parts.clear();
            current_width = 0;
            line_max = available_width; // subsequent lines get full width
        }

        if !current_parts.is_empty() {
            current_width += sep_width;
        }
        current_parts.push(seg.clone());
        current_width += seg_width;
    }

    // Flush remaining
    if !current_parts.is_empty() {
        let left_str = current_parts.join(&styled_sep);
        let left_w = measure_width(&left_str);

        if lines.is_empty() && !right_str.is_empty() {
            let fill = available_width.saturating_sub(left_w + right_width).max(1);
            lines.push(format!("{}{}{}{}", pad, left_str, " ".repeat(fill), right_str));
        } else {
            lines.push(format!("{}{}", pad, left_str));
        }
    }

    // If right segment wasn't placed yet (all segments fit on first line somehow)
    if lines.is_empty() {
        if !right_str.is_empty() {
            let fill = available_width.saturating_sub(right_width).max(1);
            lines.push(format!("{}{}{}", pad, " ".repeat(fill), right_str));
        } else {
            lines.push(pad);
        }
    }

    lines
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
