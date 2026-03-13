// prism/exec — live command output viewer
// controlled component: consumer drives rendering and forwards events
// lifecycle: create → write chunks → scroll → done/fail → freeze to scrollback
//
// renders as a bordered box with scrollable output:
//   ╭─ bash ──────────────────────╮
//   │ $ nmap -sV target.com       │
//   │ Starting Nmap 7.94...       │
//   │ Discovered open port 80/tcp │
//   ╰───────── 12s · running ─────╯

use crate::elapsed::Elapsed;
use crate::frame::BorderStyle;
use crate::style::{s, Style};
use crate::text::truncate;
use crate::writer::term_width;

/// Options for creating an exec viewer
pub struct ExecOptions {
    /// Max visible output lines (default: 16)
    pub max_height: usize,
    /// Border style (default: Rounded)
    pub border: BorderStyle,
    /// Border color function (default: dim)
    pub border_color: Box<dyn Fn(&str) -> String>,
    /// Title shown in header border (default: "bash")
    pub title: String,
    /// Title color function (default: cyan)
    pub title_color: Box<dyn Fn(&str) -> String>,
    /// Show elapsed time in footer (default: true)
    pub timer: bool,
    /// Override terminal width
    pub width: Option<u16>,
}

impl Default for ExecOptions {
    fn default() -> Self {
        Self {
            max_height: 16,
            border: BorderStyle::Rounded,
            border_color: Box::new(|t| Style::new().dim().paint(t)),
            title: "bash".into(),
            title_color: Box::new(|t| Style::new().cyan().paint(t)),
            timer: true,
            width: None,
        }
    }
}

/// Process carriage returns within a line: last segment after \r wins
fn process_cr(line: &str) -> String {
    if !line.contains('\r') {
        return line.to_string();
    }
    line.rsplit('\r').next().unwrap_or("").to_string()
}

/// Live command output viewer with bordered scrollable box
pub struct Exec {
    command: String,
    max_height: usize,
    border: BorderStyle,
    border_color: Box<dyn Fn(&str) -> String>,
    title: String,
    title_color: Box<dyn Fn(&str) -> String>,
    timer_enabled: bool,
    width_override: Option<u16>,
    elapsed: Elapsed,

    lines: Vec<String>,
    partial: String,
    scroll_pos: usize,
    user_scrolled: bool,
    stopped: bool,
    exit_code: Option<i32>,
    error_msg: Option<String>,
}

const MAX_LINES: usize = 10000;

impl Exec {
    pub fn new(command: &str, options: ExecOptions) -> Self {
        Self {
            command: command.to_string(),
            max_height: options.max_height,
            border: options.border,
            border_color: options.border_color,
            title: options.title,
            title_color: options.title_color,
            timer_enabled: options.timer,
            width_override: options.width,
            elapsed: Elapsed::new(),

            lines: Vec::new(),
            partial: String::new(),
            scroll_pos: 0,
            user_scrolled: false,
            stopped: false,
            exit_code: None,
            error_msg: None,
        }
    }

    fn all_lines(&self) -> Vec<&str> {
        let mut all: Vec<&str> = self.lines.iter().map(|s| s.as_str()).collect();
        if !self.partial.is_empty() {
            // We can't push a processed CR line here without allocation,
            // just show partial as-is for display
            all.push(&self.partial);
        }
        all
    }

    fn max_scroll(&self) -> usize {
        let total = self.all_lines().len();
        total.saturating_sub(self.max_height)
    }

    fn box_width(&self) -> usize {
        self.width_override.unwrap_or_else(term_width) as usize
    }

    fn border_chars(&self) -> (&str, &str, &str, &str, &str, &str) {
        match self.border {
            BorderStyle::Rounded => ("╭", "╮", "╰", "╯", "─", "│"),
            BorderStyle::Heavy => ("┏", "┓", "┗", "┛", "━", "┃"),
            BorderStyle::Double => ("╔", "╗", "╚", "╝", "═", "║"),
            BorderStyle::Single => ("┌", "┐", "└", "┘", "─", "│"),
        }
    }

    fn render_header(&self, width: usize) -> String {
        let (tl, tr, _, _, h, _) = self.border_chars();
        let styled_title = (self.title_color)(&format!(" {} ", self.title));
        let title_display_width = self.title.len() + 2; // " title "
        let remaining = width.saturating_sub(2 + title_display_width);
        format!(
            "{}{}{}",
            (self.border_color)(&format!("{}{}", tl, h)),
            styled_title,
            (self.border_color)(&format!("{}{}", h.repeat(remaining.saturating_sub(1)), tr))
        )
    }

    fn render_content_line(&self, text: &str, inner_width: usize) -> String {
        let (_, _, _, _, _, v) = self.border_chars();
        let truncated = truncate(text, inner_width, "…");
        let display_width = crate::ansi::measure_width(&truncated);
        let right_pad = inner_width.saturating_sub(display_width);
        format!(
            "{} {}{} {}",
            (self.border_color)(v),
            truncated,
            " ".repeat(right_pad),
            (self.border_color)(v)
        )
    }

