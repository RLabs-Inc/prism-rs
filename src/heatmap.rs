//! 2D colored heatmap primitive.
//!
//! Renders a matrix of `f64` values as a grid of colored cells using one
//! of several perceptually-organized colormaps. Useful for cross-axis
//! comparisons:
//!
//! * chipset × frame-variant filter behavior maps
//! * channel × time-bucket activity heatmaps
//! * STA × IE-tag-position fingerprint matrices
//! * target × mutation-strategy crash-rate maps
//!
//! When ANSI output is disabled (pipe / non-TTY), cells fall back to ASCII
//! intensity characters so the structure remains legible.

use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

use crate::style::{s, Color};
use crate::writer::ansi_enabled;

/// Built-in color mappings. All are normalized to t ∈ [0, 1].
#[derive(Debug, Clone, Copy, Default)]
pub enum Colormap {
    /// Viridis — perceptually uniform, dark blue → green → yellow. Default.
    #[default]
    Viridis,
    /// Plasma — perceptually uniform, dark purple → red → yellow.
    Plasma,
    /// Diverging — red (low) → white (mid) → blue (high).
    /// Good for ratio/signed data centered on 0.5.
    Diverging,
    /// Grayscale.
    Grayscale,
    /// SuccessHot — dark green (low) → bright green (high): `(0,40,0)` → `(0,255,80)`.
    /// Built for the fuzzer lab's "a break is a success" palette: green glows
    /// where breaks cluster. Monotonically increasing luma, so darker = quieter.
    SuccessHot,
}

/// Heatmap rendering options.
#[derive(Debug, Clone)]
pub struct HeatmapOptions {
    /// Cell width in display columns. Default: 2.
    pub cell_width: usize,
    /// Row labels (one per data row). When empty, no row labels are drawn.
    pub row_labels: Vec<String>,
    /// Column labels (one per data column). When empty, no header row is drawn.
    pub col_labels: Vec<String>,
    /// Color mapping.
    pub colormap: Colormap,
    /// Minimum value (for normalization). Values below clamp to this.
    pub value_min: f64,
    /// Maximum value (for normalization). Values above clamp to this.
    pub value_max: f64,
    /// Overlay each cell's value as text. Cells should be ≥3 wide for legibility.
    pub show_values: bool,
    /// Apply log10 transform before normalization (useful for skewed data).
    /// Values ≤ 0 are treated as `value_min` in log mode.
    pub log_scale: bool,
    /// Maximum width for row labels (truncated if longer). Default: 12.
    pub row_label_width: usize,
}

impl Default for HeatmapOptions {
    fn default() -> Self {
        Self {
            cell_width: 2,
            row_labels: Vec::new(),
            col_labels: Vec::new(),
            colormap: Colormap::Viridis,
            value_min: 0.0,
            value_max: 1.0,
            show_values: false,
            log_scale: false,
            row_label_width: 12,
        }
    }
}

/// Render a 2D matrix as a colored heatmap. Returns one `String` per row.
///
/// `values[r][c]` is normalized to `[value_min, value_max]` and colored
/// according to the chosen colormap. When colors aren't enabled (non-TTY
/// output), cells fall back to ASCII intensity characters.
///
/// # Example
/// ```
/// use prism::heatmap::{heatmap, Colormap, HeatmapOptions};
///
/// let data = vec![
///     vec![0.1, 0.5, 0.9],
///     vec![0.3, 0.7, 0.2],
/// ];
/// let lines = heatmap(&data, &HeatmapOptions {
///     row_labels: vec!["row A".into(), "row B".into()],
///     col_labels: vec!["x".into(), "y".into(), "z".into()],
///     colormap: Colormap::Viridis,
///     value_min: 0.0,
///     value_max: 1.0,
///     ..Default::default()
/// });
/// assert_eq!(lines.len(), 3); // header + 2 data rows
/// ```
pub fn heatmap(values: &[Vec<f64>], opts: &HeatmapOptions) -> Vec<String> {
    if values.is_empty() {
        return Vec::new();
    }
    let n_rows = values.len();
    let n_cols = values.iter().map(Vec::len).max().unwrap_or(0);
    if n_cols == 0 {
        return Vec::new();
    }

    let cell_w = opts.cell_width.max(1);
    let row_label_w = if opts.row_labels.is_empty() {
        0
    } else {
        opts.row_label_width.max(1)
    };

    let mut out = Vec::with_capacity(n_rows + 1);

    // Column header row (if labels provided).
    if !opts.col_labels.is_empty() {
        let mut line = String::new();
        if row_label_w > 0 {
            line.push_str(&" ".repeat(row_label_w + 1));
        }
        for col in 0..n_cols {
            let label = opts
                .col_labels
                .get(col)
                .map(String::as_str)
                .unwrap_or("");
            line.push_str(&pad_or_truncate(label, cell_w));
        }
        out.push(line);
    }

    // Data rows.
    for (row_idx, row) in values.iter().enumerate() {
        let mut line = String::new();
        if row_label_w > 0 {
            let label = opts
                .row_labels
                .get(row_idx)
                .map(String::as_str)
                .unwrap_or("");
            line.push_str(&pad_or_truncate(label, row_label_w));
            line.push(' ');
        }
        for col in 0..n_cols {
            let raw = row.get(col).copied().unwrap_or(0.0);
            let t = normalize(raw, opts);
            line.push_str(&render_cell(t, raw, cell_w, opts));
        }
        out.push(line);
    }

    out
}

