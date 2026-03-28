// prism/spinner - animated inline loaders and spinners
// 44 animations from classic braille dots to creative art
// inline by design: animates on current line, completes with icon + message
// pipe-aware: degrades to static text when not a TTY

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::activity_line::{ActivityLine, ActivityLineOptions, Icon};
use crate::block::{live_block, BlockRender, LiveBlockOptions};
use crate::cursor;
use crate::style::s;
use crate::writer;

// ─── Spinner catalog ────────────────────────────────────────────────────

/// A spinner animation definition
#[derive(Debug, Clone)]
pub struct SpinnerDef {
    pub frames: &'static [&'static str],
    pub interval_ms: u64,
}

// Define all 44 spinners as static constants
const DOTS: SpinnerDef = SpinnerDef {
    frames: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
    interval_ms: 80,
};

const DOTS2: SpinnerDef = SpinnerDef {
    frames: &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
    interval_ms: 80,
};

const DOTS3: SpinnerDef = SpinnerDef {
    frames: &["⠋", "⠙", "⠚", "⠞", "⠖", "⠦", "⠴", "⠲", "⠳", "⠓"],
    interval_ms: 80,
};

const DOTS4: SpinnerDef = SpinnerDef {
    frames: &["⠄", "⠆", "⠇", "⠋", "⠙", "⠸", "⠰", "⠠", "⠐", "⠈"],
    interval_ms: 80,
};

const LINE: SpinnerDef = SpinnerDef {
    frames: &["-", "\\", "|", "/"],
    interval_ms: 130,
};

const PIPE: SpinnerDef = SpinnerDef {
    frames: &["┤", "┘", "┴", "└", "├", "┌", "┬", "┐"],
    interval_ms: 100,
};

const SIMPLE_DOTS: SpinnerDef = SpinnerDef {
    frames: &[".  ", ".. ", "...", "   "],
    interval_ms: 400,
};

const STAR: SpinnerDef = SpinnerDef {
    frames: &["✶", "✸", "✹", "✺", "✹", "✸"],
    interval_ms: 100,
};

const SPARK: SpinnerDef = SpinnerDef {
    frames: &["·", "✦", "✧", "✦"],
    interval_ms: 150,
};

const ARC: SpinnerDef = SpinnerDef {
    frames: &["◜", "◠", "◝", "◞", "◡", "◟"],
    interval_ms: 100,
};

const CIRCLE: SpinnerDef = SpinnerDef {
    frames: &["◐", "◓", "◑", "◒"],
    interval_ms: 120,
};

const SQUARE_SPIN: SpinnerDef = SpinnerDef {
    frames: &["◰", "◳", "◲", "◱"],
    interval_ms: 120,
};

const TRIANGLES: SpinnerDef = SpinnerDef {
    frames: &["◢", "◣", "◤", "◥"],
    interval_ms: 120,
};

const SECTORS: SpinnerDef = SpinnerDef {
    frames: &["◴", "◷", "◶", "◵"],
    interval_ms: 120,
};

const DIAMOND: SpinnerDef = SpinnerDef {
    frames: &["◇", "◈", "◆", "◈"],
    interval_ms: 200,
};

const TOGGLE: SpinnerDef = SpinnerDef {
    frames: &["▪", "▫"],
    interval_ms: 300,
};

const TOGGLE2: SpinnerDef = SpinnerDef {
    frames: &["◼", "◻"],
    interval_ms: 300,
};

const BLOCKS: SpinnerDef = SpinnerDef {
    frames: &["░", "▒", "▓", "█", "▓", "▒"],
    interval_ms: 100,
};

const BLOCKS2: SpinnerDef = SpinnerDef {
    frames: &["▖", "▘", "▝", "▗"],
    interval_ms: 100,
};

const BLOCKS3: SpinnerDef = SpinnerDef {
    frames: &["▌", "▀", "▐", "▄"],
    interval_ms: 100,
};

const PULSE: SpinnerDef = SpinnerDef {
    frames: &["·", "•", "●", "•"],
    interval_ms: 150,
};

