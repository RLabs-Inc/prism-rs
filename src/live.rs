// prism/live — Activity + Section with threaded animation
// Composes pure state machines (activity_line, section_block) with terminal I/O
// and a background thread for animation.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use crate::activity_line::{ActivityLine, ActivityLineOptions, Icon};
use crate::block::{live_block, BlockRender, LiveBlockOptions};
use crate::cursor::{hide_cursor, show_cursor};
use crate::section_block::{SectionBlock, SectionBlockOptions};
use crate::style::s;
use crate::writer;

// ---------------------------------------------------------------------------
// Footer config — active zone renders below live content
// ---------------------------------------------------------------------------

/// Footer configuration for rendering additional lines below the live content.
pub struct FooterConfig {
    /// Returns the footer lines to display.
    pub render: Box<dyn Fn() -> Vec<String> + Send + Sync>,
    /// Called when the live block closes.
    pub on_end: Box<dyn FnOnce() + Send>,
}

// ---------------------------------------------------------------------------
// Color helpers
// ---------------------------------------------------------------------------

fn green(text: &str) -> String {
    s().green().render(text)
}
fn red(text: &str) -> String {
    s().red().render(text)
}
fn yellow(text: &str) -> String {
    s().yellow().render(text)
}
fn blue(text: &str) -> String {
    s().blue().render(text)
}
fn white(text: &str) -> String {
    s().white().render(text)
}

// ---------------------------------------------------------------------------
// Activity — single-line animated status
// ---------------------------------------------------------------------------

/// Options for creating an [`Activity`].
#[derive(Default)]
pub struct ActivityOptions {
    pub icon: Option<Icon>,
    pub timer: bool,
    pub color: Option<fn(&str) -> String>,
    pub metrics: Option<Box<dyn Fn() -> String + Send>>,
    pub footer: Option<FooterConfig>,
    pub tty: Option<bool>,
}

/// A live activity handle. Methods are safe to call from the main thread.
pub struct Activity {
    inner: ActivityInner,
}

enum ActivityInner {
    /// Non-TTY: writes static lines, no animation.
    Pipe(PipeActivity),
    /// TTY: background thread drives animation.
    Tty(TtyActivity),
}

struct PipeActivity {
    msg: String,
}

/// Commands sent from the main thread to the animation thread.
enum ActivityCmd {
    Text(String),
    /// Stop the animation and freeze with (icon, optional_msg, color_fn).
    End {
        icon: String,
        msg: Option<String>,
        color: fn(&str) -> String,
        /// Channel to send the frozen line back on.
        reply: mpsc::Sender<String>,
    },
}

struct TtyActivity {
    tx: mpsc::Sender<ActivityCmd>,
    stopped: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    /// Shared footer config (needs to be accessible from thread and main).
    footer_on_end: Option<Box<dyn FnOnce() + Send>>,
}

