use crate::ansi::{measure_width, wrap_ansi};

// ---------------------------------------------------------------------------
// Border character sets
// ---------------------------------------------------------------------------

/// Box-drawing character set for a border style.
pub struct BorderChars {
    pub tl: &'static str,
    pub tr: &'static str,
    pub bl: &'static str,
    pub br: &'static str,
    pub h: &'static str,
    pub v: &'static str,
    pub lt: &'static str,
    pub rt: &'static str,
    pub tt: &'static str,
    pub bt: &'static str,
    pub cross: &'static str,
}

static SINGLE: BorderChars = BorderChars {
    tl: "┌",
    tr: "┐",
    bl: "└",
    br: "┘",
    h: "─",
    v: "│",
    lt: "├",
    rt: "┤",
    tt: "┬",
    bt: "┴",
    cross: "┼",
};

static DOUBLE: BorderChars = BorderChars {
    tl: "╔",
    tr: "╗",
    bl: "╚",
    br: "╝",
    h: "═",
    v: "║",
    lt: "╠",
    rt: "╣",
    tt: "╦",
    bt: "╩",
    cross: "╬",
};

static ROUNDED: BorderChars = BorderChars {
    tl: "╭",
    tr: "╮",
    bl: "╰",
    br: "╯",
    h: "─",
    v: "│",
    lt: "├",
    rt: "┤",
    tt: "┬",
    bt: "┴",
    cross: "┼",
};

static HEAVY: BorderChars = BorderChars {
    tl: "┏",
    tr: "┓",
    bl: "┗",
    br: "┛",
    h: "━",
    v: "┃",
    lt: "┣",
    rt: "┫",
    tt: "┳",
    bt: "┻",
    cross: "╋",
};

// ---------------------------------------------------------------------------
// BorderStyle enum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default)]
pub enum BorderStyle {
    #[default]
    Single,
    Double,
    Rounded,
    Heavy,
}

impl BorderStyle {
    pub fn chars(&self) -> &'static BorderChars {
        match self {
            BorderStyle::Single => &SINGLE,
            BorderStyle::Double => &DOUBLE,
            BorderStyle::Rounded => &ROUNDED,
            BorderStyle::Heavy => &HEAVY,
        }
    }
}

// ---------------------------------------------------------------------------
// FrameOptions
// ---------------------------------------------------------------------------

pub struct FrameOptions {
    pub border: BorderStyle,
    pub width: Option<usize>,
    pub padding: usize,
    pub title: Option<String>,
}

impl Default for FrameOptions {
    fn default() -> Self {
        Self {
            border: BorderStyle::Single,
            width: None,
            padding: 1,
            title: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Wrap `content` in a bordered box according to `options`.
///
/// The box outer width is `options.width` (default 80).  Content is word-wrapped
/// to fit the inner width (outer - 2 border columns - 2 × padding columns).
pub fn frame(content: &str, options: &FrameOptions) -> String {
    let bc = options.border.chars();
    let outer_width = options.width.unwrap_or(80);
    // inner_width = outer - 2 (left/right border chars each 1 column wide)
    //                      - 2 * padding
    let pad = options.padding;
    let inner_width = outer_width.saturating_sub(2).saturating_sub(2 * pad);

    // Wrap content to inner width
    let wrapped = wrap_ansi(content, if inner_width == 0 { 1 } else { inner_width });
    let content_lines: Vec<&str> = wrapped.split('\n').collect();

    let mut out = String::new();

    // --- Top border ---
    let h_fill = inner_width + 2 * pad; // number of h chars between corners
    let top_line = match &options.title {
        None => {
            format!("{}{}{}\n", bc.tl, bc.h.repeat(h_fill), bc.tr)
        }
        Some(title) => {
            // Layout: TL h* space title space h* TR
            let title_vis = measure_width(title);
            // We need: 1 (TL) + h_fill + 1 (TR) = outer_width total columns
            // Title section occupies: space + title + space = title_vis + 2
            let title_section = title_vis + 2;
            if title_section + 2 > h_fill {
                // Title wider than available — just put it without leading/trailing dashes
                format!("{}{} {} {}{}\n", bc.tl, bc.h, title, bc.h, bc.tr)
            } else {
                let remaining = h_fill.saturating_sub(title_section);
                let left_dashes = remaining / 2;
                let right_dashes = remaining - left_dashes;
                format!(
                    "{}{} {} {}{}\n",
                    bc.tl,
                    bc.h.repeat(left_dashes),
                    title,
                    bc.h.repeat(right_dashes),
                    bc.tr
                )
            }
        }
    };
    out.push_str(&top_line);

    // --- Content lines ---
    let pad_str = " ".repeat(pad);
    for line in &content_lines {
        let vis = measure_width(line);
        let right_pad = if vis < inner_width {
            " ".repeat(inner_width - vis)
        } else {
            String::new()
        };
        out.push_str(&format!(
            "{}{}{}{}{}{}\n",
            bc.v, pad_str, line, right_pad, pad_str, bc.v
        ));
    }

    // --- Bottom border ---
    out.push_str(&format!("{}{}{}\n", bc.bl, bc.h.repeat(h_fill), bc.br));

    out
}

/// Repeat `ch` exactly `width` times (by character count, not byte count).
///
/// For a single-char string this produces a string of `width` repetitions.
pub fn divider(ch: &str, width: usize) -> String {
    ch.repeat(width)
}

/// Draw a centred header: fill line `────  text  ────` to `width` columns.
///
/// Uses the single-line horizontal box-drawing character (─).
pub fn header(text: &str, width: usize) -> String {
    let h = "─";
    let text_vis = measure_width(text);
    // Format: h* space text space h*
    // text section = space + text + space = text_vis + 2
    let text_section = text_vis + 2;
    if text_section + 2 > width {
        // Not enough room for any dashes; just return text
        return text.to_string();
    }
    let remaining = width.saturating_sub(text_section);
    let left = remaining / 2;
    let right = remaining - left;
    format!("{} {} {}", h.repeat(left), text, h.repeat(right))
}
