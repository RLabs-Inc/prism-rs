use prism::line_editor::{LineEditor, LineEditorState};

// ── insert_char ──────────────────────────────────────────────────────

#[test]
fn insert_char_basic() {
    let mut ed = LineEditor::new();
    ed.insert_char("a");
    assert_eq!(ed.buffer(), "a");
    assert_eq!(ed.cursor(), 1);
}

#[test]
fn insert_char_multi() {
    let mut ed = LineEditor::new();
    ed.insert_char("h");
    ed.insert_char("i");
    assert_eq!(ed.buffer(), "hi");
    assert_eq!(ed.cursor(), 2);
}

#[test]
fn insert_char_at_middle() {
    let mut ed = LineEditor::new();
    ed.insert_char("a");
    ed.insert_char("c");
    ed.cursor_left();
    ed.insert_char("b");
    assert_eq!(ed.buffer(), "abc");
    assert_eq!(ed.cursor(), 2);
}

#[test]
fn insert_char_multi_byte() {
    let mut ed = LineEditor::new();
    ed.insert_char("é");
    assert_eq!(ed.buffer(), "é");
    assert_eq!(ed.cursor(), 2); // é is 2 bytes in UTF-8
}

// ── backspace ────────────────────────────────────────────────────────

#[test]
fn backspace_at_start_is_noop() {
    let mut ed = LineEditor::new();
    ed.backspace();
    assert_eq!(ed.buffer(), "");
    assert_eq!(ed.cursor(), 0);
}

#[test]
fn backspace_at_end() {
    let mut ed = LineEditor::new();
    ed.insert_char("a");
    ed.insert_char("b");
    ed.backspace();
    assert_eq!(ed.buffer(), "a");
    assert_eq!(ed.cursor(), 1);
}

#[test]
fn backspace_in_middle() {
    let mut ed = LineEditor::new();
    ed.set_value("abc", Some(2));
    ed.backspace();
    assert_eq!(ed.buffer(), "ac");
    assert_eq!(ed.cursor(), 1);
}

// ── delete_char ──────────────────────────────────────────────────────

#[test]
fn delete_char_at_end_is_noop() {
    let mut ed = LineEditor::new();
    ed.insert_char("a");
    ed.delete_char();
    assert_eq!(ed.buffer(), "a");
    assert_eq!(ed.cursor(), 1);
}

#[test]
fn delete_char_in_middle() {
    let mut ed = LineEditor::new();
    ed.set_value("abc", Some(1));
    ed.delete_char();
    assert_eq!(ed.buffer(), "ac");
    assert_eq!(ed.cursor(), 1);
}

// ── home / end ───────────────────────────────────────────────────────

#[test]
fn home_moves_to_start() {
    let mut ed = LineEditor::new();
    ed.set_value("hello", None);
    ed.home();
    assert_eq!(ed.cursor(), 0);
}

#[test]
fn end_moves_to_end() {
    let mut ed = LineEditor::new();
    ed.set_value("hello", Some(0));
    ed.end();
    assert_eq!(ed.cursor(), 5);
}

// ── cursor_left / cursor_right ───────────────────────────────────────

#[test]
fn cursor_left_at_start_stays() {
    let mut ed = LineEditor::new();
    ed.set_value("abc", Some(0));
    ed.cursor_left();
    assert_eq!(ed.cursor(), 0);
}

#[test]
fn cursor_right_at_end_stays() {
    let mut ed = LineEditor::new();
    ed.set_value("abc", None);
    ed.cursor_right();
    assert_eq!(ed.cursor(), 3);
}

#[test]
fn cursor_left_right_round_trip() {
    let mut ed = LineEditor::new();
    ed.set_value("abc", Some(2));
    ed.cursor_left();
    assert_eq!(ed.cursor(), 1);
    ed.cursor_right();
    assert_eq!(ed.cursor(), 2);
}

// ── word_left / word_right ───────────────────────────────────────────