// ──────────────────────────────────────────────────────────────────────────
//  Normalization
// ──────────────────────────────────────────────────────────────────────────

fn normalize(value: f64, opts: &HeatmapOptions) -> f64 {
    let (v, lo, hi) = if opts.log_scale {
        let to_log = |x: f64| {
            if x <= 0.0 {
                f64::NEG_INFINITY
            } else {
                x.log10()
            }
        };
        let lo = to_log(opts.value_min.max(f64::MIN_POSITIVE));
        let hi = to_log(opts.value_max.max(f64::MIN_POSITIVE));
        let v = to_log(value);
        (v, lo, hi)
    } else {
        (value, opts.value_min, opts.value_max)
    };

    if !lo.is_finite() || !hi.is_finite() || (hi - lo).abs() < f64::EPSILON {
        return 0.0;
    }
    if !v.is_finite() {
        return if v == f64::NEG_INFINITY { 0.0 } else { 1.0 };
    }
    ((v - lo) / (hi - lo)).clamp(0.0, 1.0)
}

// ──────────────────────────────────────────────────────────────────────────
//  Cell rendering
// ──────────────────────────────────────────────────────────────────────────

const INTENSITY_CHARS: &[u8] = b" .:-=+*#%@";

fn render_cell(t: f64, raw: f64, width: usize, opts: &HeatmapOptions) -> String {
    let content = if opts.show_values {
        let formatted = format_value(raw, width);
        pad_or_truncate(&formatted, width)
    } else {
        " ".repeat(width)
    };

    if !ansi_enabled() {
        let levels = INTENSITY_CHARS.len();
        let idx = (t * (levels - 1) as f64).round() as usize;
        let ch = INTENSITY_CHARS[idx.min(levels - 1)] as char;
        return ch.to_string().repeat(width);
    }

    let (r, g, b) = colormap_rgb(t, opts.colormap);
    let (fr, fg, fb) = contrasting_fg(r, g, b);
    s().bg_color(Color::Rgb(r, g, b))
        .fg(Color::Rgb(fr, fg, fb))
        .paint(&content)
}

fn format_value(value: f64, width: usize) -> String {
    if width >= 5 {
        if value.abs() >= 1000.0 {
            format!("{:.0}", value)
        } else if value.abs() >= 10.0 {
            format!("{:.1}", value)
        } else {
            format!("{:.2}", value)
        }
    } else if width >= 3 {
        format!("{:.1}", value)
    } else {
        format!("{:.0}", value.round())
    }
}

// ──────────────────────────────────────────────────────────────────────────
//  Colormaps
// ──────────────────────────────────────────────────────────────────────────

fn colormap_rgb(t: f64, cmap: Colormap) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    match cmap {
        Colormap::Viridis => viridis(t),
        Colormap::Plasma => plasma(t),
        Colormap::Diverging => diverging(t),
        Colormap::Grayscale => grayscale(t),
        Colormap::SuccessHot => success_hot(t),
    }
}

