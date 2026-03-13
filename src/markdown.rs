use crate::style::s;
use crate::writer::ansi_enabled;

// ---------------------------------------------------------------------------
// Inline formatter: processes **bold**, *italic*, `code`, [text](url)
// ---------------------------------------------------------------------------

fn format_inline(text: &str) -> String {
    if !ansi_enabled() {
        return text.to_string();
    }

    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut out = String::with_capacity(text.len() * 2);
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        // **bold** or __bold__
        if ch == '*' && i + 1 < len && chars[i + 1] == '*' {
            if let Some(end) = find_closing(&chars, i + 2, "**") {
                let inner: String = chars[i + 2..end].iter().collect();
                out.push_str(&s().bold().paint(&format_inline(&inner)));
                i = end + 2;
                continue;
            }
        }

        // *italic* (single star, not double)
        if ch == '*' && (i + 1 >= len || chars[i + 1] != '*') {
            if let Some(end) = find_closing_single(&chars, i + 1, '*') {
                let inner: String = chars[i + 1..end].iter().collect();
                out.push_str(&s().italic().paint(&format_inline(&inner)));
                i = end + 1;
                continue;
            }
        }

        // _italic_ (underscore)
        if ch == '_' && (i + 1 >= len || chars[i + 1] != '_') {
            if let Some(end) = find_closing_single(&chars, i + 1, '_') {
                let inner: String = chars[i + 1..end].iter().collect();
                out.push_str(&s().italic().paint(&format_inline(&inner)));
                i = end + 1;
                continue;
            }
        }

        // `inline code`
        if ch == '`' {
            if let Some(end) = find_closing_single(&chars, i + 1, '`') {
                let inner: String = chars[i + 1..end].iter().collect();
                out.push_str(&s().cyan().paint(&inner));
                i = end + 1;
                continue;
            }
        }

        // [text](url)
        if ch == '[' {
            if let Some((link_text, url, after)) = parse_link(&chars, i) {
                let styled_text = s().underline().blue().paint(&link_text);
                let styled_url = s().dim().paint(&format!(" ({})", url));
                out.push_str(&styled_text);
                out.push_str(&styled_url);
                i = after;
                continue;
            }
        }

        out.push(ch);
        i += 1;
    }

    out
}

/// Find closing `**` (two-char marker) starting at `from`.
fn find_closing(chars: &[char], from: usize, marker: &str) -> Option<usize> {
    let m: Vec<char> = marker.chars().collect();
    let mlen = m.len();
    let len = chars.len();
    let mut i = from;
    while i + mlen <= len {
        if &chars[i..i + mlen] == m.as_slice() {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Find closing single character `closer` starting at `from`.
fn find_closing_single(chars: &[char], from: usize, closer: char) -> Option<usize> {
    for (i, &ch) in chars.iter().enumerate().skip(from) {
        if ch == closer {
            return Some(i);
        }
        // Don't cross newlines
        if ch == '\n' {
            return None;
        }
    }
    None
}

/// Parse `[text](url)` starting at `[` (index `start`).
/// Returns `(text, url, index_after_closing_paren)` or None.
fn parse_link(chars: &[char], start: usize) -> Option<(String, String, usize)> {
    let len = chars.len();
    // find ]
    let mut i = start + 1;
    while i < len && chars[i] != ']' && chars[i] != '\n' {
        i += 1;
    }
    if i >= len || chars[i] != ']' {
        return None;
    }
    let text: String = chars[start + 1..i].iter().collect();
    let close_bracket = i;
    // expect (
    if close_bracket + 1 >= len || chars[close_bracket + 1] != '(' {
        return None;
    }
    let url_start = close_bracket + 2;
    let mut j = url_start;
    while j < len && chars[j] != ')' && chars[j] != '\n' {
        j += 1;
    }
    if j >= len || chars[j] != ')' {
        return None;
    }
    let url: String = chars[url_start..j].iter().collect();
    Some((text, url, j + 1))
}

// ---------------------------------------------------------------------------
// Line-by-line markdown parser
// ---------------------------------------------------------------------------

/// Render markdown `text` to a terminal-styled string.
///
/// Supports: headings (#/##/###), **bold**, *italic*, `code`,
/// ``` code blocks ```, - / * lists, 1. numbered lists, > blockquotes,
/// --- horizontal rules, and [text](url) links.
pub fn md(text: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut out_lines: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    let total = lines.len();

    while i < total {
        let line = lines[i];

        // --- Code block (``` ... ```) ---
        if line.trim_start().starts_with("```") {
            // emit separator
            let sep = s().dim().paint("─────────────────────────────");
            out_lines.push(sep.clone());
            i += 1;
            while i < total {
                let code_line = lines[i];
                if code_line.trim_start().starts_with("```") {
                    i += 1;
                    break;
                }
                out_lines.push(s().cyan().paint(code_line));
                i += 1;
            }
            out_lines.push(sep);
            continue;
        }

        // --- Horizontal rule --- (---, ***, ___)
        if matches!(line.trim(), "---" | "***" | "___") {
            let divider = s().dim().paint(&"─".repeat(40));
            out_lines.push(divider);
            i += 1;
            continue;
        }

        // --- Headings ---
        if let Some(text) = line.strip_prefix("### ") {
            let styled = s().bold().dim().paint(&format_inline(text));
            out_lines.push(styled);
            i += 1;
            continue;
        }
        if let Some(text) = line.strip_prefix("## ") {
            let styled = s().bold().paint(&format_inline(text));
            out_lines.push(styled);
            i += 1;
            continue;
        }
        if let Some(text) = line.strip_prefix("# ") {
            let styled = s().bold().underline().paint(&format_inline(text));
            out_lines.push(styled);
            i += 1;
            continue;
        }

        // --- Blockquote ---
        if let Some(text) = line.strip_prefix("> ") {
            let prefix = s().dim().paint("│ ");
            let content = s().italic().paint(&format_inline(text));
            out_lines.push(format!("{}{}", prefix, content));
            i += 1;
            continue;
        }
        if line == ">" {
            let prefix = s().dim().paint("│");
            out_lines.push(prefix);
            i += 1;
            continue;
        }

        // --- Unordered list: - item or * item ---
        if line.starts_with("- ") || line.starts_with("* ") {
            let text = &line[2..];
            let bullet = s().dim().paint("• ");
            let content = format_inline(text);
            out_lines.push(format!("{}{}", bullet, content));
            i += 1;
            continue;
        }

        // --- Numbered list: 1. item, 2. item, etc. ---
        {
            let trimmed = line;
            let mut num_end = 0;
            let lchars: Vec<char> = trimmed.chars().collect();
            while num_end < lchars.len() && lchars[num_end].is_ascii_digit() {
                num_end += 1;
            }
            if num_end > 0
                && num_end + 1 < lchars.len()
                && lchars[num_end] == '.'
                && lchars[num_end + 1] == ' '
            {
                let num_part: String = lchars[..num_end].iter().collect();
                let text_part: String = lchars[num_end + 2..].iter().collect();
                let marker = s().dim().paint(&format!("{}. ", num_part));
                let content = format_inline(&text_part);
                out_lines.push(format!("{}{}", marker, content));
                i += 1;
                continue;
            }
        }

        // --- Plain paragraph / inline formatting ---
        out_lines.push(format_inline(line));
        i += 1;
    }

    out_lines.join("\n")
}
