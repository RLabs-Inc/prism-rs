//! Two-zone terminal manager: output zone (scrollback) + active zone (refreshable bottom).
//!
//! Ports `layout.ts`. Single-threaded — all calls from main thread.
//! Uses DEC 2026 synchronized output to avoid flicker.

use std::sync::{Arc, Mutex};

use crate::ansi;
use crate::live::{
    self, Activity, ActivityOptions, FooterConfig, Section, SectionOptions,
};
use crate::stream::{self, LayoutPrint, Stream, StreamOptions};
use crate::writer;

const SYNC_BEGIN: &str = "\x1b[?2026h";
const SYNC_END: &str = "\x1b[?2026l";

// ---------------------------------------------------------------------------
// ActiveRender / ActiveFrame
// ---------------------------------------------------------------------------

/// A rendered frame from the active zone.
pub struct ActiveFrame {
    pub lines: Vec<String>,
    pub cursor: Option<(u16, u16)>,
}

/// Render callback for the active zone. Called to produce the current frame.
/// Must be `Send` so it can be referenced from footer closures on animation threads.
pub type ActiveRender = Box<dyn FnMut() -> ActiveFrame + Send>;

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

/// Options for creating a [`Layout`].
pub struct LayoutOptions {
    pub on_close: Option<Box<dyn FnOnce() + Send>>,
    pub tty: Option<bool>,
}

impl Default for LayoutOptions {
    #[allow(clippy::derivable_impls)]
    fn default() -> Self {
        Self {
            on_close: None,
            tty: None,
        }
    }
}

/// Options for layout-managed activities (footer/tty handled internally).
pub struct LayoutActivityOptions {
    pub icon: Option<crate::activity_line::Icon>,
    pub timer: bool,
    pub color: Option<fn(&str) -> String>,
    pub metrics: Option<Box<dyn Fn() -> String + Send>>,
}

impl Default for LayoutActivityOptions {
    #[allow(clippy::derivable_impls)]
    fn default() -> Self {
        Self {
            icon: None,
            timer: false,
            color: None,
            metrics: None,
        }
    }
}

/// Options for layout-managed sections (footer/tty handled internally).
pub struct LayoutSectionOptions {
    pub spinner: &'static str,
    pub color: Option<fn(&str) -> String>,
    pub indent: usize,
    pub connector: String,
    pub timer: bool,
    pub collapse_on_done: bool,
}

impl Default for LayoutSectionOptions {
    fn default() -> Self {
        Self {
            spinner: "dots",
            color: None,
            indent: 2,
            connector: "\u{23BF}".to_string(),
            timer: false,
            collapse_on_done: false,
        }
    }
}

/// Options for layout-managed streams (layout/tty handled internally).
pub struct LayoutStreamOptions {
    pub prefix: String,
    pub style: Option<fn(&str) -> String>,
}

