use unicode_width::UnicodeWidthChar;

/// Strip all ANSI escape sequences from `text`, returning only the visible characters.
///
/// Handles:
/// - CSI sequences: `ESC [ ... <letter>` (parameters + final byte A-Z/a-z)
/// - OSC sequences: `ESC ] ... BEL` or `ESC ] ... ST` (ST = ESC \)
/// - Standalone ESC (dropped)
pub fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'\x1b' {
            i += 1;
            if i >= len {
                break;
            }
            match bytes[i] {
                b'[' => {
                    // CSI sequence: ESC [ <params> <final>
                    // params: 0x20–0x3F, intermediates: 0x20–0x2F, final: 0x40–0x7E
                    i += 1;
                    while i < len && (bytes[i] >= 0x20 && bytes[i] <= 0x3F) {
                        i += 1;
                    }
                    // consume final byte (0x40–0x7E)
                    if i < len && bytes[i] >= 0x40 && bytes[i] <= 0x7E {
                        i += 1;
                    }
                }
                b']' => {
                    // OSC sequence: ESC ] ... BEL  OR  ESC ] ... ESC \
                    i += 1;
                    while i < len {
                        if bytes[i] == b'\x07' {
                            // BEL terminates
                            i += 1;
                            break;
                        } else if bytes[i] == b'\x1b' && i + 1 < len && bytes[i + 1] == b'\\' {
                            // ST (String Terminator) = ESC \
                            i += 2;
                            break;
                        }
                        i += 1;
                    }
                }
                b'\\' => {
                    // Lone ST (shouldn't appear without preceding OSC, but consume it)
                    i += 1;
                }
                _ => {
                    // Standalone ESC + unknown byte — skip both
                    i += 1;
                }
            }
        } else {
            // Regular character — find how many bytes this char occupies and push it
            let ch_start = i;
            i += 1;
            // Walk forward until we're on a character boundary
            while i < len && (bytes[i] & 0xC0) == 0x80 {
                i += 1;
            }
            out.push_str(&text[ch_start..i]);
        }
    }

    out
}

/// Return the visible column width of `text` (ANSI sequences have zero width).
pub fn measure_width(text: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(strip_ansi(text).as_str())
}

// ---------------------------------------------------------------------------
// wrap_ansi implementation
// ---------------------------------------------------------------------------

/// A token produced by the ANSI-aware tokenizer.
#[derive(Debug, Clone)]
enum Token {
    /// A run of visible text (may contain multiple chars, all same "word slot").
    Visible(char),
    /// A complete ANSI escape sequence (raw bytes, zero visual width).
    Escape(String),
    /// A space character (word boundary).
    Space,
    /// A newline (hard break in source).
    Newline,
}

