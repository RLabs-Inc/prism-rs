use crate::ansi::measure_width;
use crate::frame::BorderStyle;
use crate::style::s;
use crate::text::truncate;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

pub struct Column {
    pub key: String,
    pub label: Option<String>,
    pub align: Align,
    pub width: Option<usize>,
    pub min_width: Option<usize>,
    pub max_width: Option<usize>,
}

impl Default for Column {
    fn default() -> Self {
        Self {
            key: String::new(),
            label: None,
            align: Align::Left,
            width: None,
            min_width: None,
            max_width: None,
        }
    }
}

pub struct TableOptions {
    pub columns: Option<Vec<Column>>,
    pub border: BorderStyle,
    pub max_width: usize,
    pub compact: bool,
}

impl Default for TableOptions {
    fn default() -> Self {
        Self {
            columns: None,
            border: BorderStyle::Single,
            max_width: 80,
            compact: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Pad/align text within a fixed visible column width.
fn align_text(text: &str, width: usize, align: Align) -> String {
    let vis = measure_width(text);
    if vis >= width {
        return text.to_string();
    }
    let diff = width - vis;
    match align {
        Align::Left => format!("{}{}", text, " ".repeat(diff)),
        Align::Right => format!("{}{}", " ".repeat(diff), text),
        Align::Center => {
            let left = diff / 2;
            let right = diff - left;
            format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
        }
    }
}

// ---------------------------------------------------------------------------
// Main table() function
// ---------------------------------------------------------------------------

/// Render a table from a slice of rows, where each row is a slice of (key, value) pairs.
///
/// Returns an empty string if `data` is empty.
pub fn table(data: &[Vec<(&str, &str)>], opts: &TableOptions) -> String {
    if data.is_empty() {
        return String::new();
    }

    // --- 1. Determine columns ---
    // If caller specified columns, use them; otherwise collect unique keys from data.
    let columns: Vec<Column> = if let Some(cols) = &opts.columns {
        cols.iter()
            .map(|c| Column {
                key: c.key.clone(),
                label: c.label.clone(),
                align: c.align,
                width: c.width,
                min_width: c.min_width,
                max_width: c.max_width,
            })
            .collect()
    } else {
        // Collect unique keys preserving first-seen order
        let mut keys: Vec<String> = Vec::new();
        for row in data {
            for (k, _) in row {
                let ks = k.to_string();
                if !keys.contains(&ks) {
                    keys.push(ks);
                }
            }
        }
        keys.into_iter()
            .map(|k| Column {
                key: k,
                ..Default::default()
            })
            .collect()
    };

    if columns.is_empty() {
        return String::new();
    }

    // --- 2. Build a lookup: row index × column key → cell value ---
    // For each row, build a HashMap-like lookup using linear search (small tables).
    let cell_value = |row_idx: usize, col_key: &str| -> &str {
        for (k, v) in &data[row_idx] {
            if *k == col_key {
                return v;
            }
        }
        ""
    };

    // --- 3. Determine column labels ---
    let labels: Vec<String> = columns
        .iter()
        .map(|c| c.label.clone().unwrap_or_else(|| c.key.clone()))
        .collect();

    // --- 4. Calculate natural column widths ---
    let mut col_widths: Vec<usize> = columns
        .iter()
        .enumerate()
        .map(|(ci, col)| {
            // Start with header width
            let mut w = measure_width(&labels[ci]);
            // Expand for each cell
            for ri in 0..data.len() {
                let val = cell_value(ri, &col.key);
                let vw = measure_width(val);
                if vw > w {
                    w = vw;
                }
            }
            // Apply explicit width constraints
            if let Some(fixed) = col.width {
                return fixed;
            }
            if let Some(mn) = col.min_width {
                if w < mn {
                    w = mn;
                }
            }
            if let Some(mx) = col.max_width {
                if w > mx {
                    w = mx;
                }
            }
            w
        })
        .collect();

    // --- 5. Shrink columns to fit max_width ---
    let ncols = col_widths.len();
    let borderless = !opts.border.has_borders();

    // Overhead calculation depends on border mode:
    // Bordered:   (ncols + 1) border chars + ncols * 2 padding spaces = 3*ncols + 1
    // Borderless: (ncols - 1) separator spaces + ncols * 2 padding spaces = 3*ncols - 1
    let overhead = if borderless {
        3 * ncols - 1
    } else {
        3 * ncols + 1
    };

    // Iteratively shrink the widest column until we fit
    while col_widths.iter().sum::<usize>() + overhead > opts.max_width {
        let total = col_widths.iter().sum::<usize>();
        if total == 0 {
            break;
        }
        // Find the widest column
        let (widest_idx, &widest_val) = col_widths
            .iter()
            .enumerate()
            .max_by_key(|(_, &w)| w)
            .unwrap();
        if widest_val == 0 {
            break;
        }
        // Shrink by 1
        let new_w = widest_val.saturating_sub(1);
        col_widths[widest_idx] = new_w;

        // Respect min_width constraints — if we've hit a minimum, stop shrinking that col
        if let Some(mn) = columns[widest_idx].min_width {
            if new_w < mn {
                col_widths[widest_idx] = mn;
                break; // can't shrink further
            }
        }
    }

    // --- 6. Build border line helpers ---
    let bc = opts.border.chars();

    let h_repeat = |n: usize| bc.h.repeat(n);

    // A horizontal segment for each column: h*(width+2) to include the padding spaces
    let col_h = |ci: usize| h_repeat(col_widths[ci] + 2);

    // Top border: tl [h*(w+2) tt]* h*(w+2) tr
    let top_border = {
        let mut s = String::new();
        s.push_str(bc.tl);
        for ci in 0..ncols {
            s.push_str(&col_h(ci));
            if ci + 1 < ncols {
                s.push_str(bc.tt);
            }
        }
        s.push_str(bc.tr);
        s.push('\n');
        s
    };

    // Header separator: lt [h*(w+2) cross]* h*(w+2) rt
    let header_sep = {
        let mut s = String::new();
        s.push_str(bc.lt);
        for ci in 0..ncols {
            s.push_str(&col_h(ci));
            if ci + 1 < ncols {
                s.push_str(bc.cross);
            }
        }
        s.push_str(bc.rt);
        s.push('\n');
        s
    };

    // Bottom border: bl [h*(w+2) bt]* h*(w+2) br
    let bottom_border = {
        let mut s = String::new();
        s.push_str(bc.bl);
        for ci in 0..ncols {
            s.push_str(&col_h(ci));
            if ci + 1 < ncols {
                s.push_str(bc.bt);
            }
        }
        s.push_str(bc.br);
        s.push('\n');
        s
    };

    // Helper: render a data row (or header row) as a bordered line
    let render_row = |cells: &[String], aligns: &[Align]| -> String {
        let mut line = String::new();
        line.push_str(bc.v);
        for ci in 0..ncols {
            let w = col_widths[ci];
            let cell = &cells[ci];
            // Truncate if needed
            let truncated = truncate(cell, w, "…");
            let padded = align_text(&truncated, w, aligns[ci]);
            line.push(' ');
            line.push_str(&padded);
            line.push(' ');
            line.push_str(bc.v);
        }
        line.push('\n');
        line
    };

    // --- 7. Assemble the table ---
    let mut out = String::new();

    if borderless {
        // Borderless mode: header + thin separator + data rows, no box borders
        let render_borderless_row = |cells: &[String], aligns: &[Align]| -> String {
            let mut line = String::new();
            for ci in 0..ncols {
                let w = col_widths[ci];
                let cell = &cells[ci];
                let truncated = truncate(cell, w, "…");
                let padded = align_text(&truncated, w, aligns[ci]);
                if ci > 0 {
                    line.push_str("  "); // 2-space column separator
                }
                line.push_str(&padded);
            }
            line.push('\n');
            line
        };

        // Header row — bold+dim labels
        let header_cells: Vec<String> = labels.iter().map(|label| s().bold().dim().paint(label)).collect();
        let header_aligns: Vec<Align> = columns.iter().map(|_| Align::Left).collect();
        out.push_str(&render_borderless_row(&header_cells, &header_aligns));

        // Thin separator line (─ characters spanning full width)
        let total_vis = col_widths.iter().sum::<usize>() + (ncols - 1) * 2;
        out.push_str(&"─".repeat(total_vis));
        out.push('\n');

        // Data rows
        let data_aligns: Vec<Align> = columns.iter().map(|c| c.align).collect();
        for ri in 0..data.len() {
            let cells: Vec<String> = columns
                .iter()
                .map(|col| cell_value(ri, &col.key).to_string())
                .collect();
            out.push_str(&render_borderless_row(&cells, &data_aligns));
        }
    } else {
        // Bordered mode: full box drawing

        // Top border
        out.push_str(&top_border);

        // Header row — bold labels
        let header_cells: Vec<String> = labels.iter().map(|label| s().bold().paint(label)).collect();
        let header_aligns: Vec<Align> = columns.iter().map(|_| Align::Left).collect();
        out.push_str(&render_row(&header_cells, &header_aligns));

        // Header separator
        out.push_str(&header_sep);

        // Data rows
        let data_aligns: Vec<Align> = columns.iter().map(|c| c.align).collect();
        for ri in 0..data.len() {
            let cells: Vec<String> = columns
                .iter()
                .map(|col| cell_value(ri, &col.key).to_string())
                .collect();
            out.push_str(&render_row(&cells, &data_aligns));
        }

        // Bottom border
        out.push_str(&bottom_border);
    }

    out
}