    fn render_footer(&self, width: usize) -> String {
        let (_, _, bl, br, h, _) = self.border_chars();
        let all = self.all_lines();

        // Status text
        let status_text = if self.stopped {
            if let Some(ref err) = self.error_msg {
                format!("{} {}", s().red().paint("✗"), err)
            } else {
                let code = self.exit_code.unwrap_or(0);
                let icon = if code == 0 {
                    s().green().paint("✓")
                } else {
                    s().red().paint("✗")
                };
                let mut parts = vec![icon];
                if self.timer_enabled {
                    parts.push(s().dim().paint(&self.elapsed.render()));
                }
                let exit_str = if code == 0 {
                    s().green().paint(&format!("exit {}", code))
                } else {
                    s().red().paint(&format!("exit {}", code))
                };
                parts.push(exit_str);
                parts.join(&s().dim().paint(" · "))
            }
        } else {
            let mut parts = Vec::new();
            if self.timer_enabled {
                parts.push(self.elapsed.render());
            }
            parts.push("running".into());
            s().dim().paint(&parts.join(" · "))
        };

        // Scroll indicator
        let mut scroll_str = String::new();
        let mut scroll_width = 0;
        if all.len() > self.max_height {
            let from = self.scroll_pos + 1;
            let to = (self.scroll_pos + self.max_height).min(all.len());
            scroll_str = format!(
                " {} ",
                s().dim().paint(&format!("{}-{}/{}", from, to, all.len()))
            );
            scroll_width = format!(" {}-{}/{} ", from, to, all.len()).len();
        }

        let content_width = width.saturating_sub(2);
        let status_display = format!(" {} ", status_text);
        let status_width = crate::ansi::measure_width(&crate::ansi::strip_ansi(&status_display));
        let fill = content_width.saturating_sub(scroll_width + status_width);

        format!(
            "{}{}{}{}{}",
            (self.border_color)(bl),
            scroll_str,
            (self.border_color)(&h.repeat(fill)),
            status_display,
            (self.border_color)(br)
        )
    }

    /// Append streaming data from command output
    pub fn write(&mut self, data: &str) {
        if self.stopped || data.is_empty() {
            return;
        }

        let normalized = data.replace("\r\n", "\n");
        self.partial.push_str(&normalized);

        let segments: Vec<&str> = self.partial.split('\n').collect();
        let last = segments.last().unwrap().to_string();
        for seg in &segments[..segments.len() - 1] {
            self.lines.push(process_cr(seg));
        }
        self.partial = last;

        // Cap line buffer
        if self.lines.len() > MAX_LINES {
            let excess = self.lines.len() - MAX_LINES;
            self.lines.drain(..excess);
        }

        // Auto-scroll
        if !self.user_scrolled {
            self.scroll_pos = self.max_scroll();
        }
    }

    /// Scroll the visible window: +N down, -N up
    pub fn scroll(&mut self, delta: i32) {
        if self.all_lines().len() <= self.max_height {
            return;
        }
        self.user_scrolled = true;
        let new_pos = self.scroll_pos as i32 + delta;
        self.scroll_pos = new_pos.max(0).min(self.max_scroll() as i32) as usize;
    }

    /// Mark command as complete with exit code
    pub fn done(&mut self, exit_code: i32) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        self.exit_code = Some(exit_code);

        if !self.partial.is_empty() {
            let line = process_cr(&std::mem::take(&mut self.partial));
            self.lines.push(line);
        }

        self.scroll_pos = self.max_scroll();
        self.user_scrolled = false;
    }

    /// Mark command as failed with error message
    pub fn fail(&mut self, error: &str) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        self.error_msg = Some(error.to_string());

        if !self.partial.is_empty() {
            let line = process_cr(&std::mem::take(&mut self.partial));
            self.lines.push(line);
        }

        self.scroll_pos = self.max_scroll();
        self.user_scrolled = false;
    }

    /// Render current state as lines for embedding in an active zone
    pub fn render(&self) -> Vec<String> {
        let width = self.box_width();
        let inner_width = width.saturating_sub(4); // 2 borders + 2 padding
        let mut result = Vec::new();

        // Header
        result.push(self.render_header(width));

        // Command line (always visible)
        let cmd_display = format!("{} {}", s().green().paint("$"), self.command);
        result.push(self.render_content_line(&cmd_display, inner_width));

        // Output lines (scrollable window)
        let all = self.all_lines();
        if !all.is_empty() {
            let end = (self.scroll_pos + self.max_height).min(all.len());
            for line in &all[self.scroll_pos..end] {
                result.push(self.render_content_line(line, inner_width));
            }
        }

        // Footer
        result.push(self.render_footer(width));
        result
    }

    /// Render full output as lines for freezing to scrollback
    pub fn freeze(&self) -> Vec<String> {
        let width = self.box_width();
        let inner_width = width.saturating_sub(4);
        let mut result = Vec::new();

        result.push(self.render_header(width));

        let cmd_display = format!("{} {}", s().green().paint("$"), self.command);
        result.push(self.render_content_line(&cmd_display, inner_width));

        for line in &self.lines {
            result.push(self.render_content_line(line, inner_width));
        }

        result.push(self.render_footer(width));
        result
    }

    /// Whether the command is still running
    pub fn running(&self) -> bool {
        !self.stopped
    }

    /// Whether there's more content than max_height
    pub fn scrollable(&self) -> bool {
        self.all_lines().len() > self.max_height
    }

    /// Current scroll offset (0 = top)
    pub fn scroll_offset(&self) -> usize {
        self.scroll_pos
    }

    /// Total number of output lines
    pub fn line_count(&self) -> usize {
        self.all_lines().len()
    }
}
