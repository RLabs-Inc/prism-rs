//! Flexbox-like layout for multi-line blocks.
//!
//! Flows pre-rendered blocks horizontally, wrapping to the next row when they
//! don't fit. Like `columns()` but for multi-line content blocks instead of
//! single-line items.
//!
//! Combined with `scroll_view()` for vertical overflow, this gives responsive
//! dashboard layouts with minimal code.
//!
//! # Example
//!
//! ```rust
//! use prism::flex::{flex, FlexBlock, FlexOptions, FlexAlign};
//!
//! let identity = FlexBlock::new(vec![
//!     "── Identity ──────".into(),
//!     "   SSID  RL-WiFi".into(),
//!     "  BSSID  7C:10:C9:03:10:E4".into(),
//! ], 30);
//!
//! let radio = FlexBlock::new(vec![
//!     "── Radio ─────────".into(),
//!     "  Channel  44".into(),
//!     "     Band  5 GHz".into(),
//! ], 30);
//!
//! let lines = flex(&[identity, radio], &FlexOptions {
//!     total_width: 80,
//!     align: FlexAlign::Stretch,
//!     ..Default::default()
//! });
//! // Wide: blocks side by side. Narrow: stacked vertically.
//! ```

use crate::ansi::measure_width;
use crate::text::pad;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A pre-rendered content block with a known width.
pub struct FlexBlock {
    /// Pre-rendered lines of content.
    pub lines: Vec<String>,
    /// The width of this block in display columns.
    /// Used for layout math — lines should fit within this width.
    pub width: usize,
}

impl FlexBlock {
    /// Create a block from pre-rendered lines and a declared width.
    pub fn new(lines: Vec<String>, width: usize) -> Self {
        Self { lines, width }
    }

    /// Create a block, measuring the actual maximum line width.
    pub fn measured(lines: Vec<String>) -> Self {
        let width = lines.iter().map(|l| measure_width(l)).max().unwrap_or(0);
        Self { lines, width }
    }
}

/// How to distribute blocks horizontally within a row.
#[derive(Debug, Clone, Copy, Default)]
pub enum FlexAlign {
    /// Pack blocks to the left with fixed gap.
    #[default]
    Start,
    /// Center the group of blocks.
    Center,
    /// First block at left edge, last at right, equal gaps between.
    SpaceBetween,
    /// Equal space between blocks and at edges.
    SpaceEvenly,
    /// Blocks expand equally to fill the full width.
    /// Each block's lines are right-padded to the stretched width.
    Stretch,
}

/// Options for flex layout.
pub struct FlexOptions {
    /// Minimum horizontal gap between blocks (default 3).
    /// For `Start` and `Center`, this is the exact gap.
    /// For `SpaceBetween`/`SpaceEvenly`, this is the minimum gap.
    /// For `Stretch`, this is the gap between stretched blocks.
    pub gap: usize,
    /// Blank lines between rows of blocks (default 1).
    pub row_gap: usize,
    /// Total available width (default 80).
    pub total_width: usize,
    /// Left indent for all content (default 0).
    pub indent: usize,
    /// Minimum block width before wrapping to fewer columns (default 25).
    /// When blocks would be narrower than this (in Stretch mode),
    /// the layout uses fewer columns.
    pub min_block_width: usize,
    /// How to distribute blocks horizontally.
    pub align: FlexAlign,
}

impl Default for FlexOptions {
    fn default() -> Self {
        Self {
            gap: 3,
            row_gap: 1,
            total_width: 80,
            indent: 0,
            min_block_width: 25,
            align: FlexAlign::Start,
        }
    }
}

// ---------------------------------------------------------------------------
// Main layout function
// ---------------------------------------------------------------------------

