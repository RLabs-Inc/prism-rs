//! Bitmap-to-braille art converter.
//!
//! Converts 2D pixel grids into Unicode braille characters (U+2800–U+28FF).
//! Each braille character encodes a 2×4 dot grid, giving high-resolution
//! pixel art in monospace terminals.
//!
//! # Braille dot numbering
//!
//! ```text
//! ┌───┬───┐
//! │ 1 │ 4 │  row 0
//! │ 2 │ 5 │  row 1
//! │ 3 │ 6 │  row 2
//! │ 7 │ 8 │  row 3
//! └───┴───┘
//!  col0 col1
//! ```
//!
//! Codepoint = U+2800 + (d1×1 + d2×2 + d3×4 + d4×8 + d5×16 + d6×32 + d7×64 + d8×128)

/// Convert a 2D boolean grid to a braille art string.
///
/// `grid[y][x]` = true means the pixel is "on" (filled).
/// The grid is processed in 2×4 blocks, each becoming one braille character.
/// Returns a multi-line string (no trailing newline).
pub fn grid_to_braille(grid: &[Vec<bool>]) -> String {
    if grid.is_empty() {
        return String::new();
    }

    let height = grid.len();
    let width = grid.iter().map(|row| row.len()).max().unwrap_or(0);
    if width == 0 {
        return String::new();
    }

    let char_rows = height.div_ceil(4);
    let char_cols = width.div_ceil(2);

    let pixel = |x: usize, y: usize| -> bool {
        if y < grid.len() && x < grid[y].len() {
            grid[y][x]
        } else {
            false
        }
    };

    let mut lines: Vec<String> = Vec::with_capacity(char_rows);

    for cr in 0..char_rows {
        let mut line = String::with_capacity(char_cols * 3); // UTF-8 braille = 3 bytes each
        for cc in 0..char_cols {
            let px = cc * 2;
            let py = cr * 4;

            let mut val: u32 = 0x2800;
            if pixel(px, py)     { val |= 0x01; } // dot 1
            if pixel(px, py + 1) { val |= 0x02; } // dot 2
            if pixel(px, py + 2) { val |= 0x04; } // dot 3
            if pixel(px + 1, py)     { val |= 0x08; } // dot 4
            if pixel(px + 1, py + 1) { val |= 0x10; } // dot 5
            if pixel(px + 1, py + 2) { val |= 0x20; } // dot 6
            if pixel(px, py + 3)     { val |= 0x40; } // dot 7
            if pixel(px + 1, py + 3) { val |= 0x80; } // dot 8

            if let Some(ch) = char::from_u32(val) {
                line.push(ch);
            }
        }
        // Trim trailing blank braille chars (U+2800)
        let trimmed = line.trim_end_matches('\u{2800}');
        lines.push(trimmed.to_string());
    }

    // Remove trailing empty lines
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }

    lines.join("\n")
}

/// Convert ASCII art to braille. Each `#` or `█` pixel is "on", everything else is "off".
///
/// This is the easiest way to create braille art: design with `#` and spaces,
/// then convert to high-resolution braille rendering.
///
/// ```rust
/// use prism::braille::ascii_to_braille;
///
/// let art = &[
///     "  ####  ",
///     " #    # ",
///     " # ## # ",
///     " #    # ",
///     "  ####  ",
/// ];
/// let braille = ascii_to_braille(art);
/// // Each line is now rendered in 4× vertical resolution
/// ```
pub fn ascii_to_braille(art: &[&str]) -> String {
    let grid: Vec<Vec<bool>> = art
        .iter()
        .map(|line| {
            line.chars()
                .map(|ch| ch == '#' || ch == '█' || ch == '▓' || ch == '▒')
                .collect()
        })
        .collect();
    grid_to_braille(&grid)
}

/// Convert ASCII art to braille with a custom "on" predicate.
///
/// Useful when your art uses different fill characters.
pub fn ascii_to_braille_with<F>(art: &[&str], is_on: F) -> String
where
    F: Fn(char) -> bool,
{
    let grid: Vec<Vec<bool>> = art
        .iter()
        .map(|line| line.chars().map(&is_on).collect())
        .collect();
    grid_to_braille(&grid)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_grid() {
        assert_eq!(grid_to_braille(&[]), "");
        assert_eq!(ascii_to_braille(&[]), "");
    }

    #[test]
    fn test_single_pixel() {
        // One pixel at (0,0) → dot 1 → U+2800 + 1 = U+2801 = ⠁
        let grid = vec![vec![true]];
        assert_eq!(grid_to_braille(&grid), "⠁");
    }

    #[test]
    fn test_full_block() {
        // All 8 dots on → U+2800 + 255 = U+28FF = ⣿
        let grid = vec![
            vec![true, true],
            vec![true, true],
            vec![true, true],
            vec![true, true],
        ];
        assert_eq!(grid_to_braille(&grid), "⣿");
    }

    #[test]
    fn test_left_column_only() {
        // Left column all on: dots 1,2,3,7 → 1+2+4+64 = 71 → U+2847 = ⡇
        let grid = vec![
            vec![true, false],
            vec![true, false],
            vec![true, false],
            vec![true, false],
        ];
        assert_eq!(grid_to_braille(&grid), "⡇");
    }

    #[test]
    fn test_right_column_only() {
        // Right column all on: dots 4,5,6,8 → 8+16+32+128 = 184 → U+28B8 = ⢸
        let grid = vec![
            vec![false, true],
            vec![false, true],
            vec![false, true],
            vec![false, true],
        ];
        assert_eq!(grid_to_braille(&grid), "⢸");
    }

    #[test]
    fn test_ascii_to_braille_simple() {
        let art = &[
            "##",
            "##",
            "##",
            "##",
        ];
        assert_eq!(ascii_to_braille(art), "⣿");
    }

    #[test]
    fn test_ascii_to_braille_multichar() {
        let art = &[
            "####",
            "####",
            "####",
            "####",
        ];
        // 4 wide = 2 braille chars, all full
        assert_eq!(ascii_to_braille(art), "⣿⣿");
    }

    #[test]
    fn test_two_braille_rows() {
        let art = &[
            "##",
            "##",
            "##",
            "##",
            "##",
            "##",
            "##",
            "##",
        ];
        // 8 rows = 2 braille rows, each full
        assert_eq!(ascii_to_braille(art), "⣿\n⣿");
    }

    #[test]
    fn test_trailing_blanks_trimmed() {
        let art = &[
            "##    ",
            "##    ",
            "##    ",
            "##    ",
        ];
        // The trailing spaces should produce blank braille chars that get trimmed
        assert_eq!(ascii_to_braille(art), "⣿");
    }

    #[test]
    fn test_ascii_with_custom_predicate() {
        let art = &["XX", "XX", "XX", "XX"];
        let result = ascii_to_braille_with(art, |ch| ch == 'X');
        assert_eq!(result, "⣿");
    }

    #[test]
    fn test_circle_shape() {
        let art = &[
            "  ####  ",
            " ##  ## ",
            "##    ##",
            "##    ##",
            "##    ##",
            "##    ##",
            " ##  ## ",
            "  ####  ",
        ];
        let braille = ascii_to_braille(art);
        // Should produce 4 chars wide × 2 rows
        let lines: Vec<&str> = braille.split('\n').collect();
        assert_eq!(lines.len(), 2);
        // Each line should be 4 braille chars
        assert_eq!(lines[0].chars().count(), 4);
    }
}