const PULSE2: SpinnerDef = SpinnerDef {
    frames: &["○", "◎", "●", "◎"],
    interval_ms: 150,
};

const BREATHE: SpinnerDef = SpinnerDef {
    frames: &["  ∙  ", " ∙∙∙ ", "∙∙∙∙∙", " ∙∙∙ "],
    interval_ms: 200,
};

const HEARTBEAT: SpinnerDef = SpinnerDef {
    frames: &["♡", "♡", "♥", "♥", "♡", "♡", " ", " "],
    interval_ms: 150,
};

const GROWING: SpinnerDef = SpinnerDef {
    frames: &[
        "▏", "▎", "▍", "▌", "▋", "▊", "▉", "█", "▉", "▊", "▋", "▌", "▍", "▎",
    ],
    interval_ms: 80,
};

const BOUNCE: SpinnerDef = SpinnerDef {
    frames: &["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"],
    interval_ms: 120,
};

const BOUNCING_BAR: SpinnerDef = SpinnerDef {
    frames: &[
        "[    =     ]",
        "[   =      ]",
        "[  =       ]",
        "[ =        ]",
        "[=         ]",
        "[ =        ]",
        "[  =       ]",
        "[   =      ]",
        "[    =     ]",
        "[     =    ]",
        "[      =   ]",
        "[       =  ]",
        "[        = ]",
        "[         =]",
        "[        = ]",
        "[       =  ]",
        "[      =   ]",
        "[     =    ]",
    ],
    interval_ms: 80,
};

const BOUNCING_BALL: SpinnerDef = SpinnerDef {
    frames: &[
        "( ●    )",
        "(  ●   )",
        "(   ●  )",
        "(    ● )",
        "(     ●)",
        "(    ● )",
        "(   ●  )",
        "(  ●   )",
        "( ●    )",
        "(●     )",
    ],
    interval_ms: 80,
};

const ARROWS: SpinnerDef = SpinnerDef {
    frames: &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
    interval_ms: 120,
};

const ARROW_PULSE: SpinnerDef = SpinnerDef {
    frames: &["▹▹▹▹▹", "►▹▹▹▹", "▹►▹▹▹", "▹▹►▹▹", "▹▹▹►▹", "▹▹▹▹►"],
    interval_ms: 120,
};

const WAVE: SpinnerDef = SpinnerDef {
    frames: &[
        "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█", "▇", "▆", "▅", "▄", "▃", "▂",
    ],
    interval_ms: 80,
};

const WAVE2: SpinnerDef = SpinnerDef {
    frames: &[
        "▁▂▃",
        "▂▃▄",
        "▃▄▅",
        "▄▅▆",
        "▅▆▇",
        "▆▇█",
        "▇█▇",
        "█▇▆",
        "▇▆▅",
        "▆▅▄",
        "▅▄▃",
        "▄▃▂",
        "▃▂▁",
    ],
    interval_ms: 80,
};

const AESTHETIC: SpinnerDef = SpinnerDef {
    frames: &[
        "▱▱▱▱▱",
        "▰▱▱▱▱",
        "▰▰▱▱▱",
        "▰▰▰▱▱",
        "▰▰▰▰▱",
        "▰▰▰▰▰",
        "▱▱▱▱▱",
    ],
    interval_ms: 150,
};

const FILLING: SpinnerDef = SpinnerDef {
    frames: &[
        "□□□□□",
        "■□□□□",
        "■■□□□",
        "■■■□□",
        "■■■■□",
        "■■■■■",
        "□□□□□",
    ],
    interval_ms: 150,
};

const SCANNING: SpinnerDef = SpinnerDef {
    frames: &[
        "░░░░░",
        "▒░░░░",
        "░▒░░░",
        "░░▒░░",
        "░░░▒░",
        "░░░░▒",
        "░░░░░",
    ],
    interval_ms: 100,
};

const BINARY: SpinnerDef = SpinnerDef {
    frames: &["010010", "001101", "100110", "110011", "011001", "101100"],
    interval_ms: 100,
};