#[test]
fn word_left_skips_word() {
    let mut ed = LineEditor::new();
    ed.set_value("hello world", None);
    ed.word_left();
    assert_eq!(ed.cursor(), 6); // before 'w'
}

#[test]
fn word_left_at_start_is_noop() {
    let mut ed = LineEditor::new();
    ed.set_value("hello", Some(0));
    ed.word_left();
    assert_eq!(ed.cursor(), 0);
}

#[test]
fn word_right_skips_word() {
    let mut ed = LineEditor::new();
    ed.set_value("hello world", Some(0));
    ed.word_right();
    assert_eq!(ed.cursor(), 6); // after space, at 'w'
}

#[test]
fn word_right_at_end_is_noop() {
    let mut ed = LineEditor::new();
    ed.set_value("hello", None);
    ed.word_right();
    assert_eq!(ed.cursor(), 5);
}

// ── delete_word ──────────────────────────────────────────────────────

#[test]
fn delete_word_removes_previous_word() {
    let mut ed = LineEditor::new();
    ed.set_value("hello world", None);
    ed.delete_word();
    assert_eq!(ed.buffer(), "hello ");
    assert_eq!(ed.cursor(), 6);
}

#[test]
fn delete_word_at_start_is_noop() {
    let mut ed = LineEditor::new();
    ed.set_value("hello", Some(0));
    ed.delete_word();
    assert_eq!(ed.buffer(), "hello");
    assert_eq!(ed.cursor(), 0);
}

// ── clear_before / clear_after / clear_line ──────────────────────────

#[test]
fn clear_before_removes_text_before_cursor() {
    let mut ed = LineEditor::new();
    ed.set_value("hello world", Some(5));
    ed.clear_before();
    assert_eq!(ed.buffer(), " world");
    assert_eq!(ed.cursor(), 0);
}

#[test]
fn clear_before_at_start_is_noop() {
    let mut ed = LineEditor::new();
    ed.set_value("hello", Some(0));
    ed.clear_before();
    assert_eq!(ed.buffer(), "hello");
}

#[test]
fn clear_after_removes_text_after_cursor() {
    let mut ed = LineEditor::new();
    ed.set_value("hello world", Some(5));
    ed.clear_after();
    assert_eq!(ed.buffer(), "hello");
    assert_eq!(ed.cursor(), 5);
}

#[test]
fn clear_after_at_end_is_noop() {
    let mut ed = LineEditor::new();
    ed.set_value("hello", None);
    ed.clear_after();
    assert_eq!(ed.buffer(), "hello");
}

#[test]
fn clear_line_empties_buffer() {
    let mut ed = LineEditor::new();
    ed.set_value("hello world", None);
    ed.clear_line();
    assert_eq!(ed.buffer(), "");
    assert_eq!(ed.cursor(), 0);
}

// ── set_value ────────────────────────────────────────────────────────

#[test]
fn set_value_without_position_puts_cursor_at_end() {
    let mut ed = LineEditor::new();
    ed.set_value("test", None);
    assert_eq!(ed.buffer(), "test");
    assert_eq!(ed.cursor(), 4);
}

#[test]
fn set_value_with_position() {
    let mut ed = LineEditor::new();
    ed.set_value("test", Some(2));
    assert_eq!(ed.buffer(), "test");
    assert_eq!(ed.cursor(), 2);
}

// ── submit ───────────────────────────────────────────────────────────

#[test]
fn submit_returns_buffer_and_resets() {
    let mut ed = LineEditor::new();
    ed.set_value("hello", None);
    let line = ed.submit();
    assert_eq!(line, "hello");
    assert_eq!(ed.buffer(), "");
    assert_eq!(ed.cursor(), 0);
}

#[test]
fn submit_adds_to_history() {
    let mut ed = LineEditor::new();
    ed.set_value("cmd1", None);
    ed.submit();
    ed.set_value("cmd2", None);
    ed.submit();
    // Navigate history to verify
    ed.history_up();
    assert_eq!(ed.buffer(), "cmd2");
    ed.history_up();
    assert_eq!(ed.buffer(), "cmd1");
}