impl Default for LayoutStreamOptions {
    #[allow(clippy::derivable_impls)]
    fn default() -> Self {
        Self {
            prefix: String::new(),
            style: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

type OnCloseSlot = Arc<Mutex<Option<Box<dyn FnOnce() + Send>>>>;

// ---------------------------------------------------------------------------
// Shared mutable state (behind Arc<Mutex<>>)
// ---------------------------------------------------------------------------

struct LayoutInner {
    render_fn: Option<ActiveRender>,
    prev_height: u16,
    prev_cursor_row: u16,
    write_buffer: String,
    closed: bool,
    live_active: u32,
}

impl LayoutInner {
    fn erase_active(&self) {
        if self.prev_height == 0 {
            return;
        }
        if self.prev_cursor_row > 0 {
            writer::write(&format!("\x1b[{}A", self.prev_cursor_row));
        }
        writer::write("\r\x1b[J");
    }

    fn draw_active(&mut self, frame: Option<ActiveFrame>) {
        let rendered = match frame {
            Some(f) => f,
            None => match self.render_fn.as_mut() {
                Some(rf) => rf(),
                None => return,
            },
        };

        let ActiveFrame { lines, cursor } = rendered;
        let width = writer::term_width();

        if lines.is_empty() {
            self.prev_height = 0;
            self.prev_cursor_row = 0;
            return;
        }

        for line in &lines {
            writer::write(line);
            writer::write("\n");
        }

        let rows_per_line: Vec<u16> = lines
            .iter()
            .map(|l| writer::visual_rows(l, width))
            .collect();
        let total_visual_rows: u16 = rows_per_line.iter().sum();
        self.prev_height = total_visual_rows;

        if let Some((row, col)) = cursor {
            if !lines.is_empty() {
                let cursor_line = if (row as usize) < lines.len() {
                    &lines[row as usize]
                } else {
                    ""
                };
                let line_display_width = ansi::measure_width(cursor_line) as u16;
                let safe_col = col.min(line_display_width);

                let cursor_line_start: u16 = rows_per_line
                    .iter()
                    .take((row as usize).min(rows_per_line.len()))
                    .sum();

                let cursor_sub_row = if safe_col >= width {
                    safe_col / width
                } else {
                    0
                };
                let cursor_visual_row = cursor_line_start + cursor_sub_row;

                let move_up = total_visual_rows.saturating_sub(cursor_visual_row);
                if move_up > 0 {
                    writer::write(&format!("\x1b[{}A", move_up));
                }
                writer::write("\r");
                let adjusted_col = if safe_col >= width {
                    safe_col % width
                } else {
                    safe_col
                };
                if adjusted_col > 0 {
                    writer::write(&format!("\x1b[{}C", adjusted_col));
                }

                self.prev_cursor_row = cursor_visual_row;
            } else {
                self.prev_cursor_row = total_visual_rows;
            }
        } else {
            self.prev_cursor_row = total_visual_rows;
        }
    }
}

// ---------------------------------------------------------------------------
// Layout (public API)
// ---------------------------------------------------------------------------

/// Two-zone terminal manager.
///
/// - **Output zone** (top): scrollback text via `print()` / `write()`.
/// - **Active zone** (bottom): refreshable region via `set_active()` / `refresh()`.
///
/// In non-TTY mode the active zone is silent; only the output zone works.
#[derive(Clone)]
pub struct Layout {
    inner: Arc<Mutex<LayoutInner>>,
    tty_mode: bool,
    on_close: OnCloseSlot,
}

impl Layout {
    /// Print a complete line into the output zone (above the active zone).
    pub fn print(&self, text: &str) {
        let mut inner = self.inner.lock().unwrap();
        if inner.closed {
            return;
        }

        if !self.tty_mode {
            writer::writeln(text);
            return;
        }

        // Check if active zone is showing
        let has_active = inner.render_fn.is_some();
        if !has_active || (inner.prev_height == 0) {
            // Render to see if there is content
            let frame = inner.render_fn.as_mut().map(|rf| rf());
            match frame {
                Some(ref f) if f.lines.is_empty() && inner.prev_height == 0 => {
                    writer::writeln(text);
                }
                None => {
                    writer::writeln(text);
                }
                _ => {
                    writer::write(SYNC_BEGIN);
                    inner.erase_active();
                    writer::writeln(text);
                    inner.draw_active(frame);
                    writer::write(SYNC_END);
                }
            }
        } else {
            let frame = inner.render_fn.as_mut().map(|rf| rf());
            writer::write(SYNC_BEGIN);
            inner.erase_active();
            writer::writeln(text);
            inner.draw_active(frame);
            writer::write(SYNC_END);
        }
    }

    /// Set (or replace) the active zone render function.
    pub fn set_active(&self, render: ActiveRender) {
        let mut inner = self.inner.lock().unwrap();
        if inner.closed {
            return;
        }

        if !self.tty_mode {
            inner.render_fn = Some(render);
            return;
        }

        if inner.live_active > 0 {
            inner.render_fn = Some(render);
            return;
        }

        inner.render_fn = Some(render);
        let frame = inner.render_fn.as_mut().unwrap()();
        if inner.prev_height == 0 && frame.lines.is_empty() {
            return;
        }
        writer::write(SYNC_BEGIN);
        inner.erase_active();
        inner.draw_active(Some(frame));
        writer::write(SYNC_END);
    }

    /// Redraw the active zone with the current render function.
    pub fn refresh(&self) {
        let mut inner = self.inner.lock().unwrap();
        if inner.closed || inner.render_fn.is_none() {
            return;
        }

        if !self.tty_mode || inner.live_active > 0 {
            return;
        }

        let frame = inner.render_fn.as_mut().unwrap()();
        if inner.prev_height == 0 && frame.lines.is_empty() {
            return;
        }
        writer::write(SYNC_BEGIN);
        inner.erase_active();
        inner.draw_active(Some(frame));
        writer::write(SYNC_END);
    }

    /// Write raw data into the output zone. Buffers until a newline is seen,
    /// then flushes complete lines above the active zone.
    pub fn write(&self, data: &str) {
        let mut inner = self.inner.lock().unwrap();
        if inner.closed || data.is_empty() {
            return;
        }

        if !self.tty_mode {
            writer::write(data);
            return;
        }

        inner.write_buffer.push_str(data);
        let last_newline = match inner.write_buffer.rfind('\n') {
            Some(pos) => pos,
            None => return,
        };

        let complete = inner.write_buffer[..last_newline].to_string();
        let remainder = inner.write_buffer[last_newline + 1..].to_string();
        inner.write_buffer = remainder;

        let frame = inner.render_fn.as_mut().map(|rf| rf());
        match &frame {
            Some(f) if f.lines.is_empty() && inner.prev_height == 0 => {
                writer::writeln(&complete);
            }
            None => {
                writer::writeln(&complete);
            }
            _ => {
                writer::write(SYNC_BEGIN);
                inner.erase_active();
                writer::writeln(&complete);
                inner.draw_active(frame);
                writer::write(SYNC_END);
            }
        }
    }

    /// Close the layout. Erases the active zone, flushes any buffered text,
    /// and optionally prints a final message.
    pub fn close(&self, message: Option<&str>) {
        let mut inner = self.inner.lock().unwrap();
        if inner.closed {
            return;
        }
        inner.closed = true;

        if !self.tty_mode {
            if let Some(msg) = message {
                writer::writeln(msg);
            }
            drop(inner);
            if let Some(cb) = self.on_close.lock().unwrap().take() {
                cb();
            }
            return;
        }

        let has_active = inner.prev_height > 0;
        let has_buffered = !inner.write_buffer.is_empty();

        if has_active || has_buffered {
            writer::write(SYNC_BEGIN);
        }
        inner.erase_active();
        if has_buffered {
            let buf = std::mem::take(&mut inner.write_buffer);
            writer::writeln(&buf);
        }
        if let Some(msg) = message {
            writer::writeln(msg);
        }
        if has_active || has_buffered {
            writer::write(SYNC_END);
        }

        inner.prev_height = 0;
        inner.prev_cursor_row = 0;
        inner.render_fn = None;

        drop(inner);
        if let Some(cb) = self.on_close.lock().unwrap().take() {
            cb();
        }
    }

    /// Create a live activity managed by this layout.
    pub fn activity(&self, text: &str, options: Option<LayoutActivityOptions>) -> Activity {
        let opts = options.unwrap_or_default();

        let inner = self.inner.lock().unwrap();
        if inner.closed {
            return live::activity(
                text,
                ActivityOptions {
                    icon: opts.icon,
                    timer: opts.timer,
                    color: opts.color,
                    metrics: opts.metrics,
                    footer: None,
                    tty: None,
                },
            );
        }
        drop(inner);

        if !self.tty_mode {
            return live::activity(
                text,
                ActivityOptions {
                    icon: opts.icon,
                    timer: opts.timer,
                    color: opts.color,
                    metrics: opts.metrics,
                    footer: None,
                    tty: Some(false),
                },
            );
        }

        let footer = self.create_footer();
        live::activity(
            text,
            ActivityOptions {
                icon: opts.icon,
                timer: opts.timer,
                color: opts.color,
                metrics: opts.metrics,
                footer: Some(footer),
                tty: Some(true),
            },
        )
    }

    /// Create a live section managed by this layout.
    pub fn section(&self, title: &str, options: Option<LayoutSectionOptions>) -> Section {
        let opts = options.unwrap_or_default();

        let inner = self.inner.lock().unwrap();
        if inner.closed {
            return live::section(
                title,
                SectionOptions {
                    spinner: opts.spinner,
                    color: opts.color,
                    indent: opts.indent,
                    connector: opts.connector,
                    timer: opts.timer,
                    collapse_on_done: opts.collapse_on_done,
                    footer: None,
                    tty: None,
                },
            );
        }
        drop(inner);

        if !self.tty_mode {
            return live::section(
                title,
                SectionOptions {
                    spinner: opts.spinner,
                    color: opts.color,
                    indent: opts.indent,
                    connector: opts.connector,
                    timer: opts.timer,
                    collapse_on_done: opts.collapse_on_done,
                    footer: None,
                    tty: Some(false),
                },
            );
        }

        let footer = self.create_footer();
        live::section(
            title,
            SectionOptions {
                spinner: opts.spinner,
                color: opts.color,
                indent: opts.indent,
                connector: opts.connector,
                timer: opts.timer,
                collapse_on_done: opts.collapse_on_done,
                footer: Some(footer),
                tty: Some(true),
            },
        )
    }

    /// Create a buffered stream managed by this layout.
    pub fn stream(&self, options: Option<LayoutStreamOptions>) -> Stream {
        let opts = options.unwrap_or_default();

        let inner = self.inner.lock().unwrap();
        if inner.closed {
            return stream::stream_with(StreamOptions {
                layout: None,
                prefix: opts.prefix,
                style: opts.style,
                tty: None,
            });
        }
        drop(inner);

        let tty = if self.tty_mode { Some(true) } else { Some(false) };

        stream::stream_with(StreamOptions {
            layout: Some(Box::new(LayoutPrinter {
                inner: Arc::clone(&self.inner),
                tty_mode: self.tty_mode,
            })),
            prefix: opts.prefix,
            style: opts.style,
            tty,
        })
    }

    // -- internal --

    fn create_footer(&self) -> FooterConfig {
        // Erase active zone and reset height tracking
        {
            let mut inner = self.inner.lock().unwrap();
            inner.erase_active();
            inner.prev_height = 0;
            inner.prev_cursor_row = 0;
            inner.live_active += 1;
        }

        let render_inner = Arc::clone(&self.inner);
        let end_inner = Arc::clone(&self.inner);

        FooterConfig {
            render: Box::new(move || {
                let mut inner = render_inner.lock().unwrap();
                match inner.render_fn.as_mut() {
                    Some(rf) => rf().lines,
                    None => Vec::new(),
                }
            }),
            on_end: Box::new(move || {
                let mut inner = end_inner.lock().unwrap();
                inner.live_active = inner.live_active.saturating_sub(1);
                if inner.live_active == 0 {
                    inner.draw_active(None);
                }
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// LayoutPrint impl for Stream integration
// ---------------------------------------------------------------------------

struct LayoutPrinter {
    inner: Arc<Mutex<LayoutInner>>,
    tty_mode: bool,
}

impl LayoutPrint for LayoutPrinter {
    fn print(&self, text: &str) {
        let mut inner = self.inner.lock().unwrap();
        if inner.closed {
            return;
        }

        if !self.tty_mode {
            writer::writeln(text);
            return;
        }

        let frame = inner.render_fn.as_mut().map(|rf| rf());
        match &frame {
            Some(f) if f.lines.is_empty() && inner.prev_height == 0 => {
                writer::writeln(text);
            }
            None => {
                writer::writeln(text);
            }
            _ => {
                writer::write(SYNC_BEGIN);
                inner.erase_active();
                writer::writeln(text);
                inner.draw_active(frame);
                writer::write(SYNC_END);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Constructor
// ---------------------------------------------------------------------------

/// Create a two-zone layout manager.
///
/// In TTY mode, manages an output zone (scrollback) and an active zone
/// (refreshable bottom region). In non-TTY mode, the active zone is silent
/// and only output zone methods (`print`, `write`) produce output.
pub fn layout(options: Option<LayoutOptions>) -> Layout {
    let opts = options.unwrap_or_default();
    let tty_mode = opts.tty.unwrap_or_else(writer::is_tty);

    Layout {
        inner: Arc::new(Mutex::new(LayoutInner {
            render_fn: None,
            prev_height: 0,
            prev_cursor_row: 0,
            write_buffer: String::new(),
            closed: false,
            live_active: 0,
        })),
        tty_mode,
        on_close: Arc::new(Mutex::new(
            opts.on_close,
        )),
    }
}