const MATRIX: SpinnerDef = SpinnerDef {
    frames: &["Ξ", "Σ", "Φ", "Ψ", "Ω", "λ", "μ", "π"],
    interval_ms: 100,
};

const HACK: SpinnerDef = SpinnerDef {
    frames: &["▓▒░", "▒░▓", "░▓▒"],
    interval_ms: 100,
};

const BRAILLE_SNAKE: SpinnerDef = SpinnerDef {
    frames: &["⠏", "⠛", "⠹", "⢸", "⣰", "⣤", "⣆", "⡇"],
    interval_ms: 100,
};

const BRAILLE_WAVE: SpinnerDef = SpinnerDef {
    frames: &[
        "⠁", "⠂", "⠄", "⡀", "⡈", "⡐", "⡠", "⣀", "⣁", "⣂", "⣄", "⣌", "⣔", "⣤", "⣥", "⣦", "⣮", "⣶",
        "⣷", "⣿", "⡿", "⠿", "⢟", "⠟", "⠏", "⠇", "⠃", "⠁",
    ],
    interval_ms: 60,
};

const ORBIT: SpinnerDef = SpinnerDef {
    frames: &["◯", "◎", "●", "◎"],
    interval_ms: 200,
};

const EARTH: SpinnerDef = SpinnerDef {
    frames: &["🌍", "🌎", "🌏"],
    interval_ms: 300,
};

const MOON: SpinnerDef = SpinnerDef {
    frames: &["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"],
    interval_ms: 200,
};

const CLOCK: SpinnerDef = SpinnerDef {
    frames: &[
        "🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚", "🕛",
    ],
    interval_ms: 150,
};

const HOURGLASS: SpinnerDef = SpinnerDef {
    frames: &["⏳", "⌛"],
    interval_ms: 500,
};

/// Get a spinner by name
pub fn get_spinner(name: &str) -> Option<&'static SpinnerDef> {
    match name {
        "dots" => Some(&DOTS),
        "dots2" => Some(&DOTS2),
        "dots3" => Some(&DOTS3),
        "dots4" => Some(&DOTS4),
        "line" => Some(&LINE),
        "pipe" => Some(&PIPE),
        "simpleDots" => Some(&SIMPLE_DOTS),
        "star" => Some(&STAR),
        "spark" => Some(&SPARK),
        "arc" => Some(&ARC),
        "circle" => Some(&CIRCLE),
        "squareSpin" => Some(&SQUARE_SPIN),
        "triangles" => Some(&TRIANGLES),
        "sectors" => Some(&SECTORS),
        "diamond" => Some(&DIAMOND),
        "toggle" => Some(&TOGGLE),
        "toggle2" => Some(&TOGGLE2),
        "blocks" => Some(&BLOCKS),
        "blocks2" => Some(&BLOCKS2),
        "blocks3" => Some(&BLOCKS3),
        "pulse" => Some(&PULSE),
        "pulse2" => Some(&PULSE2),
        "breathe" => Some(&BREATHE),
        "heartbeat" => Some(&HEARTBEAT),
        "growing" => Some(&GROWING),
        "bounce" => Some(&BOUNCE),
        "bouncingBar" => Some(&BOUNCING_BAR),
        "bouncingBall" => Some(&BOUNCING_BALL),
        "arrows" => Some(&ARROWS),
        "arrowPulse" => Some(&ARROW_PULSE),
        "wave" => Some(&WAVE),
        "wave2" => Some(&WAVE2),
        "aesthetic" => Some(&AESTHETIC),
        "filling" => Some(&FILLING),
        "scanning" => Some(&SCANNING),
        "binary" => Some(&BINARY),
        "matrix" => Some(&MATRIX),
        "hack" => Some(&HACK),
        "brailleSnake" => Some(&BRAILLE_SNAKE),
        "brailleWave" => Some(&BRAILLE_WAVE),
        "orbit" => Some(&ORBIT),
        "earth" => Some(&EARTH),
        "moon" => Some(&MOON),
        "clock" => Some(&CLOCK),
        "hourglass" => Some(&HOURGLASS),
        _ => None,
    }
}

