// prism/input_line — pure state machine for prompted input
// wraps line_editor with prompt rendering and cursor position calculation
// zero I/O — returns { lines, cursor } for liveBlock to render

use crate::ansi::measure_width;
use crate::line_editor::LineEditor;
use crate::style::Style;

/// Rendered output from an input line
pub struct InputLineRender {
    pub lines: Vec<String>,
    pub cursor: (usize, usize), // (row, col)
}

/// Options for creating an input line
pub struct InputLineOptions {
    /// Prompt string or function for dynamic prompts
    pub prompt: PromptSource,
    /// Prompt color function (default: cyan)
    pub prompt_color: Box<dyn Fn(&str) -> String + Send + Sync>,
    /// Shared history (passed to line editor)
    pub history: Vec<String>,
    /// Max history entries
    pub history_size: Option<usize>,
    /// Mask character for sensitive input (e.g., "●")
    pub mask: Option<String>,
}

/// Either a static string or a closure that returns a prompt
pub enum PromptSource {
    Static(String),
    Dynamic(Box<dyn Fn() -> String + Send + Sync>),
}

impl PromptSource {
    fn resolve(&self) -> String {
        match self {
            PromptSource::Static(s) => s.clone(),
            PromptSource::Dynamic(f) => f(),
        }
    }
}

impl Default for InputLineOptions {
    fn default() -> Self {
        Self {
            prompt: PromptSource::Static("> ".into()),
            prompt_color: Box::new(|t: &str| Style::new().cyan().paint(t)),
            history: Vec::new(),
            history_size: None,
            mask: None,
        }
    }
}

/// Prompted input line — wraps LineEditor with prompt styling and mask support
pub struct InputLine {
    editor: LineEditor,
    prompt: PromptSource,
    prompt_color: Box<dyn Fn(&str) -> String + Send + Sync>,
    mask: Option<String>,
    history_size: Option<usize>,
}

impl InputLine {
    pub fn new(options: InputLineOptions) -> Self {
        let editor = LineEditor::with_history(options.history);
        Self {
            editor,
            prompt: options.prompt,
            prompt_color: options.prompt_color,
            mask: options.mask,
            history_size: options.history_size,
        }
    }

    // Delegate all editing to LineEditor
    pub fn insert_char(&mut self, ch: &str) {
        self.editor.insert_char(ch);
    }
    pub fn backspace(&mut self) {
        self.editor.backspace();
    }
    pub fn delete_char(&mut self) {
        self.editor.delete_char();
    }
    pub fn home(&mut self) {
        self.editor.home();
    }
    pub fn end(&mut self) {
        self.editor.end();
    }
    pub fn cursor_left(&mut self) {
        self.editor.cursor_left();
    }
    pub fn cursor_right(&mut self) {
        self.editor.cursor_right();
    }
    pub fn word_left(&mut self) {
        self.editor.word_left();
    }
    pub fn word_right(&mut self) {
        self.editor.word_right();
    }
    pub fn delete_word(&mut self) {
        self.editor.delete_word();
    }
    pub fn clear_before(&mut self) {
        self.editor.clear_before();
    }
    pub fn clear_after(&mut self) {
        self.editor.clear_after();
    }
    pub fn clear_line(&mut self) {
        self.editor.clear_line();
    }
    pub fn set_value(&mut self, text: &str, pos: Option<usize>) {
        self.editor.set_value(text, pos);
    }
    pub fn history_up(&mut self) {
        self.editor.history_up();
    }
    pub fn history_down(&mut self) {
        self.editor.history_down();
    }

    /// Submit: return buffer, add to history, enforce history_size, reset state
    pub fn submit(&mut self) -> String {
        let result = self.editor.submit();
        if let Some(max) = self.history_size {
            self.editor.truncate_history(max);
        }
        result
    }

    /// Render input line with prompt — returns lines + cursor position for liveBlock
    pub fn render(&self) -> InputLineRender {
        let raw_prompt = self.prompt.resolve();
        let styled_prompt = (self.prompt_color)(&raw_prompt);
        let prompt_width = measure_width(&raw_prompt);

        let display = match &self.mask {
            Some(mask) => mask.repeat(self.editor.buffer().chars().count()),
            None => self.editor.buffer().to_string(),
        };

        let line = format!("{}{}", styled_prompt, display);

        let cursor_display = match &self.mask {
            Some(mask) => mask.repeat(self.editor.buffer()[..self.editor.cursor()].chars().count()),
            None => self.editor.buffer()[..self.editor.cursor()].to_string(),
        };
        let cursor_col = prompt_width + measure_width(&cursor_display);

        InputLineRender {
            lines: vec![line],
            cursor: (0, cursor_col),
        }
    }

    /// Current buffer content
    pub fn buffer(&self) -> &str {
        self.editor.buffer()
    }

    /// Cursor position in buffer
    pub fn cursor(&self) -> usize {
        self.editor.cursor()
    }

    /// The underlying line editor
    pub fn editor(&self) -> &LineEditor {
        &self.editor
    }

    /// Mutable access to the underlying line editor
    pub fn editor_mut(&mut self) -> &mut LineEditor {
        &mut self.editor
    }
}