#[test]
fn submit_does_not_add_empty_to_history() {
    let mut ed = LineEditor::new();
    ed.submit(); // empty
    ed.set_value("   ", None);
    ed.submit(); // whitespace only
                 // history should be empty — history_up should be noop
    ed.history_up();
    assert_eq!(ed.buffer(), "");
}

#[test]
fn submit_does_not_add_duplicate_of_last() {
    let mut ed = LineEditor::new();
    ed.set_value("same", None);
    ed.submit();
    ed.set_value("same", None);
    ed.submit();
    // Only one entry in history
    ed.history_up();
    assert_eq!(ed.buffer(), "same");
    ed.history_up(); // should stay at same (no second entry)
    assert_eq!(ed.buffer(), "same");
}

// ── history navigation ──────────────────────────────────────────────

#[test]
fn history_up_down_navigation() {
    let mut ed = LineEditor::new();
    ed.set_value("first", None);
    ed.submit();
    ed.set_value("second", None);
    ed.submit();

    ed.history_up();
    assert_eq!(ed.buffer(), "second");
    ed.history_up();
    assert_eq!(ed.buffer(), "first");
    ed.history_down();
    assert_eq!(ed.buffer(), "second");
    ed.history_down();
    assert_eq!(ed.buffer(), ""); // back to current line
}

#[test]
fn history_saves_current_line() {
    let mut ed = LineEditor::new();
    ed.set_value("old", None);
    ed.submit();

    ed.set_value("typing", None);
    ed.history_up();
    assert_eq!(ed.buffer(), "old");
    ed.history_down();
    assert_eq!(ed.buffer(), "typing"); // restored
}

#[test]
fn history_up_on_empty_history_is_noop() {
    let mut ed = LineEditor::new();
    ed.history_up();
    assert_eq!(ed.buffer(), "");
}

#[test]
fn history_down_below_current_is_noop() {
    let mut ed = LineEditor::new();
    ed.history_down();
    assert_eq!(ed.buffer(), "");
}

#[test]
fn with_history_preloads() {
    let mut ed = LineEditor::with_history(vec!["a".into(), "b".into()]);
    ed.history_up();
    assert_eq!(ed.buffer(), "a");
    ed.history_up();
    assert_eq!(ed.buffer(), "b");
}

// ── render_input ────────────────────────────────────────────────────

#[test]
fn render_input_basic() {
    let mut ed = LineEditor::new();
    ed.set_value("hello", Some(3));
    let (line, col) = ed.render_input("> ");
    assert_eq!(line, "> hello");
    assert_eq!(col, 5); // 2 (prompt) + 3 (cursor in buffer)
}

#[test]
fn render_input_with_ansi_prompt() {
    let mut ed = LineEditor::new();
    ed.set_value("hi", None);
    // ANSI colored prompt: visible width is 2 ("$ ")
    let prompt = "\x1b[32m$\x1b[0m ";
    let (line, col) = ed.render_input(prompt);
    assert_eq!(line, format!("{}hi", prompt));
    assert_eq!(col, 4); // 2 (visible "$ ") + 2 ("hi")
}

// ── state ───────────────────────────────────────────────────────────

#[test]
fn state_returns_snapshot() {
    let mut ed = LineEditor::new();
    ed.set_value("test", Some(2));
    let s = ed.state();
    assert_eq!(s.buffer, "test");
    assert_eq!(s.cursor, 2);
    assert_eq!(s.history_index, -1);
}

// ── on_render callback ──────────────────────────────────────────────

#[test]
fn on_render_fires_on_mutation() {
    use std::sync::{Arc, Mutex};

    let count = Arc::new(Mutex::new(0u32));
    let count_clone = count.clone();

    let mut ed = LineEditor::with_on_render(Box::new(move |_state: &LineEditorState| {
        *count_clone.lock().unwrap() += 1;
    }));

    ed.insert_char("a");
    ed.insert_char("b");
    ed.backspace();
    assert_eq!(*count.lock().unwrap(), 3);
}