/// List all spinner names
pub fn all_spinner_names() -> Vec<&'static str> {
    vec![
        "dots",
        "dots2",
        "dots3",
        "dots4",
        "line",
        "pipe",
        "simpleDots",
        "star",
        "spark",
        "arc",
        "circle",
        "squareSpin",
        "triangles",
        "sectors",
        "diamond",
        "toggle",
        "toggle2",
        "blocks",
        "blocks2",
        "blocks3",
        "pulse",
        "pulse2",
        "breathe",
        "heartbeat",
        "growing",
        "bounce",
        "bouncingBar",
        "bouncingBall",
        "arrows",
        "arrowPulse",
        "wave",
        "wave2",
        "aesthetic",
        "filling",
        "scanning",
        "binary",
        "matrix",
        "hack",
        "brailleSnake",
        "brailleWave",
        "orbit",
        "earth",
        "moon",
        "clock",
        "hourglass",
    ]
}

// ─── Spinner runtime ────────────────────────────────────────────────────

fn green_color(text: &str) -> String {
    s().green().render(text)
}

fn red_color(text: &str) -> String {
    s().red().render(text)
}

fn yellow_color(text: &str) -> String {
    s().yellow().render(text)
}

fn blue_color(text: &str) -> String {
    s().blue().render(text)
}

fn white_color(text: &str) -> String {
    s().white().render(text)
}

fn dim_color(text: &str) -> String {
    s().dim().render(text)
}

fn cyan_color(text: &str) -> String {
    s().cyan().render(text)
}

/// Options for creating a spinner.
pub struct SpinnerOptions {
    /// Animation style from the catalog (default: "dots")
    pub style: &'static str,
    /// Custom frames — overrides style
    pub frames: Option<Vec<String>>,
    /// Custom interval in ms — overrides style default
    pub interval: Option<u64>,
    /// Spinner frame color function (default: cyan)
    pub color: fn(&str) -> String,
    /// Show elapsed time
    pub timer: bool,
}

impl Default for SpinnerOptions {
    fn default() -> Self {
        Self {
            style: "dots",
            frames: None,
            interval: None,
            color: cyan_color,
            timer: false,
        }
    }
}

/// Command sent from the Spinner handle to the background thread to stop.
struct StopCmd {
    icon: String,
    msg: String,
    color: fn(&str) -> String,
}

/// Shared state between Spinner handle and background thread.
struct SharedState {
    /// Pending text update (main thread writes, bg thread reads and clears).
    text_update: Mutex<Option<String>>,
    /// Stop command (main thread writes, bg thread reads).
    stop_cmd: Mutex<Option<StopCmd>>,
    /// Whether the bg thread is still running.
    running: AtomicBool,
    /// Signal the bg thread to wake up (for immediate stop).
    wake: Condvar,
    /// Mutex paired with wake condvar.
    wake_lock: Mutex<()>,
}

/// A running spinner handle. Call methods to update text or stop with an outcome.
///
/// Dropping a `Spinner` without calling a stop method will automatically stop it
/// with a dim "■" icon (same as abort behavior in the TS version).
pub struct Spinner {
    inner: SpinnerInner,
}

enum SpinnerInner {
    Tty(TtySpinner),
    Pipe(PipeSpinner),
}

struct TtySpinner {
    state: Arc<SharedState>,
    thread: Option<JoinHandle<()>>,
    initial_text: String,
    stopped: bool,
}

struct PipeSpinner {
    msg: String,
}

