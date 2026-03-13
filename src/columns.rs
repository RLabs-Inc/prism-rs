use crate::ansi::measure_width;

pub struct ColumnsOptions {
    pub gap: usize,
    pub padding: usize,
    pub min_width: usize,
    pub max_columns: Option<usize>,
    pub total_width: usize,
}

impl Default for ColumnsOptions {
    fn default() -> Self {
        Self {
            gap: 2,
            padding: 0,
            min_width: 10,
            max_columns: None,
            total_width: 80,
        }
    }
}

pub fn columns(items: &[&str], options: &ColumnsOptions) -> String {
    if items.is_empty() {
        return String::new();
    }

    let total_width = options.total_width.saturating_sub(options.padding);
    let max_item_width = items.iter().map(|i| measure_width(i)).max().unwrap_or(0);
    let col_width = max_item_width.max(options.min_width);

    let mut num_cols = ((total_width + options.gap) / (col_width + options.gap)).max(1);
    if let Some(max) = options.max_columns {
        num_cols = num_cols.min(max);
    }

    let pad = " ".repeat(options.padding);
    let gap_str = " ".repeat(options.gap);

    items
        .chunks(num_cols)
        .map(|row| {
            let formatted: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(j, item)| {
                    if j == row.len() - 1 {
                        item.to_string()
                    } else {
                        let w = measure_width(item);
                        format!("{}{}", item, " ".repeat(col_width.saturating_sub(w)))
                    }
                })
                .collect();
            format!("{}{}", pad, formatted.join(&gap_str))
        })
        .collect::<Vec<_>>()
        .join("\n")
}
