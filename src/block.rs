// prism/block - live terminal block
// the core I/O primitive: one updatable region pinned to bottom of terminal

use crate::writer;
use crate::ansi;

const SYNC_BEGIN: &str = "\x1b[?2026h";
const SYNC_END: &str = "\x1b[?2026l";

/// Output from a render callback: lines to display plus an optional cursor position.
pub struct BlockRender {
    pub lines: Vec<String>,
    /// Optional cursor placement as (row, col) within the rendered lines.
    pub cursor: Option<(u16, u16)>,
}

/// A live terminal block that can be updated, printed above, and closed.
pub struct LiveBlock {
    inner: LiveBlockInner,
}

enum LiveBlockInner {
    Tty(TtyBlock),
    Pipe(PipeBlock),
}

struct TtyBlock {
    render: Box<dyn FnMut() -> BlockRender>,
    on_close: Option<Box<dyn FnOnce()>>,
    prev_total_rows: u16,
    prev_cursor_row: u16,
    closed: bool,
}

struct PipeBlock {
    on_close: Option<Box<dyn FnOnce()>>,
    closed: bool,
}

/// Options for creating a [`LiveBlock`].
pub struct LiveBlockOptions {
    /// Render callback returning the current frame's lines and optional cursor position.
    pub render: Box<dyn FnMut() -> BlockRender>,
    /// Optional callback invoked when the block is closed.
    pub on_close: Option<Box<dyn FnOnce()>>,
    /// Force TTY mode on/off. If `None`, auto-detects via `writer::is_tty()`.
    pub tty: Option<bool>,
}

/// Create a new [`LiveBlock`] from the given options.
pub fn live_block(options: LiveBlockOptions) -> LiveBlock {
    let tty_mode = options.tty.unwrap_or_else(writer::is_tty);

    if !tty_mode {
        LiveBlock {
            inner: LiveBlockInner::Pipe(PipeBlock {
                on_close: options.on_close,
                closed: false,
            }),
        }
    } else {
        LiveBlock {
            inner: LiveBlockInner::Tty(TtyBlock {
                render: options.render,
                on_close: options.on_close,
                prev_total_rows: 0,
                prev_cursor_row: 0,
                closed: false,
            }),
        }
    }
}

impl LiveBlock {
    /// Erase the previous frame and draw the current one.
    pub fn update(&mut self) {
        match &mut self.inner {
            LiveBlockInner::Pipe(_) => {}
            LiveBlockInner::Tty(tty) => {
                if tty.closed {
                    return;
                }
                writer::write(SYNC_BEGIN);
                tty.erase();
                tty.draw();
                writer::write(SYNC_END);
            }
        }
    }

    /// Print text above the live block (erase, print, redraw).
    pub fn print(&mut self, text: &str) {
        match &mut self.inner {
            LiveBlockInner::Pipe(pipe) => {
                if !pipe.closed {
                    writer::writeln(text);
                }
            }
            LiveBlockInner::Tty(tty) => {
                if tty.closed {
                    return;
                }
                writer::write(SYNC_BEGIN);
                tty.erase();
                writer::writeln(text);
                tty.draw();
                writer::write(SYNC_END);
            }
        }
    }

    /// Close the block, optionally printing a final message.
    pub fn close(&mut self, message: Option<&str>) {
        match &mut self.inner {
            LiveBlockInner::Pipe(pipe) => {
                if pipe.closed {
                    return;
                }
                pipe.closed = true;
                if let Some(msg) = message {
                    writer::writeln(msg);
                }
                if let Some(cb) = pipe.on_close.take() {
                    cb();
                }
            }
            LiveBlockInner::Tty(tty) => {
                if tty.closed {
                    return;
                }
                tty.closed = true;
                tty.erase();
                if let Some(msg) = message {
                    writer::writeln(msg);
                }
                tty.prev_total_rows = 0;
                tty.prev_cursor_row = 0;
                if let Some(cb) = tty.on_close.take() {
                    cb();
                }
            }
        }
    }
}

impl TtyBlock {
    fn erase(&self) {
        if self.prev_total_rows == 0 {
            return;
        }
        if self.prev_cursor_row > 0 {
            writer::write(&format!("\x1b[{}A", self.prev_cursor_row));
        }
        writer::write("\r\x1b[J");
    }

    fn draw(&mut self) {
        let BlockRender { lines, cursor } = (self.render)();
        let width = writer::term_width();

        for line in &lines {
            writer::writeln(line);
        }

        let rows_per_line: Vec<u16> = lines.iter().map(|l| writer::visual_rows(l, width)).collect();
        let total_visual_rows: u16 = rows_per_line.iter().sum();
        self.prev_total_rows = total_visual_rows;

        if let Some((row, col)) = cursor {
            if !lines.is_empty() {
                let cursor_line = if (row as usize) < lines.len() {
                    &lines[row as usize]
                } else {
                    ""
                };
                let line_display_width = ansi::measure_width(cursor_line) as u16;
                let safe_col = col.min(line_display_width);

                let cursor_line_start: u16 = rows_per_line.iter()
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