/// Create and start a spinner animation.
///
/// In TTY mode, spawns a background thread that owns the `ActivityLine` and
/// `LiveBlock`, ticking the animation at the configured interval. The cursor
/// is hidden during animation.
///
/// In non-TTY mode, prints the text as a static line. Stop methods print
/// their icon + message.
pub fn spinner(text: &str, options: SpinnerOptions) -> Spinner {
    // Non-TTY: static text, no animation
    if !writer::is_tty() {
        writer::writeln(text);
        return Spinner {
            inner: SpinnerInner::Pipe(PipeSpinner {
                msg: text.to_string(),
            }),
        };
    }

    // TTY mode: spawn a background thread that owns ActivityLine + LiveBlock

    // Resolve spinner definition
    let def = get_spinner(options.style).unwrap_or_else(|| get_spinner("dots").unwrap());

    // Build icon spec for ActivityLine
    let icon = if let Some(ref custom_frames) = options.frames {
        Icon::Frames(custom_frames.clone())
    } else {
        Icon::Spinner(options.style)
    };

    let interval_ms = options.interval.unwrap_or(def.interval_ms);
    let color_fn = options.color;
    let timer = options.timer;
    let text_owned = text.to_string();

    let shared = Arc::new(SharedState {
        text_update: Mutex::new(None),
        stop_cmd: Mutex::new(None),
        running: AtomicBool::new(true),
        wake: Condvar::new(),
        wake_lock: Mutex::new(()),
    });

    let state_for_thread = Arc::clone(&shared);

    // Hide cursor before spawning (must happen on main thread for immediate effect)
    cursor::hide_cursor();

    let thread = thread::spawn(move || {
        // Create ActivityLine on this thread (it's not Send, so it lives here)
        let mut activity = ActivityLine::new(
            &text_owned,
            ActivityLineOptions {
                icon: Some(icon),
                interval_ms: Some(interval_ms),
                color: Some(color_fn),
                timer,
                metrics: None,
            },
        );

        // Create LiveBlock with render callback that reads from our local ActivityLine.
        // We can't use a closure over `activity` for the render callback since LiveBlock
        // needs FnMut. Instead, we'll manage rendering manually.
        //
        // We'll use a simple approach: store rendered lines in a shared spot that
        // the render callback reads from.
        let rendered = Arc::new(Mutex::new(activity.render()));

        let rendered_for_block = Arc::clone(&rendered);
        let mut block = live_block(LiveBlockOptions {
            render: Box::new(move || BlockRender {
                lines: rendered_for_block.lock().unwrap().clone(),
                cursor: None,
            }),
            on_close: None,
            tty: Some(true),
        });

        // Initial draw
        block.update();

        let tick_duration = Duration::from_millis(interval_ms);

        loop {
            // Wait for tick interval or wake signal
            {
                let guard = state_for_thread.wake_lock.lock().unwrap();
                let _ = state_for_thread
                    .wake
                    .wait_timeout(guard, tick_duration)
                    .unwrap();
            }

            // Check for text update
            {
                let mut text_guard = state_for_thread.text_update.lock().unwrap();
                if let Some(new_text) = text_guard.take() {
                    activity.text(&new_text);
                }
            }

            // Check for stop command
            {
                let stop_guard = state_for_thread.stop_cmd.lock().unwrap();
                if stop_guard.is_some() {
                    break;
                }
            }

            // Check running flag
            if !state_for_thread.running.load(Ordering::SeqCst) {
                break;
            }

            // Tick and re-render
            activity.tick();
            *rendered.lock().unwrap() = activity.render();
            block.update();
        }

        // Process stop command: freeze and close
        let stop = state_for_thread.stop_cmd.lock().unwrap().take();
        if let Some(cmd) = stop {
            let frozen = activity.freeze(&cmd.icon, Some(&cmd.msg), Some(cmd.color));
            block.close(frozen.first().map(|s| s.as_str()));
        } else {
            // Stopped without a command (shouldn't happen, but handle gracefully)
            block.close(None);
        }

        // Show cursor (ref-counted, so safe from any thread)
        cursor::show_cursor();

        state_for_thread.running.store(false, Ordering::SeqCst);
    });

    Spinner {
        inner: SpinnerInner::Tty(TtySpinner {
            state: shared,
            thread: Some(thread),
            initial_text: text.to_string(),
            stopped: false,
        }),
    }
}

impl Spinner {
    /// Update the spinner's display text.
    pub fn text(&self, msg: &str) {
        match &self.inner {
            SpinnerInner::Pipe(_) => {
                writer::writeln(msg);
            }
            SpinnerInner::Tty(tty) => {
                if !tty.stopped {
                    *tty.state.text_update.lock().unwrap() = Some(msg.to_string());
                    // Wake the thread so text update is picked up promptly
                    tty.state.wake.notify_one();
                }
            }
        }
    }