/// Approximate viridis (matplotlib-style perceptually uniform).
fn viridis(t: f64) -> (u8, u8, u8) {
    // Polynomial approximation, fits the matplotlib viridis lookup table well.
    let r = (0.267_004 + 1.286 * t - 5.143 * t * t + 7.143 * t.powi(3)
        - 2.286 * t.powi(4))
    .clamp(0.0, 1.0);
    let g = (0.004_874 + 1.572 * t - 0.857 * t * t).clamp(0.0, 1.0);
    let b = (0.329_415 + 1.000 * t - 5.143 * t * t + 5.714 * t.powi(3)
        - 2.000 * t.powi(4))
    .clamp(0.0, 1.0);
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

/// Approximate plasma (matplotlib-style perceptually uniform).
fn plasma(t: f64) -> (u8, u8, u8) {
    let r = (0.050_383 + 2.286 * t - 1.714 * t * t).clamp(0.0, 1.0);
    let g = (0.029_803 + 0.286 * t + 1.000 * t * t).clamp(0.0, 1.0);
    let b = (0.527_975 + 1.286 * t - 3.857 * t * t + 2.286 * t.powi(3))
        .clamp(0.0, 1.0);
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

/// Red → white → blue. Useful for signed/centered data (t=0.5 = neutral).
fn diverging(t: f64) -> (u8, u8, u8) {
    if t < 0.5 {
        let s_ = t * 2.0;
        let r = 1.0;
        let g = s_;
        let b = s_;
        ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    } else {
        let s_ = (t - 0.5) * 2.0;
        let r = 1.0 - s_;
        let g = 1.0 - s_;
        let b = 1.0;
        ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }
}

fn grayscale(t: f64) -> (u8, u8, u8) {
    let v = (t * 255.0) as u8;
    (v, v, v)
}

/// Dark green → bright green: `(0,40,0)` (quiet) → `(0,255,80)` (jackpot).
/// Red stays 0; green and blue both rise with `t`, so Rec. 601 luma increases
/// monotonically — brighter cells unambiguously mean more breaks.
fn success_hot(t: f64) -> (u8, u8, u8) {
    let g = (40.0 + 215.0 * t).round().clamp(0.0, 255.0) as u8;
    let b = (80.0 * t).round().clamp(0.0, 255.0) as u8;
    (0, g, b)
}

/// Black on light backgrounds, white on dark — Rec. 601 luma.
fn contrasting_fg(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let luma = 0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64;
    if luma > 128.0 {
        (0, 0, 0)
    } else {
        (255, 255, 255)
    }
}

// ──────────────────────────────────────────────────────────────────────────
//  Width-aware text utilities
// ──────────────────────────────────────────────────────────────────────────

fn pad_or_truncate(s: &str, width: usize) -> String {
    let w = UnicodeWidthStr::width(s);
    if w == width {
        return s.to_string();
    }
    if w > width {
        return truncate_to_cols(s, width);
    }
    let mut out = String::from(s);
    out.push_str(&" ".repeat(width - w));
    out
}

fn truncate_to_cols(s: &str, width: usize) -> String {
    let mut out = String::new();
    let mut acc = 0;
    for ch in s.chars() {
        let cw = UnicodeWidthChar::width(ch).unwrap_or(0);
        if acc + cw > width {
            break;
        }
        out.push(ch);
        acc += cw;
    }
    // Pad with spaces if truncation didn't fill exactly to width.
    if acc < width {
        out.push_str(&" ".repeat(width - acc));
    }
    out
}

// ──────────────────────────────────────────────────────────────────────────
//  Tests
// ──────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_yields_empty_output() {
        let out = heatmap(&[] as &[Vec<f64>], &HeatmapOptions::default());
        assert!(out.is_empty());
    }

    #[test]
    fn empty_rows_yield_empty_output() {
        let out = heatmap(&[Vec::<f64>::new()], &HeatmapOptions::default());
        assert!(out.is_empty());
    }

    #[test]
    fn single_cell_renders_one_line() {
        let out = heatmap(&[vec![0.5]], &HeatmapOptions::default());
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn n_rows_yields_n_lines_when_no_col_header() {
        let data = vec![vec![0.1, 0.2], vec![0.3, 0.4], vec![0.5, 0.6]];
        let out = heatmap(&data, &HeatmapOptions::default());
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn col_header_adds_one_line() {
        let data = vec![vec![0.1, 0.2]];
        let out = heatmap(
            &data,
            &HeatmapOptions {
                col_labels: vec!["x".into(), "y".into()],
                ..Default::default()
            },
        );
        assert_eq!(out.len(), 2); // header + 1 row
    }

    #[test]
    fn normalize_clamps_below_min() {
        let opts = HeatmapOptions {
            value_min: 0.0,
            value_max: 1.0,
            ..Default::default()
        };
        assert_eq!(normalize(-5.0, &opts), 0.0);
    }

    #[test]
    fn normalize_clamps_above_max() {
        let opts = HeatmapOptions {
            value_min: 0.0,
            value_max: 1.0,
            ..Default::default()
        };
        assert_eq!(normalize(5.0, &opts), 1.0);
    }

    #[test]
    fn normalize_midpoint() {
        let opts = HeatmapOptions {
            value_min: 0.0,
            value_max: 10.0,
            ..Default::default()
        };
        assert!((normalize(5.0, &opts) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn log_scale_handles_zero() {
        let opts = HeatmapOptions {
            value_min: 1.0,
            value_max: 1000.0,
            log_scale: true,
            ..Default::default()
        };
        // 0 should clamp to value_min (i.e., t=0 after log)
        assert_eq!(normalize(0.0, &opts), 0.0);
    }

    #[test]
    fn log_scale_maps_geometric_midpoint() {
        let opts = HeatmapOptions {
            value_min: 1.0,
            value_max: 100.0,
            log_scale: true,
            ..Default::default()
        };
        // 10 is the geometric midpoint of [1, 100], should map to ~0.5
        assert!((normalize(10.0, &opts) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn equal_min_max_is_safe() {
        let opts = HeatmapOptions {
            value_min: 5.0,
            value_max: 5.0,
            ..Default::default()
        };
        // Degenerate range — should not panic, should yield 0
        assert_eq!(normalize(5.0, &opts), 0.0);
    }

    #[test]
    fn pad_or_truncate_pads() {
        assert_eq!(pad_or_truncate("hi", 5), "hi   ");
    }

    #[test]
    fn pad_or_truncate_truncates() {
        assert_eq!(pad_or_truncate("hello", 3), "hel");
    }

    #[test]
    fn pad_or_truncate_exact() {
        assert_eq!(pad_or_truncate("hi", 2), "hi");
    }

    #[test]
    fn viridis_endpoints_distinct() {
        let lo = viridis(0.0);
        let hi = viridis(1.0);
        assert_ne!(lo, hi);
    }

    #[test]
    fn diverging_midpoint_is_white() {
        let (r, g, b) = diverging(0.5);
        assert_eq!((r, g, b), (255, 255, 255));
    }

    #[test]
    fn grayscale_endpoints() {
        assert_eq!(grayscale(0.0), (0, 0, 0));
        assert_eq!(grayscale(1.0), (255, 255, 255));
    }

    #[test]
    fn contrasting_fg_white_on_dark() {
        // Pure dark blue should get white text
        assert_eq!(contrasting_fg(20, 20, 80), (255, 255, 255));
    }

    #[test]
    fn contrasting_fg_black_on_light() {
        // Pure yellow should get black text
        assert_eq!(contrasting_fg(255, 240, 0), (0, 0, 0));
    }

    #[test]
    fn ragged_input_padded_to_max_cols() {
        // Row 0 has 3 cols, row 1 has 1. Renderer should pad row 1 to 3 cells.
        let data = vec![vec![0.1, 0.5, 0.9], vec![0.5]];
        let out = heatmap(
            &data,
            &HeatmapOptions {
                cell_width: 2,
                ..Default::default()
            },
        );
        assert_eq!(out.len(), 2);
        // Each rendered line should describe 3 cells (even if values are 0 for missing)
    }

    // ── SuccessHot colormap (fuzzer-lab "a break is a success" palette) ──────

    fn luma(rgb: (u8, u8, u8)) -> f64 {
        0.299 * rgb.0 as f64 + 0.587 * rgb.1 as f64 + 0.114 * rgb.2 as f64
    }

    #[test]
    fn success_hot_low_endpoint_is_dark_green() {
        assert_eq!(success_hot(0.0), (0, 40, 0));
    }

    #[test]
    fn success_hot_high_endpoint_is_bright_green() {
        assert_eq!(success_hot(1.0), (0, 255, 80));
    }

    #[test]
    fn success_hot_red_channel_always_zero() {
        for i in 0..=10 {
            let (r, _, _) = success_hot(i as f64 / 10.0);
            assert_eq!(r, 0, "red must stay 0 at t={}", i as f64 / 10.0);
        }
    }

    #[test]
    fn success_hot_midpoint_between_endpoints() {
        let (r, g, b) = success_hot(0.5);
        assert_eq!(r, 0);
        // 40 + 215*0.5 = 147.5 → 148; 80*0.5 = 40
        assert_eq!((g, b), (148, 40));
    }

    #[test]
    fn success_hot_luma_monotonic_increasing() {
        let mut prev = luma(success_hot(0.0));
        for i in 1..=20 {
            let cur = luma(success_hot(i as f64 / 20.0));
            assert!(cur > prev, "luma must rise at step {i}: {cur} !> {prev}");
            prev = cur;
        }
    }

    #[test]
    fn success_hot_through_colormap_rgb_dispatch() {
        for i in 0..=10 {
            let t = i as f64 / 10.0;
            assert_eq!(colormap_rgb(t, Colormap::SuccessHot), success_hot(t));
        }
        // colormap_rgb clamps t before dispatch
        assert_eq!(colormap_rgb(-1.0, Colormap::SuccessHot), success_hot(0.0));
        assert_eq!(colormap_rgb(2.0, Colormap::SuccessHot), success_hot(1.0));
    }

    #[test]
    fn success_hot_heatmap_linear_row_count() {
        let data = vec![vec![0.0, 0.5, 1.0], vec![0.2, 0.8, 0.4]];
        let out = heatmap(
            &data,
            &HeatmapOptions {
                colormap: Colormap::SuccessHot,
                ..Default::default()
            },
        );
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn success_hot_heatmap_log_scale_no_panic() {
        // Break counts are skewed → log scale is the realistic notebook path.
        let data = vec![vec![0.0, 1.0, 100.0], vec![3.0, 0.0, 9.0]];
        let out = heatmap(
            &data,
            &HeatmapOptions {
                colormap: Colormap::SuccessHot,
                value_min: 1.0,
                value_max: 100.0,
                log_scale: true,
                ..Default::default()
            },
        );
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn success_hot_heatmap_ragged_padded() {
        let data = vec![vec![0.1, 0.5, 0.9], vec![0.5]];
        let out = heatmap(
            &data,
            &HeatmapOptions {
                colormap: Colormap::SuccessHot,
                cell_width: 2,
                ..Default::default()
            },
        );
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn success_hot_heatmap_with_labels_adds_header() {
        let data = vec![vec![0.0, 1.0]];
        let out = heatmap(
            &data,
            &HeatmapOptions {
                colormap: Colormap::SuccessHot,
                row_labels: vec!["assoc".into()],
                col_labels: vec!["bitflip".into(), "havoc".into()],
                ..Default::default()
            },
        );
        assert_eq!(out.len(), 2); // header + 1 data row
    }

    #[test]
    fn ascii_fallback_independent_of_colormap() {
        // The non-TTY ASCII fallback depends only on the normalized value `t`,
        // never on the colormap — so a new colormap can't perturb piped output.
        if ansi_enabled() {
            return; // colored path under FORCE_COLOR; fallback assertion N/A
        }
        let data = vec![vec![0.0, 0.5, 1.0]];
        let hot = heatmap(
            &data,
            &HeatmapOptions {
                colormap: Colormap::SuccessHot,
                ..Default::default()
            },
        );
        let viridis = heatmap(
            &data,
            &HeatmapOptions {
                colormap: Colormap::Viridis,
                ..Default::default()
            },
        );
        assert_eq!(hot, viridis);
    }

    #[test]
    fn ascii_fallback_intensity_endpoints() {
        if ansi_enabled() {
            return;
        }
        // t=0 → ' ', t=1 → '@' (first/last of INTENSITY_CHARS), cell_width 1.
        let data = vec![vec![0.0, 1.0]];
        let out = heatmap(
            &data,
            &HeatmapOptions {
                colormap: Colormap::SuccessHot,
                cell_width: 1,
                ..Default::default()
            },
        );
        assert_eq!(out, vec![" @".to_string()]);
    }
}
