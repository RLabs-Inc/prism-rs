use crate::elapsed::Elapsed;
use crate::spinner::get_spinner;
use crate::style::s;

/// Options for creating a SectionBlock.
pub struct SectionBlockOptions {
    pub spinner: &'static str,
    pub color: Option<fn(&str) -> String>,
    pub indent: usize,
    pub connector: String,
    pub timer: bool,
    pub collapse_on_done: bool,
}

impl Default for SectionBlockOptions {
    fn default() -> Self {
        Self {
            spinner: "dots",
            color: None,
            indent: 2,
            connector: "\u{23BF}".to_string(), // ⎿
            timer: false,
            collapse_on_done: false,
        }
    }
}

fn default_color(text: &str) -> String {
    s().cyan().render(text)
}

fn white_color(text: &str) -> String {
    s().white().render(text)
}

/// Pure state machine for a section block with title, spinner, and body items.
/// No I/O — caller drives animation by calling `tick()` on an interval.
pub struct SectionBlock {
    frames: Vec<&'static str>,
    interval_ms: u64,
    color_fn: fn(&str) -> String,
    idx: usize,
    msg: String,
    items: Vec<String>,
    pad: String,
    connector: String,
    elapsed: Option<Elapsed>,
    collapse_on_done: bool,
}

impl SectionBlock {
    pub fn new(title: &str, options: SectionBlockOptions) -> Self {
        let def = get_spinner(options.spinner).unwrap_or_else(|| get_spinner("dots").unwrap());
        let color_fn = options.color.unwrap_or(default_color);
        let elapsed = if options.timer {
            Some(Elapsed::new())
        } else {
            None
        };

        Self {
            frames: def.frames.to_vec(),
            interval_ms: def.interval_ms,
            color_fn,
            idx: 0,
            msg: title.to_string(),
            items: Vec::new(),
            pad: " ".repeat(options.indent),
            connector: options.connector,
            elapsed,
            collapse_on_done: options.collapse_on_done,
        }
    }

    /// The tick interval in milliseconds.
    pub fn interval_ms(&self) -> u64 {
        self.interval_ms
    }

    /// Update the title.
    pub fn title(&mut self, msg: &str) {
        self.msg = msg.to_string();
    }

    /// Alias for title — update the displayed text.
    pub fn text(&mut self, msg: &str) {
        self.msg = msg.to_string();
    }

    /// Add a line item below the title.
    pub fn add(&mut self, line: &str) {
        self.items.push(line.to_string());
    }

    /// Replace all body items. Empty string clears items.
    pub fn body(&mut self, content: &str) {
        if content.is_empty() {
            self.items.clear();
        } else {
            self.items = content.split('\n').map(|s| s.to_string()).collect();
        }
    }

    /// Advance the spinner frame by one.
    pub fn tick(&mut self) {
        self.idx += 1;
    }

    /// Render current state as lines.
    pub fn render(&self) -> Vec<String> {
        self.build_lines(None, None)
    }

    /// Freeze the block with a final icon. Returns the frozen lines.
    ///
    /// - `icon`: the icon string to display
    /// - `msg`: optional replacement message for the title
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
        let timer_str = self.timer_str();

        let mut lines = Vec::new();
        lines.push(format!(
            "{}{} {}{}",
            self.pad,
            icon_color(icon),
            self.msg,
            timer_str,
        ));

        if !self.collapse_on_done {
            for item in &self.items {
                lines.push(format!(
                    "{}{}  {}",
                    self.pad,
                    s().dim().render(&self.connector),
                    item,
                ));
            }
        }

        lines
    }

    fn build_lines(
        &self,
        final_icon: Option<&str>,
        icon_color: Option<fn(&str) -> String>,
    ) -> Vec<String> {
        let mut lines = Vec::new();
        let icon_str = match final_icon {
            Some(icon) => {
                let color = icon_color.unwrap_or(white_color);
                color(icon)
            }
            None => (self.color_fn)(self.frames[self.idx % self.frames.len()]),
        };
        let timer_str = self.timer_str();
        lines.push(format!(
            "{}{} {}{}",
            self.pad, icon_str, self.msg, timer_str
        ));
        for item in &self.items {
            lines.push(format!(
                "{}{}  {}",
                self.pad,
                s().dim().render(&self.connector),
                item,
            ));
        }
        lines
    }

    fn timer_str(&self) -> String {
        match &self.elapsed {
            Some(el) => s().dim().render(&format!(" {}", el.render())),
            None => String::new(),
        }
    }
}
