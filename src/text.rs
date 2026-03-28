use unicode_width::UnicodeWidthStr;

use crate::ansi::{measure_width, strip_ansi, wrap_ansi};
use crate::style::RESET;
use crate::unicode::grapheme_segments;
use crate::writer::ansi_enabled;

/// Truncate `text` to at most `width` visible columns, appending `ellipsis` when truncated.
///
/// - ANSI escape sequences are skipped and do not count toward width.
/// - If the text already fits within `width`, it is returned unchanged.
/// - If `ellipsis` is wider than `width`, the ellipsis itself is truncated to `width` chars.
pub fn truncate(text: &str, width: usize, ellipsis: &str) -> String {
    if measure_width(text) <= width {
        return text.to_string();
    }

    let ellipsis_width = measure_width(ellipsis);

    // If ellipsis is wider than allowed width, just truncate the ellipsis
    if ellipsis_width >= width {
        // Return first `width` characters of ellipsis
        let stripped = strip_ansi(ellipsis);
        let result: String = stripped.chars().take(width).collect();
        return result;
    }

    let target_width = width - ellipsis_width;

    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut result = String::with_capacity(text.len());
    let mut visible_width: usize = 0;
    let mut has_ansi = false;

    'outer: while i < len {
        // CSI sequence: \x1b[ ... letter
        if bytes[i] == b'\x1b' && i + 1 < len && bytes[i + 1] == b'[' {
            has_ansi = true;
            let start = i;
            i += 2;
            while i < len && !(bytes[i].is_ascii_alphabetic()) {
                i += 1;
            }
            if i < len {
                i += 1; // consume final letter
            }
            result.push_str(&text[start..i]);
            continue;
        }

        // OSC sequence: \x1b] ... BEL or ESC \
        if bytes[i] == b'\x1b' && i + 1 < len && bytes[i + 1] == b']' {
            has_ansi = true;
            let start = i;
            i += 2;
            while i < len {
                if bytes[i] == b'\x07' {
                    i += 1;
                    break;
                } else if bytes[i] == b'\x1b' && i + 1 < len && bytes[i + 1] == b'\\' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            result.push_str(&text[start..i]);
            continue;
        }

        // Plain text segment: find up to the next escape or end of string
        let next_escape = {
            let mut j = i;
            while j < len && bytes[j] != b'\x1b' {
                j += 1;
            }
            j
        };

        let plain = &text[i..next_escape];

        // Walk graphemes of plain segment
        let mut consumed_bytes = 0;
        for seg in grapheme_segments(plain) {
            let cw = UnicodeWidthStr::width(seg.segment.as_str());
            if visible_width + cw > target_width {
                // We've reached the truncation point
                result.push_str(&plain[..consumed_bytes]);
                let base = if ansi_enabled() {
                    result
                } else {
                    strip_ansi(&result)
                };
                let reset = if ansi_enabled() && (has_ansi || base.contains('\x1b')) {
                    RESET
                } else {
                    ""
                };
                return base + reset + ellipsis;
            }
            visible_width += cw;
            consumed_bytes += seg.segment.len();
        }

        result.push_str(&plain[..consumed_bytes]);
        i = next_escape;

        if i >= len {
            break 'outer;
        }
    }

    // Consumed all text (shouldn't normally reach here given the measure_width check at top,
    // but handle gracefully)
    let base = if ansi_enabled() {
        result
    } else {
        strip_ansi(&result)
    };
    let reset = if ansi_enabled() && (has_ansi || base.contains('\x1b')) {
        RESET
    } else {
        ""
    };
    base + reset + ellipsis
}

/// Indent every line of `text` by `level` repetitions of `char_str`.
pub fn indent(text: &str, level: usize, char_str: &str) -> String {
    let prefix: String = char_str.repeat(level);
    text.split('\n')
        .map(|line| format!("{}{}", prefix, line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Pad `text` to `width` visible columns using spaces.
///
/// `align` is one of `"left"`, `"right"`, or `"center"`.
/// If the text is already wider than `width`, it is returned unchanged.
pub fn pad(text: &str, width: usize, align: &str) -> String {
    let display_width = measure_width(text);
    if display_width >= width {
        return text.to_string();
    }
    let diff = width - display_width;
    match align {
        "right" => {
            let spaces = " ".repeat(diff);
            format!("{}{}", spaces, text)
        }
        "center" => {
            let left = diff / 2;
            let right = diff - left;
            let left_spaces = " ".repeat(left);
            let right_spaces = " ".repeat(right);
            format!("{}{}{}", left_spaces, text, right_spaces)
        }
        _ => {
            // "left" (default)
            let spaces = " ".repeat(diff);
            format!("{}{}", text, spaces)
        }
    }
}

/// Create an OSC 8 hyperlink, or a plain `text (url)` fallback for non-TTY output.
pub fn link(text: &str, url: &str, is_tty: bool) -> String {
    if !is_tty {
        format!("{} ({})", text, url)
    } else {
        format!("\x1b]8;;{}\x07{}\x1b]8;;\x07", url, text)
    }
}

/// Word-wrap `text` so no line exceeds `width` visible columns.
///
/// Delegates to `crate::ansi::wrap_ansi`.
pub fn wrap(text: &str, width: usize) -> String {
    wrap_ansi(text, width)
}
