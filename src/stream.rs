//! Buffered text stream that flushes complete lines.
//!
//! Ports `stream.ts` — buffers incoming text chunks and emits whole lines,
//! with optional partial-line display for TTY mode.

use crate::writer;

/// Trait for layout-aware line output. Layout modules implement this so the
/// stream can print through a managed layout rather than directly to stdout.
pub trait LayoutPrint {
    fn print(&self, text: &str);
}

/// Configuration for creating a [`Stream`].
pub struct StreamOptions {
    /// Layout target for line output (if `None`, writes directly to stdout).
    pub layout: Option<Box<dyn LayoutPrint>>,
    /// Prefix prepended to every line before output.
    pub prefix: String,
    /// Optional styling function applied to `prefix + line` content.
    pub style: Option<fn(&str) -> String>,
    /// Whether to use TTY mode (partial line display, \r clearing).
    /// Defaults to `writer::is_tty()` if `None`.
    pub tty: Option<bool>,
}

impl Default for StreamOptions {
    #[allow(clippy::derivable_impls)]
    fn default() -> Self {
        Self {
            layout: None,
            prefix: String::new(),
            style: None,
            tty: None,
        }
    }
}

/// A buffered text stream that collects chunks and emits complete lines.
pub struct Stream {
    layout: Option<Box<dyn LayoutPrint>>,
    prefix: String,
    style_fn: Option<fn(&str) -> String>,
    tty_mode: bool,
    buffer: String,
    closed: bool,
    has_partial: bool,
    /// True when operating in passthrough mode (non-TTY, no layout).
    passthrough: bool,
}

impl Stream {
    /// Create a new stream with the given options.
    pub fn new(options: StreamOptions) -> Self {
        let tty_mode = options.tty.unwrap_or_else(writer::is_tty);
        let passthrough = !tty_mode && options.layout.is_none();

        Self {
            layout: options.layout,
            prefix: options.prefix,
            style_fn: options.style,
            tty_mode,
            buffer: String::new(),
            closed: false,
            has_partial: false,
            passthrough,
        }
    }

    /// Create a stream with default options.
    pub fn default_stream() -> Self {
        Self::new(StreamOptions::default())
    }

    /// Write a chunk of data into the stream. Complete lines are flushed
    /// immediately; any trailing partial line is buffered (and shown as a
    /// partial update in TTY mode).
    pub fn write(&mut self, data: &str) {
        if self.closed || data.is_empty() {
            return;
        }

        if self.passthrough {
            writer::write(data);
            return;
        }

        self.buffer.push_str(data);
        self.process_buffer();
    }

    /// Flush any remaining buffered text as a complete line.
    pub fn flush(&mut self) {
        if self.closed {
            return;
        }
        if !self.passthrough {
            self.flush_buffer();
        }
    }

    /// Mark the stream as done. Flushes remaining buffer and optionally
    /// prints a final message.
    pub fn done(&mut self, final_text: Option<&str>) {
        if self.closed {
            return;
        }
        self.closed = true;

        if self.passthrough {
            if let Some(text) = final_text {
                writer::writeln(text);
            }
            return;
        }

        self.flush_buffer();
        if let Some(text) = final_text {
            if let Some(ref ly) = self.layout {
                ly.print(text);
            } else {
                writer::writeln(text);
            }
        }
    }

    /// Mark the stream as failed. Flushes remaining buffer and optionally
    /// prints an error message in red.
    pub fn fail(&mut self, error_text: Option<&str>) {
        if self.closed {
            return;
        }
        self.closed = true;

        if self.passthrough {
            if let Some(text) = error_text {
                writer::writeln(text);
            }
            return;
        }

        self.flush_buffer();
        if let Some(text) = error_text {
            let red = if writer::ansi_enabled() {
                crate::style::s().red().render(text)
            } else if self.tty_mode || self.layout.is_some() {
                format!("\x1b[31m{}\x1b[39m", text)
            } else {
                text.to_string()
            };
            if let Some(ref ly) = self.layout {
                ly.print(&red);
            } else {
                writer::writeln(&red);
            }
        }
    }

    /// Update the prefix prepended to every line.
    pub fn text(&mut self, new_prefix: &str) {
        self.prefix = new_prefix.to_string();
    }

    /// Whether this stream has been closed (via `done` or `fail`).
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    // -- internal helpers --

    fn format_line(&self, line: &str) -> String {
        let content = format!("{}{}", self.prefix, line);
        if let Some(f) = self.style_fn {
            f(&content)
        } else {
            content
        }
    }

    fn output_line(&self, line: &str) {
        let formatted = self.format_line(line);
        if let Some(ref ly) = self.layout {
            ly.print(&formatted);
        } else {
            writer::writeln(&formatted);
        }
    }

    fn clear_partial(&mut self) {
        if self.layout.is_none() && self.has_partial {
            writer::write("\r\x1b[2K");
            self.has_partial = false;
        }
    }

    fn show_partial(&mut self) {
        if self.layout.is_some() || self.buffer.is_empty() {
            return;
        }
        let formatted = self.format_line(&self.buffer);
        writer::write(&format!("\r\x1b[2K{}", formatted));
        self.has_partial = true;
    }

    fn process_buffer(&mut self) {
        let last_newline = match self.buffer.rfind('\n') {
            Some(pos) => pos,
            None => {
                self.show_partial();
                return;
            }
        };

        let complete = self.buffer[..last_newline].to_string();
        let remainder = self.buffer[last_newline + 1..].to_string();
        self.buffer = remainder;

        self.clear_partial();
        for line in complete.split('\n') {
            self.output_line(line);
        }
        self.show_partial();
    }

    fn flush_buffer(&mut self) {
        if self.buffer.is_empty() {
            return;
        }
        self.clear_partial();
        let buf = std::mem::take(&mut self.buffer);
        self.output_line(&buf);
    }
}

/// Convenience constructor: create a stream with default options.
pub fn stream() -> Stream {
    Stream::default_stream()
}

/// Create a stream with the given options.
pub fn stream_with(options: StreamOptions) -> Stream {
    Stream::new(options)
}