    /// Stop with success: green checkmark.
    pub fn done(self, msg: Option<&str>) {
        self.end("\u{2713}", msg, green_color); // ✓
    }

    /// Stop with failure: red cross.
    pub fn fail(self, msg: Option<&str>) {
        self.end("\u{2717}", msg, red_color); // ✗
    }

    /// Stop with warning: yellow warning sign.
    pub fn warn(self, msg: Option<&str>) {
        self.end("\u{26a0}", msg, yellow_color); // ⚠
    }

    /// Stop with info: blue info symbol.
    pub fn info(self, msg: Option<&str>) {
        self.end("\u{2139}", msg, blue_color); // ℹ
    }

    /// Stop with a custom icon, message, and optional color.
    pub fn stop(self, icon: Option<&str>, msg: Option<&str>, color: Option<fn(&str) -> String>) {
        let i = icon.unwrap_or("\u{25a0}"); // ■
        let c = color.unwrap_or(white_color);
        self.end(i, msg, c);
    }

    fn end(mut self, icon: &str, msg: Option<&str>, color: fn(&str) -> String) {
        self.end_inner(icon, msg, color);
    }

    fn end_inner(&mut self, icon: &str, msg: Option<&str>, color: fn(&str) -> String) {
        match &mut self.inner {
            SpinnerInner::Pipe(pipe) => {
                let final_msg = msg.unwrap_or(&pipe.msg);
                writer::writeln(&format!("{} {}", icon, final_msg));
            }
            SpinnerInner::Tty(tty) => {
                if tty.stopped {
                    return;
                }
                tty.stopped = true;

                let final_msg = msg.unwrap_or(&tty.initial_text);

                // Send stop command to the background thread
                *tty.state.stop_cmd.lock().unwrap() = Some(StopCmd {
                    icon: icon.to_string(),
                    msg: final_msg.to_string(),
                    color,
                });

                // Wake the thread to process the stop immediately
                tty.state.wake.notify_one();

                // Join the thread
                if let Some(handle) = tty.thread.take() {
                    let _ = handle.join();
                }
            }
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        // Auto-stop on drop if not already stopped (abort behavior)
        match &self.inner {
            SpinnerInner::Pipe(_) => {}
            SpinnerInner::Tty(tty) => {
                if !tty.stopped {
                    self.end_inner("\u{25a0}", None, dim_color); // ■ dim
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_spinner_known_name() {
        let def = get_spinner("dots").unwrap();
        assert_eq!(def.frames.len(), 10);
        assert_eq!(def.interval_ms, 80);
    }

    #[test]
    fn test_get_spinner_unknown_returns_none() {
        assert!(get_spinner("nonexistent").is_none());
    }

    #[test]
    fn test_all_spinner_names_count() {
        assert_eq!(all_spinner_names().len(), 45);
    }

    #[test]
    fn test_all_spinner_names_resolve() {
        for name in all_spinner_names() {
            assert!(
                get_spinner(name).is_some(),
                "spinner '{}' in all_spinner_names() but not in get_spinner()",
                name
            );
        }
    }

    #[test]
    fn test_spinner_options_default() {
        let opts = SpinnerOptions::default();
        assert_eq!(opts.style, "dots");
        assert!(opts.frames.is_none());
        assert!(opts.interval.is_none());
        assert!(!opts.timer);
    }

    #[test]
    fn test_spinner_options_custom_frames() {
        let opts = SpinnerOptions {
            frames: Some(vec!["A".to_string(), "B".to_string()]),
            ..Default::default()
        };
        assert_eq!(opts.frames.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_color_helpers_produce_output() {
        // Verify color helper functions return non-empty strings
        assert!(!green_color("x").is_empty());
        assert!(!red_color("x").is_empty());
        assert!(!yellow_color("x").is_empty());
        assert!(!blue_color("x").is_empty());
        assert!(!white_color("x").is_empty());
        assert!(!dim_color("x").is_empty());
        assert!(!cyan_color("x").is_empty());
    }
}
