use crate::elapsed::Elapsed;
use crate::spinner::get_spinner;
use crate::style::s;

/// Icon specification: either a named spinner style or custom static frames.
pub enum Icon {
    /// A spinner name (e.g. "dots", "star") looked up via `get_spinner()`
    Spinner(&'static str),
    /// A single static icon string (no animation)
    Static(String),
    /// Custom animation frames
    Frames(Vec<String>),
}

/// Options for creating an ActivityLine.
#[derive(Default)]
pub struct ActivityLineOptions {
    pub icon: Option<Icon>,
    pub interval_ms: Option<u64>,
    pub color: Option<fn(&str) -> String>,
    pub timer: bool,
    pub metrics: Option<Box<dyn Fn() -> String>>,
}

/// Pure state machine for an animated activity line.
/// No I/O — caller drives animation by calling `tick()` on an interval.
pub struct ActivityLine {
    frames: Vec<String>,
    interval_ms: u64,
    color_fn: fn(&str) -> String,
    idx: usize,
    msg: String,
    elapsed: Option<Elapsed>,
    metrics: Option<Box<dyn Fn() -> String>>,
}

fn default_color(text: &str) -> String {
    s().cyan().render(text)
}

fn white_color(text: &str) -> String {
    s().white().render(text)
}

impl ActivityLine {
    pub fn new(text: &str, options: ActivityLineOptions) -> Self {
        let color_fn = options.color.unwrap_or(default_color);

        // Resolve spinner definition and frames
        let (frames, base_interval) = match options.icon {
            None => {
                // Default: dots spinner
                let def = get_spinner("dots").unwrap();
                (
                    def.frames.iter().map(|f| f.to_string()).collect::<Vec<_>>(),
                    def.interval_ms,
                )
            }
            Some(Icon::Spinner(name)) => {
                let def = get_spinner(name).unwrap_or_else(|| get_spinner("dots").unwrap());
                (
                    def.frames.iter().map(|f| f.to_string()).collect::<Vec<_>>(),
                    def.interval_ms,
                )
            }
            Some(Icon::Static(icon)) => (vec![icon], 80),
            Some(Icon::Frames(f)) => (f, 80),
        };

        let interval_ms = options.interval_ms.unwrap_or(base_interval);
        let elapsed = if options.timer {
            Some(Elapsed::new())
        } else {
            None
        };

        Self {
            frames,
            interval_ms,
            color_fn,
            idx: 0,
            msg: text.to_string(),
            elapsed,
            metrics: options.metrics,
        }
    }

    /// The tick interval in milliseconds (caller uses this to drive the timer loop).
    pub fn interval_ms(&self) -> u64 {
        self.interval_ms
    }

    /// Update the displayed text.
    pub fn text(&mut self, msg: &str) {
        self.msg = msg.to_string();
    }

    /// Advance the spinner frame by one.
    pub fn tick(&mut self) {
        self.idx += 1;
    }

    /// Render current state as lines.
    pub fn render(&self) -> Vec<String> {
        vec![self.build_line()]
    }

    /// Freeze the line with a final icon. Returns the frozen lines.
    ///
    /// - `icon`: the icon string to display (e.g. "✓")
    /// - `msg`: optional replacement message
    /// - `color`: optional color function for the icon (defaults to white)
    pub fn freeze(
        &mut self,
        icon: &str,
        msg: Option<&str>,
        color: Option<fn(&str) -> String>,
    ) -> Vec<String> {
        if let Some(m) = msg {
            self.msg = m.to_string();
        }
        let icon_color = color.unwrap_or(white_color);
        vec![self.build_frozen(icon, icon_color)]
    }

    fn build_line(&self) -> String {
        let frame = (self.color_fn)(&self.frames[self.idx % self.frames.len()]);
        let meta_str = self.build_meta(true);
        format!("{} {}{}", frame, self.msg, meta_str)
    }

    fn build_frozen(&self, icon: &str, icon_color: fn(&str) -> String) -> String {
        let meta_str = self.build_meta(false);
        format!("{} {}{}", icon_color(icon), self.msg, meta_str)
    }

    fn build_meta(&self, include_metrics: bool) -> String {
        let mut meta = Vec::new();
        if let Some(ref el) = self.elapsed {
            meta.push(el.render());
        }
        if include_metrics {
            if let Some(ref m) = self.metrics {
                meta.push(m());
            }
        }
        if meta.is_empty() {
            String::new()
        } else {
            s().dim().render(&format!(" ({})", meta.join(" \u{00b7} ")))
        }
    }
}
