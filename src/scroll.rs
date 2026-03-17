//! Scrollable view container — fixed header/footer with scrollable data rows.
//!
//! Pure render function: takes pre-formatted rows and handles all scroll chrome
//! (indicators, capacity calculation, visible range). Caller formats rows however
//! they want — ScrollView just manages the viewport.
//!
//! Also provides scroll math utilities (clamping, ensure-visible, paging) that
//! the caller's key handler can use.

use crate::style::s;

// ---------------------------------------------------------------------------
// Scroll math utilities
// ---------------------------------------------------------------------------

/// Clamp scroll offset so the viewport stays within bounds.
///
/// Returns the clamped offset. If `total <= visible`, returns 0.
pub fn clamp_scroll(offset: usize, total: usize, visible: usize) -> usize {
    if total <= visible {
        return 0;
    }
    offset.min(total - visible)
}

/// Adjust scroll offset to ensure `selected` row is visible.
///
/// If selected is above the viewport, scrolls up. If below, scrolls down.
/// Returns the new scroll offset.
pub fn ensure_visible(offset: usize, selected: usize, visible: usize) -> usize {
    if visible == 0 {
        return offset;
    }
    if selected < offset {
        // Selected is above viewport — scroll up to it
        selected
    } else if selected >= offset + visible {
        // Selected is below viewport — scroll down so it's the last visible row
        selected - visible + 1
    } else {
        // Already visible — no change
        offset
    }
}

/// Move scroll offset one page down. Returns new offset.
pub fn page_down(offset: usize, visible: usize, total: usize) -> usize {
    clamp_scroll(offset + visible, total, visible)
}

/// Move scroll offset one page up. Returns new offset.
pub fn page_up(offset: usize, visible: usize) -> usize {
    offset.saturating_sub(visible)
}

/// Calculate how many data rows fit given a total line budget and chrome lines.
///
/// Chrome = header lines + footer lines + 2 (above/below scroll indicators).
/// The indicators always occupy a line each to keep layout stable.
pub fn data_capacity(max_rows: usize, header_lines: usize, footer_lines: usize) -> usize {
    // above_indicator(1) + below_indicator(1) = 2
    max_rows.saturating_sub(header_lines + footer_lines + 2)
}

// ---------------------------------------------------------------------------
// ScrollView configuration
// ---------------------------------------------------------------------------

/// Configuration for rendering a scrollable view.
pub struct ScrollViewConfig<'a> {
    /// Pinned header lines (column labels, separator, etc.)
    /// These are always rendered at the top.
    pub header: &'a [String],

    /// All data rows (pre-formatted). Only the visible slice is rendered.
    pub rows: &'a [String],

    /// Pinned footer lines (hints, summaries, etc.)
    /// These are always rendered at the bottom.
    pub footer: &'a [String],

    /// Current scroll offset (0-based index of first visible row).
    pub scroll_offset: usize,

    /// Maximum total lines available for the entire view.
    pub max_rows: usize,

    /// Left indent for scroll indicators (default: 2).
    pub indent: usize,

    /// Message shown when `rows` is empty (e.g. "No APs discovered yet.").
    /// If None, shows nothing when empty.
    pub empty_message: Option<&'a str>,
}

impl<'a> Default for ScrollViewConfig<'a> {
    fn default() -> Self {
        Self {
            header: &[],
            rows: &[],
            footer: &[],
            scroll_offset: 0,
            max_rows: 24,
            indent: 2,
            empty_message: None,
        }
    }
}

/// Result of rendering a scroll view.
pub struct ScrollViewResult {
    /// The rendered lines.
    pub lines: Vec<String>,
    /// Number of data rows that fit in the viewport.
    pub capacity: usize,
    /// The range of row indices that are visible.
    pub visible_range: std::ops::Range<usize>,
    /// Number of items above the viewport.
    pub above: usize,
    /// Number of items below the viewport.
    pub below: usize,
}

// ---------------------------------------------------------------------------
// Main render function
// ---------------------------------------------------------------------------