/// Tokenize a string into a sequence of `Token`s, preserving ANSI sequences verbatim.
fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'\n' {
            tokens.push(Token::Newline);
            i += 1;
        } else if bytes[i] == b' ' {
            tokens.push(Token::Space);
            i += 1;
        } else if bytes[i] == b'\x1b' {
            // Collect the full escape sequence
            let start = i;
            i += 1;
            if i < len {
                match bytes[i] {
                    b'[' => {
                        i += 1;
                        while i < len && bytes[i] >= 0x20 && bytes[i] <= 0x3F {
                            i += 1;
                        }
                        if i < len && bytes[i] >= 0x40 && bytes[i] <= 0x7E {
                            i += 1;
                        }
                    }
                    b']' => {
                        i += 1;
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
                    }
                    b'\\' => {
                        i += 1;
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            tokens.push(Token::Escape(text[start..i].to_string()));
        } else {
            // UTF-8 character
            let ch_start = i;
            i += 1;
            while i < len && (bytes[i] & 0xC0) == 0x80 {
                i += 1;
            }
            // Safe: we know these byte boundaries are valid UTF-8
            let ch = text[ch_start..i].chars().next().unwrap();
            if ch == '\n' {
                tokens.push(Token::Newline);
            } else if ch == ' ' {
                tokens.push(Token::Space);
            } else {
                tokens.push(Token::Visible(ch));
            }
        }
    }

    tokens
}

/// Determine if a CSI sequence is a "reset" (clears all styling).
fn is_reset_sequence(seq: &str) -> bool {
    // \x1b[m or \x1b[0m or \x1b[00m etc.
    seq == "\x1b[m" || seq == "\x1b[0m" || seq == "\x1b[00m" || seq == "\x1b[0;m"
}

/// Word-wrap `text` so no line exceeds `width` visible columns.
///
/// - Wraps at word boundaries (spaces).
/// - Hard-breaks words longer than `width`.
/// - Preserves existing newlines.
/// - Passes ANSI escape sequences through without counting their width.
/// - Re-emits active ANSI codes after inserted newlines so styling continues.
pub fn wrap_ansi(text: &str, width: usize) -> String {
    if width == 0 || text.is_empty() {
        return text.to_string();
    }

    // Split on existing newlines and wrap each logical line independently,
    // then rejoin.
    let lines: Vec<&str> = text.split('\n').collect();
    let wrapped: Vec<String> = lines.iter().map(|line| wrap_line(line, width)).collect();
    wrapped.join("\n")
}

/// Wrap a single line (no embedded newlines) to `width` columns.
fn wrap_line(line: &str, width: usize) -> String {
    if line.is_empty() {
        return String::new();
    }

    let tokens = tokenize(line);

    // Group tokens into "words": a word is a sequence of Visible/Escape tokens
    // between Spaces. Spaces are separators.
    //
    // We'll walk token-by-token and build the output directly.

    let mut out = String::with_capacity(line.len() + 16);
    let mut col: usize = 0; // current column position on this output line
    // Active style sequences (re-emitted after a line break)
    let mut active_styles: Vec<String> = Vec::new();

    // We process word-by-word. First collect words as (visible_width, raw_string) pairs.
    // A "word" includes any embedded ANSI escapes.
    let words = collect_words(&tokens);

    let mut first_word_on_line = true;

    for word in &words {
        match word {
            WordChunk::Spaces(n) => {
                if !first_word_on_line {
                    // Emit spaces only if we're not at the start of a line
                    // and they fit (or even if they don't fit — we'll handle wrap below)
                    // Actually: spaces between words: emit as-is, they'll be trimmed by
                    // the next word's wrap check below. Just emit them.
                    for _ in 0..*n {
                        out.push(' ');
                        col += 1;
                    }
                }
            }
            WordChunk::Word { vis_width, content } => {
                // Decide where to put this word
                if first_word_on_line {
                    // First word on the line — always emit (hard-break if needed)
                    let styles_snapshot = active_styles.clone();
                    emit_word_hard_break(&mut out, content, *vis_width, width, &mut col, &mut active_styles, &styles_snapshot);
                    first_word_on_line = false;
                } else {
                    // Would this word fit on the current line?
                    if col + vis_width <= width {
                        // Fits — emit directly
                        emit_word_raw(&mut out, content, &mut col, &mut active_styles);
                    } else {
                        // Doesn't fit — wrap
                        // Remove trailing spaces from current line
                        while out.ends_with(' ') {
                            out.pop();
                            col = col.saturating_sub(1);
                        }
                        out.push('\n');
                        col = 0;
                        // Re-emit active styles
                        let styles_snapshot = active_styles.clone();
                        for s in &styles_snapshot {
                            out.push_str(s);
                        }
                        // Now emit the word (may need hard-break if word itself > width)
                        emit_word_hard_break(&mut out, content, *vis_width, width, &mut col, &mut active_styles, &styles_snapshot);
                        first_word_on_line = false;
                    }
                }
            }
        }
    }

    out
}

/// Emit a word, hard-breaking it character by character if it exceeds `width`.
/// `styles_on_break` are the active styles to re-emit after each hard break.
fn emit_word_hard_break(
    out: &mut String,
    content: &[WordToken],
    vis_width: usize,
    width: usize,
    col: &mut usize,
    active_styles: &mut Vec<String>,
    styles_on_break: &[String],
) {
    if vis_width <= width {
        // No hard break needed
        emit_word_raw(out, content, col, active_styles);
        return;
    }

    // Hard break: emit char by char, wrapping at width
    for tok in content {
        match tok {
            WordToken::Escape(seq) => {
                out.push_str(seq);
                update_active_styles(active_styles, seq);
            }
            WordToken::Char(ch) => {
                let ch_w = UnicodeWidthChar::width(*ch).unwrap_or(1);
                if *col + ch_w > width && *col > 0 {
                    out.push('\n');
                    *col = 0;
                    for s in styles_on_break {
                        out.push_str(s);
                    }
                }
                out.push(*ch);
                *col += ch_w;
            }
        }
    }
}

/// Emit a word (sequence of WordTokens) into `out`, updating `col` and `active_styles`.
fn emit_word_raw(
    out: &mut String,
    content: &[WordToken],
    col: &mut usize,
    active_styles: &mut Vec<String>,
) {
    for tok in content {
        match tok {
            WordToken::Escape(seq) => {
                out.push_str(seq);
                update_active_styles(active_styles, seq);
            }
            WordToken::Char(ch) => {
                out.push(*ch);
                *col += UnicodeWidthChar::width(*ch).unwrap_or(1);
            }
        }
    }
}

/// Update the active styles stack given a newly encountered escape sequence.
fn update_active_styles(active: &mut Vec<String>, seq: &str) {
    if is_reset_sequence(seq) {
        active.clear();
    } else if seq.starts_with("\x1b[") {
        // It's a CSI sequence — push it as an active style
        // (we don't try to be clever about which ones "cancel" others)
        active.push(seq.to_string());
    }
    // OSC and other sequences are passed through but not tracked for re-emission
}

// ---------------------------------------------------------------------------
// Word collection helpers
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum WordToken {
    Char(char),
    Escape(String),
}

#[derive(Debug)]
enum WordChunk {
    Spaces(usize),
    Word { vis_width: usize, content: Vec<WordToken> },
}

/// Collect tokens into a sequence of `WordChunk`s.
fn collect_words(tokens: &[Token]) -> Vec<WordChunk> {
    let mut chunks: Vec<WordChunk> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Space => {
                // Count consecutive spaces
                let mut n = 0;
                while i < tokens.len() {
                    if let Token::Space = &tokens[i] {
                        n += 1;
                        i += 1;
                    } else {
                        break;
                    }
                }
                chunks.push(WordChunk::Spaces(n));
            }
            Token::Newline => {
                // Should not appear here (we split on newlines before calling wrap_line)
                i += 1;
            }
            Token::Visible(_) | Token::Escape(_) => {
                // Collect a word: consecutive Visible/Escape tokens
                let mut word_tokens: Vec<WordToken> = Vec::new();
                let mut vis_width = 0usize;
                while i < tokens.len() {
                    match &tokens[i] {
                        Token::Visible(ch) => {
                            vis_width += UnicodeWidthChar::width(*ch).unwrap_or(1);
                            word_tokens.push(WordToken::Char(*ch));
                            i += 1;
                        }
                        Token::Escape(seq) => {
                            word_tokens.push(WordToken::Escape(seq.clone()));
                            i += 1;
                        }
                        _ => break,
                    }
                }
                chunks.push(WordChunk::Word { vis_width, content: word_tokens });
            }
        }
    }

    chunks
}
