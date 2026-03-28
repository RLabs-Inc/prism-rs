use crate::style::s;
use crate::writer::ansi_enabled;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BannerStyle {
    #[default]
    Block,
    Shade,
    Dots,
    Ascii,
    Outline,
}

/// Color callback: takes a string, returns a styled string.
/// Default when ANSI is enabled: `s().bold().render(t)` (matching TS `s.bold`).
pub type BannerColorFn = fn(&str) -> String;

pub struct BannerOptions {
    pub style: BannerStyle,
    pub color: Option<BannerColorFn>,
    pub letter_spacing: usize, // gap columns between letters, default 1
}

impl Default for BannerOptions {
    fn default() -> Self {
        Self {
            style: BannerStyle::Block,
            color: if ansi_enabled() {
                Some(|t: &str| s().bold().render(t))
            } else {
                None
            },
            letter_spacing: 1,
        }
    }
}

// Each glyph is 5 rows of 5 bits (bit 4 = leftmost column).
// Copied exactly from the TS reference implementation.
const FONT: &[(char, [u8; 5])] = &[
    ('A', [0b01110, 0b10001, 0b11111, 0b10001, 0b10001]),
    ('B', [0b11110, 0b10001, 0b11110, 0b10001, 0b11110]),
    ('C', [0b01111, 0b10000, 0b10000, 0b10000, 0b01111]),
    ('D', [0b11110, 0b10001, 0b10001, 0b10001, 0b11110]),
    ('E', [0b11111, 0b10000, 0b11110, 0b10000, 0b11111]),
    ('F', [0b11111, 0b10000, 0b11110, 0b10000, 0b10000]),
    ('G', [0b01111, 0b10000, 0b10011, 0b10001, 0b01111]),
    ('H', [0b10001, 0b10001, 0b11111, 0b10001, 0b10001]),
    ('I', [0b11111, 0b00100, 0b00100, 0b00100, 0b11111]),
    ('J', [0b00111, 0b00001, 0b00001, 0b10001, 0b01110]),
    ('K', [0b10001, 0b10010, 0b11100, 0b10010, 0b10001]),
    ('L', [0b10000, 0b10000, 0b10000, 0b10000, 0b11111]),
    ('M', [0b10001, 0b11011, 0b10101, 0b10001, 0b10001]),
    ('N', [0b10001, 0b11001, 0b10101, 0b10011, 0b10001]),
    ('O', [0b01110, 0b10001, 0b10001, 0b10001, 0b01110]),
    ('P', [0b11110, 0b10001, 0b11110, 0b10000, 0b10000]),
    ('Q', [0b01110, 0b10001, 0b10101, 0b10010, 0b01101]),
    ('R', [0b11110, 0b10001, 0b11110, 0b10010, 0b10001]),
    ('S', [0b01111, 0b10000, 0b01110, 0b00001, 0b11110]),
    ('T', [0b11111, 0b00100, 0b00100, 0b00100, 0b00100]),
    ('U', [0b10001, 0b10001, 0b10001, 0b10001, 0b01110]),
    ('V', [0b10001, 0b10001, 0b10001, 0b01010, 0b00100]),
    ('W', [0b10001, 0b10001, 0b10101, 0b11011, 0b10001]),
    ('X', [0b10001, 0b01010, 0b00100, 0b01010, 0b10001]),
    ('Y', [0b10001, 0b01010, 0b00100, 0b00100, 0b00100]),
    ('Z', [0b11111, 0b00010, 0b00100, 0b01000, 0b11111]),
    ('0', [0b01110, 0b10011, 0b10101, 0b11001, 0b01110]),
    ('1', [0b00100, 0b01100, 0b00100, 0b00100, 0b11111]),
    ('2', [0b01110, 0b10001, 0b00110, 0b01000, 0b11111]),
    ('3', [0b11110, 0b00001, 0b01110, 0b00001, 0b11110]),
    ('4', [0b10001, 0b10001, 0b11111, 0b00001, 0b00001]),
    ('5', [0b11111, 0b10000, 0b11110, 0b00001, 0b11110]),
    ('6', [0b01110, 0b10000, 0b11110, 0b10001, 0b01110]),
    ('7', [0b11111, 0b00001, 0b00010, 0b00100, 0b00100]),
    ('8', [0b01110, 0b10001, 0b01110, 0b10001, 0b01110]),
    ('9', [0b01110, 0b10001, 0b01111, 0b00001, 0b01110]),
    ('!', [0b00100, 0b00100, 0b00100, 0b00000, 0b00100]),
    ('?', [0b01110, 0b10001, 0b00110, 0b00000, 0b00100]),
    ('.', [0b00000, 0b00000, 0b00000, 0b00000, 0b00100]),
    ('-', [0b00000, 0b00000, 0b11111, 0b00000, 0b00000]),
    ('_', [0b00000, 0b00000, 0b00000, 0b00000, 0b11111]),
    ('/', [0b00001, 0b00010, 0b00100, 0b01000, 0b10000]),
    (':', [0b00000, 0b00100, 0b00000, 0b00100, 0b00000]),
    (' ', [0b00000, 0b00000, 0b00000, 0b00000, 0b00000]),
];

/// Look up the 5-row bitmap for a character.
/// Returns the space glyph for unknown characters.
fn get_glyph(ch: char) -> [u8; 5] {
    let upper = ch.to_ascii_uppercase();
    for &(c, bitmap) in FONT {
        if c == upper {
            return bitmap;
        }
    }
    // Unknown: return space
    [0u8; 5]
}

/// Get the on/off cell strings for a given style.
fn style_cells(style: BannerStyle) -> (&'static str, &'static str) {
    match style {
        BannerStyle::Block => ("██", "  "),
        BannerStyle::Shade => ("▓▓", "░░"),
        BannerStyle::Dots => ("⣿⣿", "  "),
        BannerStyle::Ascii => ("##", "  "),
        BannerStyle::Outline => ("▐▌", "  "),
    }
}

/// Apply color styling to a row string.
fn apply_color(text: &str, color: &Option<BannerColorFn>) -> String {
    match color {
        Some(f) => f(text),
        None => text.to_string(),
    }
}

/// Render text as 5-row block-letter art.
pub fn banner(text: &str, options: &BannerOptions) -> String {
    let (on, off) = style_cells(options.style);

    // For each of the 5 rows, build the row string across all glyphs.
    let chars: Vec<char> = text.chars().collect();

    let rows: Vec<String> = (0..5)
        .map(|row| {
            let mut line = String::new();
            let mut rendered = 0usize;
            for &ch in &chars {
                let bitmap = get_glyph(ch);
                if bitmap == [0u8; 5] && ch != ' ' {
                    // skip truly unknown chars (but not space which has all-zero bitmap)
                    let upper = ch.to_ascii_uppercase();
                    let known = FONT.iter().any(|&(c, _)| c == upper);
                    if !known {
                        continue;
                    }
                }

                // Add letter spacing gap (using the style's off character)
                if rendered > 0 {
                    for _ in 0..options.letter_spacing {
                        line.push_str(off);
                    }
                }

                let bits = bitmap[row];
                // 5 bits, bit 4 = leftmost
                for bit_pos in (0..5).rev() {
                    if (bits >> bit_pos) & 1 == 1 {
                        line.push_str(on);
                    } else {
                        line.push_str(off);
                    }
                }
                rendered += 1;
            }
            apply_color(&line, &options.color)
        })
        .collect();

    rows.join("\n")
}
