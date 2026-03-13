use crate::ansi::{measure_width, strip_ansi};
use crate::unicode::{
    next_grapheme_boundary, normalize_grapheme_boundary, previous_grapheme_boundary,
};

/// Snapshot of the editor state, suitable for rendering callbacks.
pub struct LineEditorState {
    pub buffer: String,
    pub cursor: usize,
    pub history_index: i32,
}

/// A pure line-editing state machine.
///
/// Manages a text buffer, cursor position, and history stack.
/// No terminal I/O — all mutations happen through method calls,
/// and an optional `on_render` callback is fired after each change.
pub struct LineEditor {
    buffer: String,
    cursor: usize,
    history: Vec<String>,
    history_index: i32, // -1 = current line
    saved_line: String,
    #[allow(clippy::type_complexity)]
    on_render: Option<Box<dyn FnMut(&LineEditorState)>>,
}

impl LineEditor {
    /// Create a new editor with empty state.
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
            history: Vec::new(),
            history_index: -1,
            saved_line: String::new(),
            on_render: None,
        }
    }

    /// Create a new editor pre-loaded with history entries.
    /// `history[0]` is the most recent (accessed first by history_up).
    pub fn with_history(history: Vec<String>) -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
            history,
            history_index: -1,
            saved_line: String::new(),
            on_render: None,
        }
    }

    /// Create a new editor with a render callback.
    pub fn with_on_render(on_render: Box<dyn FnMut(&LineEditorState)>) -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
            history: Vec::new(),
            history_index: -1,
            saved_line: String::new(),
            on_render: Some(on_render),
        }
    }

    /// Insert text at the current cursor position.
    pub fn insert_char(&mut self, ch: &str) {
        self.buffer.insert_str(self.cursor, ch);
        self.cursor += ch.len();
        self.notify();
    }

    /// Delete the grapheme cluster before the cursor.
    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let start = previous_grapheme_boundary(&self.buffer, self.cursor);
        self.buffer.drain(start..self.cursor);
        self.cursor = start;
        self.notify();
    }

    /// Delete the grapheme cluster after the cursor.
    pub fn delete_char(&mut self) {
        if self.cursor >= self.buffer.len() {
            return;
        }
        let end = next_grapheme_boundary(&self.buffer, self.cursor);
        self.buffer.drain(self.cursor..end);
        self.notify();
    }

    /// Move cursor to the beginning of the buffer.
    pub fn home(&mut self) {
        self.cursor = 0;
        self.notify();
    }

    /// Move cursor to the end of the buffer.
    pub fn end(&mut self) {
        self.cursor = self.buffer.len();
        self.notify();
    }

    /// Move cursor one grapheme cluster to the left.
    pub fn cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = previous_grapheme_boundary(&self.buffer, self.cursor);
        }
        self.notify();
    }

    /// Move cursor one grapheme cluster to the right.
    pub fn cursor_right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.cursor = next_grapheme_boundary(&self.buffer, self.cursor);
        }
        self.notify();
    }

    /// Move cursor one word to the left (skip spaces then non-spaces backward).
    pub fn word_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let bytes = self.buffer.as_bytes();
        let mut i = self.cursor;
        // skip spaces backward
        while i > 0 && bytes[i - 1] == b' ' {
            i -= 1;
        }
        // skip non-spaces backward
        while i > 0 && bytes[i - 1] != b' ' {
            i -= 1;
        }
        self.cursor = i;
        self.notify();
    }

    /// Move cursor one word to the right (skip non-spaces then spaces forward).
    pub fn word_right(&mut self) {
        if self.cursor >= self.buffer.len() {
            return;
        }
        let bytes = self.buffer.as_bytes();
        let mut i = self.cursor;
        // skip non-spaces forward
        while i < bytes.len() && bytes[i] != b' ' {
            i += 1;
        }
        // skip spaces forward
        while i < bytes.len() && bytes[i] == b' ' {
            i += 1;
        }
        self.cursor = i;
        self.notify();
    }

    /// Delete the word before the cursor (Ctrl+W).
    pub fn delete_word(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let start = self.cursor;
        let bytes = self.buffer.as_bytes();
        let mut i = self.cursor;
        while i > 0 && bytes[i - 1] == b' ' {
            i -= 1;
        }
        while i > 0 && bytes[i - 1] != b' ' {
            i -= 1;
        }
        self.buffer.drain(i..start);
        self.cursor = i;
        self.notify();
    }

    /// Delete everything before the cursor (Ctrl+U).
    pub fn clear_before(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.buffer.drain(..self.cursor);
        self.cursor = 0;
        self.notify();
    }

    /// Delete everything after the cursor (Ctrl+K).
    pub fn clear_after(&mut self) {
        if self.cursor >= self.buffer.len() {
            return;
        }
        self.buffer.truncate(self.cursor);
        self.notify();
    }

    /// Clear the entire line.
    pub fn clear_line(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
        self.notify();
    }

    /// Replace the buffer contents. Cursor goes to `pos` or end of text.
    pub fn set_value(&mut self, text: &str, pos: Option<usize>) {
        self.buffer = text.to_string();
        self.cursor = normalize_grapheme_boundary(text, pos.unwrap_or(text.len()));
        self.notify();
    }

    /// Submit the current line: return it, push to history, and reset.
    pub fn submit(&mut self) -> String {
        let line = self.buffer.clone();
        if !line.trim().is_empty() && (self.history.is_empty() || self.history[0] != line) {
            self.history.insert(0, line.clone());
        }
        self.buffer.clear();
        self.cursor = 0;
        self.history_index = -1;
        self.saved_line.clear();
        self.notify();
        line
    }

    /// Navigate to the previous (older) history entry.
    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        if self.history_index >= self.history.len() as i32 - 1 {
            return;
        }
        if self.history_index == -1 {
            self.saved_line = self.buffer.clone();
        }
        self.history_index += 1;
        self.buffer = self.history[self.history_index as usize].clone();
        self.cursor = self.buffer.len();
        self.notify();
    }

    /// Navigate to the next (newer) history entry, or back to the current line.
    pub fn history_down(&mut self) {
        if self.history_index < 0 {
            return;
        }
        self.history_index -= 1;
        if self.history_index == -1 {
            self.buffer = std::mem::take(&mut self.saved_line);
        } else {
            self.buffer = self.history[self.history_index as usize].clone();
        }
        self.cursor = self.buffer.len();
        self.notify();
    }

    /// Render the prompt + buffer, returning the full line and the cursor column.
    pub fn render_input(&self, prompt: &str) -> (String, usize) {
        let prompt_width = measure_width(&strip_ansi(prompt));
        let cursor_text = &self.buffer[..self.cursor];
        let cursor_col = prompt_width + measure_width(cursor_text);
        (format!("{}{}", prompt, self.buffer), cursor_col)
    }

    /// The current buffer contents.
    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    /// The current cursor byte offset.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// A snapshot of the current editor state.
    pub fn state(&self) -> LineEditorState {
        LineEditorState {
            buffer: self.buffer.clone(),
            cursor: self.cursor,
            history_index: self.history_index,
        }
    }

    fn notify(&mut self) {
        if let Some(ref mut cb) = self.on_render {
            let state = LineEditorState {
                buffer: self.buffer.clone(),
                cursor: self.cursor,
                history_index: self.history_index,
            };
            cb(&state);
        }
    }
}

impl Default for LineEditor {
    fn default() -> Self {
        Self::new()
    }
}
