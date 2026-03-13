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

#[derive(Debug, Clone)]
pub struct BannerOptions {
    pub style: BannerStyle,
    pub color: Option<BannerColor>,
    pub letter_spacing: usize, // gap columns between letters, default 1
}

#[derive(Debug, Clone)]
pub enum BannerColor {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl Default for BannerOptions {
    fn default() -> Self {
        Self {
            style: BannerStyle::Block,
            color: None,
            letter_spacing: 1,
        }
    }
}

// Each glyph is 5 rows of 5 bits (bit 4 = leftmost column).
const FONT: &[(char, [u8; 5])] = &[
    ('A', [0b01110, 0b10001, 0b11111, 0b10001, 0b10001]),
    ('B', [0b11110, 0b10001, 0b11110, 0b10001, 0b11110]),
    ('C', [0b01111, 0b10000, 0b10000, 0b10000, 0b01111]),
    ('D', [0b11110, 0b10001, 0b10001, 0b10001, 0b11110]),
    ('E', [0b11111, 0b10000, 0b11110, 0b10000, 0b11111]),
    ('F', [0b11111, 0b10000, 0b11110, 0b10000, 0b10000]),
    ('G', [0b01111, 0b10000, 0b10111, 0b10001, 0b01111]),
    ('H', [0b10001, 0b10001, 0b11111, 0b10001, 0b10001]),
    ('I', [0b11111, 0b00100, 0b00100, 0b00100, 0b11111]),
    ('J', [0b11111, 0b00010, 0b00010, 0b10010, 0b01100]),
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
    ('1', [0b00100, 0b01100, 0b00100, 0b00100, 0b01110]),
    ('2', [0b01110, 0b10001, 0b00110, 0b01000, 0b11111]),
    ('3', [0b11111, 0b00010, 0b00110, 0b00001, 0b11110]),
    ('4', [0b00110, 0b01010, 0b10010, 0b11111, 0b00010]),
    ('5', [0b11111, 0b10000, 0b11110, 0b00001, 0b11110]),
    ('6', [0b00110, 0b01000, 0b11110, 0b10001, 0b01110]),
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
        BannerStyle::Block   => ("██", "  "),
        BannerStyle::Shade   => ("▓▓", "░░"),
        BannerStyle::Dots    => ("⣿⣿", "  "),
        BannerStyle::Ascii   => ("##", "  "),
        BannerStyle::Outline => ("▐▌", "  "),
    }
}

/// Apply color styling to a row string.
fn apply_color(text: &str, color: &Option<BannerColor>) -> String {
    if !ansi_enabled() {
        return text.to_string();
    }
    match color {
        None => text.to_string(),
        Some(BannerColor::Red)     => s().red().paint(text),
        Some(BannerColor::Green)   => s().green().paint(text),
        Some(BannerColor::Yellow)  => s().yellow().paint(text),
        Some(BannerColor::Blue)    => s().blue().paint(text),
        Some(BannerColor::Magenta) => s().magenta().paint(text),
        Some(BannerColor::Cyan)    => s().cyan().paint(text),
        Some(BannerColor::White)   => s().white().paint(text),
    }
}

/// Render text as 5-row block-letter art.
pub fn banner(text: &str, options: &BannerOptions) -> String {
    let (on, off) = style_cells(options.style);
    let gap = " ".repeat(options.letter_spacing * on.len() / 2 + options.letter_spacing);

    // For each of the 5 rows, build the row string across all glyphs.
    let chars: Vec<char> = text.chars().collect();

    let rows: Vec<String> = (0..5)
        .map(|row| {
            let mut line = String::new();
            for (ci, &ch) in chars.iter().enumerate() {
                let bitmap = get_glyph(ch);
                let bits = bitmap[row];
                // 5 bits, bit 4 = leftmost
                for bit_pos in (0..5).rev() {
                    if (bits >> bit_pos) & 1 == 1 {
                        line.push_str(on);
                    } else {
                        line.push_str(off);
                    }
                }
                // Add letter spacing gap (except after last character)
                if ci < chars.len() - 1 {
                    line.push_str(&gap);
                }
            }
            apply_color(&line, &options.color)
        })
        .collect();

    rows.join("\n")
}