/// Flow blocks horizontally, wrapping to new rows as needed.
///
/// Returns rendered lines ready for display or `scroll_view()`.
pub fn flex(blocks: &[FlexBlock], options: &FlexOptions) -> Vec<String> {
    if blocks.is_empty() {
        return Vec::new();
    }

    let available = options.total_width.saturating_sub(options.indent);
    let indent_str = " ".repeat(options.indent);

    // Determine how many blocks fit per row
    let cols_per_row = compute_columns(blocks, available, options);

    let mut output: Vec<String> = Vec::new();
    let mut row_start = 0;

    while row_start < blocks.len() {
        let row_end = (row_start + cols_per_row).min(blocks.len());
        let row_blocks = &blocks[row_start..row_end];

        // Add row gap between rows (not before the first)
        if row_start > 0 {
            for _ in 0..options.row_gap {
                output.push(String::new());
            }
        }

        // Compute per-block widths and gaps for this row
        let (block_widths, gaps) = compute_row_layout(row_blocks, available, cols_per_row, options);

        // Find the tallest block in this row
        let max_height = row_blocks.iter().map(|b| b.lines.len()).max().unwrap_or(0);

        // Merge blocks line by line
        for line_idx in 0..max_height {
            let mut line = indent_str.clone();

            for (bi, block) in row_blocks.iter().enumerate() {
                let bw = block_widths[bi];

                // Add gap/padding before this block
                // gaps[0] is leading space (used by Center/SpaceEvenly)
                // gaps[1..] is space between blocks
                if gaps[bi] > 0 {
                    line.push_str(&" ".repeat(gaps[bi]));
                }

                // Get this line from the block (or empty if block is shorter)
                let content = block.lines.get(line_idx).map(|s| s.as_str()).unwrap_or("");

                // Pad to block width
                line.push_str(&pad(content, bw, "left"));
            }

            output.push(line);
        }

        row_start = row_end;
    }

    output
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Determine how many blocks fit per row.
fn compute_columns(blocks: &[FlexBlock], available: usize, options: &FlexOptions) -> usize {
    match options.align {
        FlexAlign::Stretch => {
            // In stretch mode, check that the first N blocks fit at their
            // natural widths (content must not overflow). Also respect min_block_width.
            let max_cols = blocks.len();
            for n in (1..=max_cols).rev() {
                let total_gaps = if n > 1 { (n - 1) * options.gap } else { 0 };
                // Check: do the first N blocks' natural widths fit?
                let natural_sum: usize = blocks.iter().take(n).map(|b| b.width).sum();
                if natural_sum + total_gaps <= available {
                    // Also check min_block_width
                    let per_block = available.saturating_sub(total_gaps) / n;
                    if per_block >= options.min_block_width {
                        return n;
                    }
                }
            }
            1
        }
        _ => {
            // For non-stretch modes, use actual block widths
            let max_width = blocks.iter().map(|b| b.width).max().unwrap_or(0);
            if max_width == 0 {
                return 1;
            }
            let fits = ((available + options.gap) / (max_width + options.gap)).max(1);
            fits.min(blocks.len())
        }
    }
}

/// Compute the actual width of each block and the gap before each block in a row.
fn compute_row_layout(
    row_blocks: &[FlexBlock],
    available: usize,
    _cols_per_row: usize,
    options: &FlexOptions,
) -> (Vec<usize>, Vec<usize>) {
    let n = row_blocks.len();
    if n == 0 {
        return (Vec::new(), Vec::new());
    }

    match options.align {
        FlexAlign::Stretch => {
            // Start with each block's natural width, then distribute
            // remaining space equally. No block gets narrower than its content.
            let total_gaps = if n > 1 { (n - 1) * options.gap } else { 0 };
            let natural: Vec<usize> = row_blocks.iter().map(|b| b.width).collect();
            let natural_total: usize = natural.iter().sum::<usize>() + total_gaps;

            let widths = if natural_total >= available {
                // Content already fills or exceeds available — use natural widths
                natural
            } else {
                // Distribute remaining space equally among all blocks
                let remaining = available - natural_total;
                let extra_per = remaining / n;
                let mut leftover = remaining - extra_per * n;
                natural.iter().map(|&w| {
                    let bonus = if leftover > 0 { leftover -= 1; 1 } else { 0 };
                    w + extra_per + bonus
                }).collect()
            };

            let mut gaps = vec![0; n];
            for i in 1..n {
                gaps[i] = options.gap;
            }
            (widths, gaps)
        }
        FlexAlign::Start => {
            let widths: Vec<usize> = row_blocks.iter().map(|b| b.width).collect();
            let mut gaps = vec![0; n];
            for i in 1..n {
                gaps[i] = options.gap;
            }
            (widths, gaps)
        }
        FlexAlign::Center => {
            let widths: Vec<usize> = row_blocks.iter().map(|b| b.width).collect();
            let used: usize = widths.iter().sum::<usize>() + if n > 1 { (n - 1) * options.gap } else { 0 };
            let left_pad = available.saturating_sub(used) / 2;
            let mut gaps = vec![0; n];
            gaps[0] = left_pad;
            for i in 1..n {
                gaps[i] = options.gap;
            }
            (widths, gaps)
        }
        FlexAlign::SpaceBetween => {
            let widths: Vec<usize> = row_blocks.iter().map(|b| b.width).collect();
            let used: usize = widths.iter().sum();
            let remaining = available.saturating_sub(used);
            let mut gaps = vec![0; n];
            if n > 1 {
                let gap = remaining / (n - 1);
                for i in 1..n {
                    gaps[i] = gap.max(options.gap);
                }
            }
            (widths, gaps)
        }
        FlexAlign::SpaceEvenly => {
            let widths: Vec<usize> = row_blocks.iter().map(|b| b.width).collect();
            let used: usize = widths.iter().sum();
            let remaining = available.saturating_sub(used);
            let slot_count = n + 1; // spaces: before first, between each, after last
            let gap = (remaining / slot_count).max(options.gap);
            let mut gaps = vec![0; n];
            for i in 0..n {
                gaps[i] = gap;
            }
            (widths, gaps)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flex_single_block() {
        let b = FlexBlock::new(vec!["hello".into(), "world".into()], 10);
        let lines = flex(&[b], &FlexOptions {
            total_width: 40,
            ..Default::default()
        });
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("hello"));
        assert!(lines[1].contains("world"));
    }

    #[test]
    fn test_flex_two_blocks_side_by_side() {
        let a = FlexBlock::new(vec!["AAA".into(), "aaa".into()], 10);
        let b = FlexBlock::new(vec!["BBB".into(), "bbb".into()], 10);
        let lines = flex(&[a, b], &FlexOptions {
            total_width: 30,
            gap: 3,
            ..Default::default()
        });
        // Should fit side by side: 10 + 3 + 10 = 23 <= 30
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("AAA"));
        assert!(lines[0].contains("BBB"));
    }

    #[test]
    fn test_flex_wraps_when_narrow() {
        let a = FlexBlock::new(vec!["AAA".into()], 20);
        let b = FlexBlock::new(vec!["BBB".into()], 20);
        let lines = flex(&[a, b], &FlexOptions {
            total_width: 30, // 20 + 3 + 20 = 43 > 30, won't fit
            gap: 3,
            ..Default::default()
        });
        // Should wrap: block A on row 1, block B on row 2 with row_gap between
        assert!(lines.len() >= 2);
        assert!(lines[0].contains("AAA"));
        // row_gap=1 means blank line between
        assert!(lines.iter().any(|l| l.contains("BBB")));
    }

    #[test]
    fn test_flex_stretch_equal_widths() {
        let a = FlexBlock::new(vec!["A".into()], 10);
        let b = FlexBlock::new(vec!["B".into()], 10);
        let lines = flex(&[a, b], &FlexOptions {
            total_width: 60,
            gap: 4,
            align: FlexAlign::Stretch,
            min_block_width: 20,
            ..Default::default()
        });
        // Stretched: (60 - 4) / 2 = 28 per block
        assert_eq!(lines.len(), 1);
        let w = measure_width(&lines[0]);
        assert_eq!(w, 60); // should fill the full width
    }

    #[test]
    fn test_flex_stretch_wraps_when_too_narrow() {
        let a = FlexBlock::new(vec!["A".into()], 10);
        let b = FlexBlock::new(vec!["B".into()], 10);
        let lines = flex(&[a, b], &FlexOptions {
            total_width: 40,
            gap: 4,
            align: FlexAlign::Stretch,
            min_block_width: 30, // can't fit 2 blocks at 30 each
            ..Default::default()
        });
        // Should wrap to single column
        assert!(lines.len() >= 2); // at least 2 lines (1 + row_gap + 1)
    }

    #[test]
    fn test_flex_uneven_heights() {
        let a = FlexBlock::new(vec!["A1".into(), "A2".into(), "A3".into()], 10);
        let b = FlexBlock::new(vec!["B1".into()], 10);
        let lines = flex(&[a, b], &FlexOptions {
            total_width: 30,
            gap: 3,
            ..Default::default()
        });
        // Should have 3 lines (tallest block), B padded with empty space
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("A1"));
        assert!(lines[0].contains("B1"));
        assert!(lines[2].contains("A3"));
    }

    #[test]
    fn test_flex_empty() {
        let lines = flex(&[], &FlexOptions::default());
        assert!(lines.is_empty());
    }

    #[test]
    fn test_flex_with_indent() {
        let a = FlexBlock::new(vec!["hello".into()], 10);
        let lines = flex(&[a], &FlexOptions {
            indent: 4,
            total_width: 40,
            ..Default::default()
        });
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("    ")); // 4-space indent
    }

    #[test]
    fn test_flex_center_alignment() {
        let a = FlexBlock::new(vec!["X".into()], 5);
        let b = FlexBlock::new(vec!["Y".into()], 5);
        let lines = flex(&[a, b], &FlexOptions {
            total_width: 40,
            gap: 2,
            align: FlexAlign::Center,
            ..Default::default()
        });
        assert_eq!(lines.len(), 1);
        // Total used: 5 + 2 + 5 = 12. Centering in 40: left_pad = 14
        assert!(lines[0].starts_with("              ")); // 14 spaces
    }

    #[test]
    fn test_flex_four_blocks_2x2() {
        let blocks: Vec<FlexBlock> = (0..4)
            .map(|i| FlexBlock::new(vec![format!("Block{}", i)], 15))
            .collect();
        let lines = flex(&blocks, &FlexOptions {
            total_width: 40, // 15 + 3 + 15 = 33 fits 2, not 3
            gap: 3,
            row_gap: 1,
            ..Default::default()
        });
        // 4 blocks, 2 per row = 2 rows with 1 blank line between
        // Row 1: Block0 Block1 (1 line)
        // gap: 1 blank line
        // Row 2: Block2 Block3 (1 line)
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("Block0"));
        assert!(lines[0].contains("Block1"));
        assert_eq!(lines[1], ""); // row gap
        assert!(lines[2].contains("Block2"));
        assert!(lines[2].contains("Block3"));
    }

    #[test]
    fn test_flex_measured_block() {
        let b = FlexBlock::measured(vec![
            "short".into(),
            "a longer line here".into(),
            "mid".into(),
        ]);
        assert_eq!(b.width, 18); // "a longer line here"
    }

    #[test]
    fn test_flex_space_between() {
        let a = FlexBlock::new(vec!["L".into()], 5);
        let b = FlexBlock::new(vec!["R".into()], 5);
        let lines = flex(&[a, b], &FlexOptions {
            total_width: 40,
            gap: 2,
            align: FlexAlign::SpaceBetween,
            ..Default::default()
        });
        assert_eq!(lines.len(), 1);
        // SpaceBetween: 40 - 5 - 5 = 30 gap
        let w = measure_width(&lines[0]);
        assert_eq!(w, 40); // L padded to 5, gap 30, R padded to 5
    }
}