/// Create a live activity (single-line animated status indicator).
///
/// Returns an [`Activity`] handle. In TTY mode, a background thread drives
/// the spinner animation. In non-TTY mode, writes static lines.
pub fn activity(text: &str, options: ActivityOptions) -> Activity {
    let tty_mode = options.tty.unwrap_or_else(writer::is_tty);

    if !tty_mode {
        writer::writeln(text);
        return Activity {
            inner: ActivityInner::Pipe(PipeActivity {
                msg: text.to_string(),
            }),
        };
    }

    // TTY mode: spawn animation thread
    let (tx, rx) = mpsc::channel::<ActivityCmd>();
    let stopped = Arc::new(AtomicBool::new(false));
    let stopped_thread = Arc::clone(&stopped);

    let text_owned = text.to_string();
    let icon = options.icon;
    let timer = options.timer;
    let color = options.color;
    let metrics = options.metrics;
    let footer = options.footer;

    // Extract on_end callback to call on main thread after join
    let (footer_render, footer_on_end) = match footer {
        Some(fc) => (Some(fc.render), Some(fc.on_end)),
        None => (None, None),
    };

    let footer_render: Option<Arc<dyn Fn() -> Vec<String> + Send + Sync>> =
        footer_render.map(|r| Arc::from(r) as Arc<dyn Fn() -> Vec<String> + Send + Sync>);
    let footer_render_thread = footer_render.clone();

    hide_cursor();

    let handle = thread::spawn(move || {
        // Build the activity line state machine
        // Cast metrics from Box<dyn Fn + Send> to Box<dyn Fn> (now inside the thread)
        let metrics_unsend: Option<Box<dyn Fn() -> String>> =
            metrics.map(|m| m as Box<dyn Fn() -> String>);
        let mut act = ActivityLine::new(
            &text_owned,
            ActivityLineOptions {
                icon,
                interval_ms: None,
                color,
                timer,
                metrics: metrics_unsend,
            },
        );

        let interval = Duration::from_millis(act.interval_ms());
        let has_footer = footer_render_thread.is_some();

        if has_footer {
            // Use a LiveBlock for rendering (content + footer)
            let footer_render_ref = footer_render_thread.unwrap();
            // We need a mutable reference to act inside the render closure,
            // but also need to call act.tick() outside. Use a shared pointer.
            // Since this is all single-threaded (within the animation thread), use RefCell.
            use std::cell::RefCell;
            use std::rc::Rc;

            let act_cell = Rc::new(RefCell::new(act));
            let act_render = Rc::clone(&act_cell);

            let mut block = live_block(LiveBlockOptions {
                render: Box::new(move || {
                    let act = act_render.borrow();
                    let content_lines = act.render();
                    let footer_lines = (footer_render_ref)();
                    let cursor_row = content_lines.len().saturating_sub(1) as u16;
                    let mut lines = content_lines;
                    lines.extend(footer_lines);
                    BlockRender {
                        lines,
                        cursor: Some((cursor_row, 0)),
                    }
                }),
                on_close: None,
                tty: Some(true),
            });

            block.update();

            loop {
                // Check for commands (non-blocking)
                match rx.recv_timeout(interval) {
                    Ok(ActivityCmd::Text(m)) => {
                        act_cell.borrow_mut().text(&m);
                        act_cell.borrow_mut().tick();
                        block.update();
                    }
                    Ok(ActivityCmd::End {
                        icon,
                        msg,
                        color,
                        reply,
                    }) => {
                        let mut act = act_cell.borrow_mut();
                        let frozen = act.freeze(&icon, msg.as_deref(), Some(color));
                        let frozen_line = frozen.into_iter().next().unwrap_or_default();
                        block.close(Some(&frozen_line));
                        let _ = reply.send(frozen_line);
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if stopped_thread.load(Ordering::Relaxed) {
                            break;
                        }
                        act_cell.borrow_mut().tick();
                        block.update();
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        } else {
            // No footer: render directly with \r\x1b[2K
            writer::write(&format!("\r\x1b[2K{}", act.render()[0]));

            loop {
                match rx.recv_timeout(interval) {
                    Ok(ActivityCmd::Text(m)) => {
                        act.text(&m);
                        act.tick();
                        writer::write(&format!("\r\x1b[2K{}", act.render()[0]));
                    }
                    Ok(ActivityCmd::End {
                        icon,
                        msg,
                        color,
                        reply,
                    }) => {
                        let frozen = act.freeze(&icon, msg.as_deref(), Some(color));
                        let frozen_line = frozen.into_iter().next().unwrap_or_default();
                        writer::write(&format!("\r\x1b[2K{}\n", frozen_line));
                        let _ = reply.send(frozen_line);
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if stopped_thread.load(Ordering::Relaxed) {
                            break;
                        }
                        act.tick();
                        writer::write(&format!("\r\x1b[2K{}", act.render()[0]));
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        }
    });

    Activity {
        inner: ActivityInner::Tty(TtyActivity {
            tx,
            stopped,
            handle: Some(handle),
            footer_on_end,
        }),
    }
}

impl Activity {
    /// Update the displayed text.
    pub fn text(&self, msg: &str) {
        match &self.inner {
            ActivityInner::Pipe(p) => {
                // Non-TTY: just print the new message
                writer::writeln(msg);
                // Note: can't update p.msg because we only have &self.
                // For pipe mode the msg is only used in end methods.
                let _ = &p.msg;
            }
            ActivityInner::Tty(t) => {
                let _ = t.tx.send(ActivityCmd::Text(msg.to_string()));
            }
        }
    }

    /// Finish with a success icon.
    pub fn done(self, msg: Option<&str>) {
        self.end("\u{2713}", msg, green); // ✓
    }

    /// Finish with a failure icon.
    pub fn fail(self, msg: Option<&str>) {
        self.end("\u{2717}", msg, red); // ✗
    }

    /// Finish with a warning icon.
    pub fn warn(self, msg: Option<&str>) {
        self.end("\u{26A0}", msg, yellow); // ⚠
    }

    /// Finish with an info icon.
    pub fn info(self, msg: Option<&str>) {
        self.end("\u{2139}", msg, blue); // ℹ
    }

    /// Finish with a custom icon and message.
    pub fn stop(self, icon: &str, msg: &str, color: Option<fn(&str) -> String>) {
        self.end(icon, Some(msg), color.unwrap_or(white));
    }

    fn end(self, icon: &str, msg: Option<&str>, color: fn(&str) -> String) {
        match self.inner {
            ActivityInner::Pipe(p) => {
                let display = msg.unwrap_or(&p.msg);
                writer::writeln(&format!("{} {}", color(icon), display));
            }
            ActivityInner::Tty(mut t) => {
                // Send end command and wait for the thread to finish
                let (reply_tx, reply_rx) = mpsc::channel();
                let _ = t.tx.send(ActivityCmd::End {
                    icon: icon.to_string(),
                    msg: msg.map(|s| s.to_string()),
                    color,
                    reply: reply_tx,
                });
                // Wait for thread to finish rendering
                let _ = reply_rx.recv();
                t.stopped.store(true, Ordering::Relaxed);
                if let Some(h) = t.handle.take() {
                    let _ = h.join();
                }
                if let Some(on_end) = t.footer_on_end {
                    on_end();
                }
                show_cursor();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Section — multi-line animated section with items
// ---------------------------------------------------------------------------

/// Options for creating a [`Section`].
pub struct SectionOptions {
    pub spinner: &'static str,
    pub color: Option<fn(&str) -> String>,
    pub indent: usize,
    pub connector: String,
    pub timer: bool,
    pub collapse_on_done: bool,
    pub footer: Option<FooterConfig>,
    pub tty: Option<bool>,
}

impl Default for SectionOptions {
    fn default() -> Self {
        Self {
            spinner: "dots",
            color: None,
            indent: 2,
            connector: "\u{23BF}".to_string(), // ⎿
            timer: false,
            collapse_on_done: false,
            footer: None,
            tty: None,
        }
    }
}

/// A live section handle. Methods are safe to call from the main thread.
pub struct Section {
    inner: SectionInner,
}

enum SectionInner {
    Pipe(PipeSection),
    Tty(TtySection),
}

struct PipeSection {
    title: String,
    pad: String,
}

/// Commands sent from the main thread to the section animation thread.
enum SectionCmd {
    Title(String),
    Add(String),
    Body(String),
    End {
        icon: String,
        msg: Option<String>,
        color: fn(&str) -> String,
        reply: mpsc::Sender<String>,
    },
}

struct TtySection {
    tx: mpsc::Sender<SectionCmd>,
    stopped: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    footer_on_end: Option<Box<dyn FnOnce() + Send>>,
}

/// Create a live section (multi-line animated status with title and items).
///
/// Returns a [`Section`] handle. In TTY mode, a background thread drives
/// the spinner animation. In non-TTY mode, writes static lines.
pub fn section(title: &str, options: SectionOptions) -> Section {
    let tty_mode = options.tty.unwrap_or_else(writer::is_tty);

    if !tty_mode {
        let pad = " ".repeat(options.indent);
        writer::writeln(&format!("{}{}", pad, title));
        return Section {
            inner: SectionInner::Pipe(PipeSection {
                title: title.to_string(),
                pad,
            }),
        };
    }

    // TTY mode: spawn animation thread
    let (tx, rx) = mpsc::channel::<SectionCmd>();
    let stopped = Arc::new(AtomicBool::new(false));
    let stopped_thread = Arc::clone(&stopped);

    let title_owned = title.to_string();
    let spinner = options.spinner;
    let color = options.color;
    let indent = options.indent;
    let connector = options.connector;
    let timer = options.timer;
    let collapse_on_done = options.collapse_on_done;

    let (footer_render, footer_on_end) = match options.footer {
        Some(fc) => (Some(fc.render), Some(fc.on_end)),
        None => (None, None),
    };

    let footer_render: Option<Arc<dyn Fn() -> Vec<String> + Send + Sync>> =
        footer_render.map(|r| Arc::from(r) as Arc<dyn Fn() -> Vec<String> + Send + Sync>);
    let footer_render_thread = footer_render.clone();

    hide_cursor();

    let handle = thread::spawn(move || {
        use std::cell::RefCell;
        use std::rc::Rc;

        let sec = SectionBlock::new(
            &title_owned,
            SectionBlockOptions {
                spinner,
                color,
                indent,
                connector,
                timer,
                collapse_on_done,
            },
        );

        let interval = Duration::from_millis(sec.interval_ms());
        let sec_cell = Rc::new(RefCell::new(sec));
        let sec_render = Rc::clone(&sec_cell);

        let footer_render_ref = footer_render_thread;

        let mut block = live_block(LiveBlockOptions {
            render: Box::new(move || {
                let sec = sec_render.borrow();
                let content_lines = sec.render();
                if let Some(ref fr) = footer_render_ref {
                    let footer_lines = (fr)();
                    let cursor_row = content_lines.len().saturating_sub(1) as u16;
                    let mut lines = content_lines;
                    lines.extend(footer_lines);
                    BlockRender {
                        lines,
                        cursor: Some((cursor_row, 0)),
                    }
                } else {
                    BlockRender {
                        lines: content_lines,
                        cursor: None,
                    }
                }
            }),
            on_close: None,
            tty: Some(true),
        });

        block.update();

        loop {
            match rx.recv_timeout(interval) {
                Ok(SectionCmd::Title(m)) => {
                    sec_cell.borrow_mut().title(&m);
                    block.update();
                }
                Ok(SectionCmd::Add(line)) => {
                    sec_cell.borrow_mut().add(&line);
                    block.update();
                }
                Ok(SectionCmd::Body(content)) => {
                    sec_cell.borrow_mut().body(&content);
                    block.update();
                }
                Ok(SectionCmd::End {
                    icon,
                    msg,
                    color,
                    reply,
                }) => {
                    let mut sec = sec_cell.borrow_mut();
                    let frozen = sec.freeze(&icon, msg.as_deref(), Some(color));
                    let frozen_text = frozen.join("\n");
                    block.close(Some(&frozen_text));
                    let _ = reply.send(frozen_text);
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if stopped_thread.load(Ordering::Relaxed) {
                        break;
                    }
                    sec_cell.borrow_mut().tick();
                    block.update();
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    Section {
        inner: SectionInner::Tty(TtySection {
            tx,
            stopped,
            handle: Some(handle),
            footer_on_end,
        }),
    }
}

impl Section {
    /// Update the section title.
    pub fn title(&self, msg: &str) {
        match &self.inner {
            SectionInner::Pipe(p) => {
                writer::writeln(&format!("{}{}", p.pad, msg));
            }
            SectionInner::Tty(t) => {
                let _ = t.tx.send(SectionCmd::Title(msg.to_string()));
            }
        }
    }

    /// Alias for [`title`](Section::title).
    pub fn text(&self, msg: &str) {
        self.title(msg);
    }

    /// Add a line item below the title.
    pub fn add(&self, line: &str) {
        match &self.inner {
            SectionInner::Pipe(p) => {
                writer::writeln(&format!("{}\u{23BF}  {}", p.pad, line));
            }
            SectionInner::Tty(t) => {
                let _ = t.tx.send(SectionCmd::Add(line.to_string()));
            }
        }
    }

    /// Replace all body items with the given content (split on newlines).
    pub fn body(&self, content: &str) {
        match &self.inner {
            SectionInner::Pipe(p) => {
                for l in content.split('\n') {
                    writer::writeln(&format!("{}\u{23BF}  {}", p.pad, l));
                }
            }
            SectionInner::Tty(t) => {
                let _ = t.tx.send(SectionCmd::Body(content.to_string()));
            }
        }
    }

    /// Finish with a success icon.
    pub fn done(self, msg: Option<&str>) {
        self.end("\u{2713}", msg, green);
    }

    /// Finish with a failure icon.
    pub fn fail(self, msg: Option<&str>) {
        self.end("\u{2717}", msg, red);
    }

    /// Finish with a warning icon.
    pub fn warn(self, msg: Option<&str>) {
        self.end("\u{26A0}", msg, yellow);
    }

    /// Finish with an info icon.
    pub fn info(self, msg: Option<&str>) {
        self.end("\u{2139}", msg, blue);
    }

    /// Finish with a custom icon and message.
    pub fn stop(self, icon: &str, msg: &str, color: Option<fn(&str) -> String>) {
        self.end(icon, Some(msg), color.unwrap_or(white));
    }

    fn end(self, icon: &str, msg: Option<&str>, color: fn(&str) -> String) {
        match self.inner {
            SectionInner::Pipe(p) => {
                let display = msg.unwrap_or(&p.title);
                writer::writeln(&format!("{}{} {}", p.pad, color(icon), display));
            }
            SectionInner::Tty(mut t) => {
                let (reply_tx, reply_rx) = mpsc::channel();
                let _ = t.tx.send(SectionCmd::End {
                    icon: icon.to_string(),
                    msg: msg.map(|s| s.to_string()),
                    color,
                    reply: reply_tx,
                });
                let _ = reply_rx.recv();
                t.stopped.store(true, Ordering::Relaxed);
                if let Some(h) = t.handle.take() {
                    let _ = h.join();
                }
                if let Some(on_end) = t.footer_on_end {
                    on_end();
                }
                show_cursor();
            }
        }
    }
}