/// Render a scrollable view with fixed header/footer and scroll indicators.
///
/// Layout (top to bottom):
/// ```text
/// ┌─ header lines (pinned) ──────────┐
/// │  ▲ N more above                  │  ← scroll indicator (always 1 line)
/// │  row 0                           │
/// │  row 1                           │  ← data rows (scroll_offset..end)
/// │  ...                             │
/// │  ▼ N more below                  │  ← scroll indicator (always 1 line)
/// └─ footer lines (pinned) ──────────┘
/// ```
///
/// Scroll indicators always occupy exactly 1 line each to keep the layout
/// height stable regardless of scroll position.
pub fn scroll_view(config: &ScrollViewConfig) -> ScrollViewResult {
    let mut lines = Vec::new();
    let indent = " ".repeat(config.indent);

    // Push header lines
    for line in config.header {
        lines.push(line.clone());
    }

    let capacity = data_capacity(config.max_rows, config.header.len(), config.footer.len());

    // Handle empty state
    if config.rows.is_empty() {
        if let Some(msg) = config.empty_message {
            lines.push(format!("{}{}", indent, s().dim().paint(msg)));
        }
        // Fill remaining space + footer
        for line in config.footer {
            lines.push(line.clone());
        }
        return ScrollViewResult {
            lines,
            capacity,
            visible_range: 0..0,
            above: 0,
            below: 0,
        };
    }

    let total = config.rows.len();

    // Clamp scroll offset
    let offset = clamp_scroll(config.scroll_offset, total, capacity);
    let end = (offset + capacity).min(total);
    let above = offset;
    let below = total.saturating_sub(end);

    // Above indicator (always 1 line)
    if above > 0 {
        lines.push(format!(
            "{}{} {} more above",
            indent,
            s().dim().paint("\u{25b2}"),
            s().dim().paint(&above.to_string()),
        ));
    } else {
        lines.push(String::new());
    }

    // Visible data rows
    for row in &config.rows[offset..end] {
        lines.push(row.clone());
    }

    // Below indicator (always 1 line)
    if below > 0 {
        lines.push(format!(
            "{}{} {} more below",
            indent,
            s().dim().paint("\u{25bc}"),
            s().dim().paint(&below.to_string()),
        ));
    } else {
        lines.push(String::new());
    }

    // Footer lines
    for line in config.footer {
        lines.push(line.clone());
    }

    ScrollViewResult {
        lines,
        capacity,
        visible_range: offset..end,
        above,
        below,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── Scroll math ──────────────────────────────────────────────────────

    #[test]
    fn test_clamp_scroll_within_bounds() {
        assert_eq!(clamp_scroll(0, 100, 20), 0);
        assert_eq!(clamp_scroll(50, 100, 20), 50);
        assert_eq!(clamp_scroll(80, 100, 20), 80);
    }

    #[test]
    fn test_clamp_scroll_past_end() {
        assert_eq!(clamp_scroll(90, 100, 20), 80);
        assert_eq!(clamp_scroll(200, 100, 20), 80);
    }

    #[test]
    fn test_clamp_scroll_all_fit() {
        assert_eq!(clamp_scroll(0, 10, 20), 0);
        assert_eq!(clamp_scroll(5, 10, 20), 0);
        assert_eq!(clamp_scroll(0, 10, 10), 0);
    }

    #[test]
    fn test_ensure_visible_already_visible() {
        assert_eq!(ensure_visible(10, 15, 20), 10);
        assert_eq!(ensure_visible(10, 10, 20), 10);
        assert_eq!(ensure_visible(10, 29, 20), 10);
    }

    #[test]
    fn test_ensure_visible_above() {
        assert_eq!(ensure_visible(10, 5, 20), 5);
        assert_eq!(ensure_visible(10, 0, 20), 0);
    }

    #[test]
    fn test_ensure_visible_below() {
        assert_eq!(ensure_visible(10, 30, 20), 11);
        assert_eq!(ensure_visible(10, 50, 20), 31);
    }

    #[test]
    fn test_page_down() {
        assert_eq!(page_down(0, 20, 100), 20);
        assert_eq!(page_down(70, 20, 100), 80);
        assert_eq!(page_down(85, 20, 100), 80); // clamped
    }

    #[test]
    fn test_page_up() {
        assert_eq!(page_up(20, 20), 0);
        assert_eq!(page_up(30, 20), 10);
        assert_eq!(page_up(5, 20), 0); // saturating
    }

    #[test]
    fn test_data_capacity() {
        // 30 max, 2 header, 3 footer, 2 indicators = 23 data rows
        assert_eq!(data_capacity(30, 2, 3), 23);
        // 10 max, 2 header, 3 footer, 2 indicators = 3 data rows
        assert_eq!(data_capacity(10, 2, 3), 3);
        // Edge: not enough room
        assert_eq!(data_capacity(4, 2, 3), 0);
    }

    // ── ScrollView rendering ──────────────────────────────────────────────

    #[test]
    fn test_scroll_view_all_fit() {
        let header = vec!["HEADER".to_string(), "-----".to_string()];
        let rows: Vec<String> = (0..5).map(|i| format!("row {}", i)).collect();
        let footer = vec!["hints".to_string()];

        let result = scroll_view(&ScrollViewConfig {
            header: &header,
            rows: &rows,
            footer: &footer,
            scroll_offset: 0,
            max_rows: 30,
            indent: 2,
            empty_message: None,
        });

        assert_eq!(result.above, 0);
        assert_eq!(result.below, 0);
        assert_eq!(result.visible_range, 0..5);
        // header(2) + above(1) + rows(5) + below(1) + footer(1) = 10
        assert_eq!(result.lines.len(), 10);
        assert_eq!(result.lines[0], "HEADER");
        assert_eq!(result.lines[1], "-----");
        assert_eq!(result.lines[2], ""); // above indicator (empty)
        assert_eq!(result.lines[3], "row 0");
        assert_eq!(result.lines[7], "row 4");
        assert_eq!(result.lines[8], ""); // below indicator (empty)
        assert_eq!(result.lines[9], "hints");
    }

    #[test]
    fn test_scroll_view_scrolled() {
        let header = vec!["H".to_string()];
        let rows: Vec<String> = (0..20).map(|i| format!("row {}", i)).collect();
        let footer = vec!["F".to_string()];

        let result = scroll_view(&ScrollViewConfig {
            header: &header,
            rows: &rows,
            footer: &footer,
            scroll_offset: 5,
            max_rows: 10,
            indent: 2,
            ..Default::default()
        });

        // capacity = 10 - 1(header) - 1(footer) - 2(indicators) = 6
        assert_eq!(result.capacity, 6);
        assert_eq!(result.above, 5);
        assert_eq!(result.visible_range, 5..11);
        assert_eq!(result.below, 9);
    }

    #[test]
    fn test_scroll_view_empty() {
        let header = vec!["H".to_string()];
        let rows: Vec<String> = vec![];
        let footer = vec!["F".to_string()];

        let result = scroll_view(&ScrollViewConfig {
            header: &header,
            rows: &rows,
            footer: &footer,
            scroll_offset: 0,
            max_rows: 20,
            indent: 2,
            empty_message: Some("Nothing here."),
        });

        assert_eq!(result.capacity, 16);
        assert_eq!(result.visible_range, 0..0);
        // Should contain the empty message
        assert!(result.lines.iter().any(|l| l.contains("Nothing here.")));
    }

    #[test]
    fn test_scroll_view_offset_clamped() {
        let header = vec![];
        let rows: Vec<String> = (0..5).map(|i| format!("r{}", i)).collect();
        let footer = vec![];

        let result = scroll_view(&ScrollViewConfig {
            header: &header,
            rows: &rows,
            footer: &footer,
            scroll_offset: 100, // way past end
            max_rows: 10,
            indent: 2,
            ..Default::default()
        });

        // Should clamp and show all 5 rows (they all fit)
        assert_eq!(result.above, 0);
        assert_eq!(result.below, 0);
        assert_eq!(result.visible_range, 0..5);
    }

    #[test]
    fn test_ensure_visible_zero_capacity() {
        assert_eq!(ensure_visible(0, 5, 0), 0);
    }

    #[test]
    fn test_clamp_scroll_zero_total() {
        assert_eq!(clamp_scroll(0, 0, 20), 0);
        assert_eq!(clamp_scroll(5, 0, 20), 0);
    }
}
