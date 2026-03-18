// prism/progress - live progress bar with I/O, ETA, and smooth rendering
// Composes progress_bar (pure renderer) + block (I/O) + elapsed (timer)

use std::cell::RefCell;
use std::rc::Rc;

use crate::ansi;
use crate::block::{self, BlockRender, LiveBlock, LiveBlockOptions};
use crate::cursor;
use crate::elapsed::Elapsed;
use crate::progress_bar::{self, BarStyle, RenderOptions};
use crate::style::s;
use crate::writer;

pub use crate::progress_bar::BarStyle as ProgressStyle;

/// Options for creating a progress bar.
pub struct ProgressOptions {
    /// Total value (default: 100)
    pub total: u64,
    /// Bar width in columns (auto-sized if None)
    pub width: Option<usize>,
    /// Bar visual style (default: Bar)
    pub style: BarStyle,
    /// Bar color function (default: cyan). Passed through to render_progress_bar.
    pub color: Option<fn(&str) -> String>,
    /// Whether to show percentage (default: true)
    pub show_percent: bool,
    /// Whether to show count (e.g. "42/100")
    pub show_count: bool,
    /// Whether to show estimated time remaining
    pub show_eta: bool,
    /// Sub-character smooth rendering (default: true)
    pub smooth: bool,
}

impl Default for ProgressOptions {
    fn default() -> Self {
        Self {
            total: 100,
            width: None,
            style: BarStyle::Bar,
            color: None,
            show_percent: true,
            show_count: false,
            show_eta: false,
            smooth: true,
        }
    }
}

/// Internal mutable state shared between the render closure and the ProgressBar handle.
struct ProgressState {
    current: u64,
    total: u64,
    stopped: bool,
    timer: Elapsed,
}

/// A live progress bar that can be updated, completed, or failed.
pub struct ProgressBar {
    state: Rc<RefCell<ProgressState>>,
    block: LiveBlock,
    text: String,
    tty: bool,
}

impl ProgressBar {
    /// Update the current progress value. Optionally update the total.
    pub fn update(&mut self, current: u64, total: Option<u64>) {
        let mut st = self.state.borrow_mut();
        if st.stopped {
            return;
        }
        st.current = current;
        if let Some(t) = total {
            st.total = t;
        }
        drop(st);

        if self.tty {
            self.block.update();
        }
    }

    /// Mark the progress bar as successfully completed.
    pub fn done(&mut self, msg: Option<&str>) {
        let display = msg.map_or_else(|| self.text.clone(), |m| m.to_string());
        self.end("\u{2713}", &display, |t| s().green().paint(t));
    }

    /// Mark the progress bar as failed.
    pub fn fail(&mut self, msg: Option<&str>) {
        let display = msg.map_or_else(|| self.text.clone(), |m| m.to_string());
        self.end("\u{2717}", &display, |t| s().red().paint(t));
    }

    fn end(&mut self, icon: &str, msg: &str, icon_color: fn(&str) -> String) {
        let elapsed_str;
        {
            let mut st = self.state.borrow_mut();
            if st.stopped {
                return;
            }
            elapsed_str = st.timer.render();
            st.stopped = true;
        }

        let elapsed = s().dim().paint(&elapsed_str);
        let final_msg = format!("{} {} {}", icon_color(icon), msg, elapsed);

        if self.tty {
            self.block.close(Some(&final_msg));
            cursor::show_cursor();
        } else {
            writer::writeln(&final_msg);
        }
    }
}

/// Create a new live progress bar.
///
/// In TTY mode, renders a live-updating bar with optional percentage, count, and ETA.
/// In pipe mode, `update()` is silent; `done()`/`fail()` print a single line.
pub fn progress(text: &str, options: ProgressOptions) -> ProgressBar {
    let tty = writer::is_tty();

    let state = Rc::new(RefCell::new(ProgressState {
        current: 0,
        total: options.total,
        stopped: false,
        timer: Elapsed::new(),
    }));

    if !tty {
        // Non-TTY: create a dummy block (never rendered)
        let block = block::live_block(LiveBlockOptions {
            render: Box::new(|| BlockRender {
                lines: vec![],
                cursor: None,
            }),
            on_close: None,
            tty: Some(false),
        });

        return ProgressBar {
            state,
            block,
            text: text.to_string(),
            tty: false,
        };
    }

    cursor::hide_cursor();

    let render_text = text.to_string();
    let render_state = Rc::clone(&state);
    let show_percent = options.show_percent;
    let show_count = options.show_count;
    let show_eta = options.show_eta;
    let bar_style = options.style;
    let smooth = options.smooth;
    let explicit_width = options.width;
    let color_fn = options.color;

    let render_fn = move || {
        let st = render_state.borrow();
        let current = st.current;
        let total = st.total;
        drop(st);

        let pct = if total == 0 {
            1.0
        } else {
            (current as f64 / total as f64).clamp(0.0, 1.0)
        };

        let (left_w, right_w) = bar_style.decoration_widths();
        let decoration_width = left_w + right_w;
        let extra_width = if show_percent { 5 } else { 0 }
            + if show_count {
                // "current/total" → digits(total) * 2 + 1 slash + 1 space
                digit_count(total) * 2 + 2
            } else {
                0
            }
            + if show_eta { 10 } else { 0 };

        let text_width = ansi::measure_width(&render_text);
        let term_w = writer::term_width() as usize;
        let computed_width = explicit_width.unwrap_or_else(|| {
            term_w.saturating_sub(text_width + decoration_width + extra_width + 4)
        });

        // If computed width is too small and no explicit width, fall back to text-only
        if computed_width < 10 && explicit_width.is_none() {
            let pct_str = s()
                .bold()
                .paint(&format!("{}%", (pct * 100.0).round() as u64));
            return BlockRender {
                lines: vec![format!("{} {}", render_text, pct_str)],
                cursor: None,
            };
        }

        let bar_width = computed_width.max(10);
        let bar = progress_bar::render_progress_bar(
            current,
            &RenderOptions {
                total,
                width: bar_width,
                style: bar_style,
                color: color_fn,
                smooth,
                empty_char: None,
            },
        );

        let mut parts = vec![bar];

        if show_percent {
            parts.push(
                s().bold()
                    .paint(&format!("{}%", (pct * 100.0).round() as u64)),
            );
        }
        if show_count {
            parts.push(s().dim().paint(&format!("{}/{}", current, total)));
        }
        if show_eta && current > 0 && pct < 1.0 {
            let elapsed_sec = render_state.borrow().timer.ms() as f64 / 1000.0;
            let rate = current as f64 / elapsed_sec;
            let remaining = ((total - current) as f64 / rate).max(0.0);
            if remaining < 60.0 {
                parts.push(s().dim().paint(&format!("~{}s", remaining as u64)));
            } else {
                parts.push(s().dim().paint(&format!("~{:.1}m", remaining / 60.0)));
            }
        }

        BlockRender {
            lines: vec![format!("{} {}", render_text, parts.join(" "))],
            cursor: None,
        }
    };

    let mut block = block::live_block(LiveBlockOptions {
        render: Box::new(render_fn),
        on_close: None,
        tty: Some(true),
    });

    block.update();

    ProgressBar {
        state,
        block,
        text: text.to_string(),
        tty: true,
    }
}

/// Count decimal digits in a number.
fn digit_count(n: u64) -> usize {
    if n == 0 {
        return 1;
    }
    (n as f64).log10().floor() as usize + 1
}
