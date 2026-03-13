use crate::style::s;

/// Bar style characters
struct BarChars {
    filled: &'static str,
    empty: &'static str,
    left: &'static str,
    right: &'static str,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum BarStyle {
    #[default]
    Bar,
    Blocks,
    Shades,
    Classic,
    Arrows,
    Smooth,
    Dots,
    Square,
    Circle,
    Pipe,
}

impl BarStyle {
    fn chars(&self) -> BarChars {
        match self {
            BarStyle::Bar     => BarChars { filled: "█", empty: "░", left: "",  right: ""  },
            BarStyle::Blocks  => BarChars { filled: "▓", empty: "░", left: "",  right: ""  },
            BarStyle::Shades  => BarChars { filled: "█", empty: " ", left: "▐", right: "▌" },
            BarStyle::Classic => BarChars { filled: "=", empty: " ", left: "[", right: "]" },
            BarStyle::Arrows  => BarChars { filled: "▰", empty: "▱", left: "",  right: ""  },
            BarStyle::Smooth  => BarChars { filled: "━", empty: "─", left: "",  right: ""  },
            BarStyle::Dots    => BarChars { filled: "⣿", empty: "⠀", left: "",  right: ""  },
            BarStyle::Square  => BarChars { filled: "■", empty: "□", left: "",  right: ""  },
            BarStyle::Circle  => BarChars { filled: "●", empty: "○", left: "",  right: ""  },
            BarStyle::Pipe    => BarChars { filled: "┃", empty: "╌", left: "┫", right: "┣" },
        }
    }
}

/// Sub-character precision blocks (1/8 to 7/8)
const PARTIALS: [&str; 8] = ["", "▏", "▎", "▍", "▌", "▋", "▊", "▉"];

pub struct RenderOptions {
    pub total: u64,
    pub width: usize,
    pub style: BarStyle,
    pub smooth: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            total: 100,
            width: 30,
            style: BarStyle::Bar,
            smooth: true,
        }
    }
}

/// Pure render function — returns the bar string for a given value
pub fn render_progress_bar(current: u64, options: &RenderOptions) -> String {
    let pct = if options.total == 0 {
        1.0
    } else {
        (current as f64 / options.total as f64).clamp(0.0, 1.0)
    };

    let bs = options.style.chars();
    let bar_width = options.width.max(1);
    let can_smooth = options.smooth && matches!(options.style, BarStyle::Bar | BarStyle::Shades | BarStyle::Blocks);

    let bar = if can_smooth {
        let full_chars = (pct * bar_width as f64).floor() as usize;
        let remainder = (pct * bar_width as f64) - full_chars as f64;
        let partial_idx = (remainder * 7.0).round() as usize;
        let partial = PARTIALS.get(partial_idx).unwrap_or(&"");
        let empty_width = bar_width.saturating_sub(full_chars).saturating_sub(if partial.is_empty() { 0 } else { 1 });

        format!("{}{}{}",
            s().cyan().paint(&bs.filled.repeat(full_chars)),
            if !partial.is_empty() { s().cyan().paint(partial) } else { String::new() },
            s().dim().paint(&bs.empty.repeat(empty_width))
        )
    } else {
        let filled_width = (pct * bar_width as f64).round() as usize;
        let empty_width = bar_width - filled_width;
        format!("{}{}",
            s().cyan().paint(&bs.filled.repeat(filled_width)),
            s().dim().paint(&bs.empty.repeat(empty_width))
        )
    };

    format!("{}{}{}", bs.left, bar, bs.right)
}
