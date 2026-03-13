use prism::ansi::{strip_ansi, measure_width, wrap_ansi};

// strip_ansi tests
#[test]
fn strip_plain_text() {
    assert_eq!(strip_ansi("hello"), "hello");
}

#[test]
fn strip_csi_sequence() {
    assert_eq!(strip_ansi("\x1b[31mred\x1b[39m"), "red");
}

#[test]
fn strip_multiple_sequences() {
    assert_eq!(strip_ansi("\x1b[1m\x1b[31mbold red\x1b[39m\x1b[22m"), "bold red");
}

#[test]
fn strip_osc_hyperlink() {
    assert_eq!(strip_ansi("\x1b]8;;https://example.com\x07link\x1b]8;;\x07"), "link");
}

#[test]
fn strip_empty() {
    assert_eq!(strip_ansi(""), "");
}

#[test]
fn strip_only_ansi() {
    assert_eq!(strip_ansi("\x1b[31m\x1b[39m"), "");
}

// measure_width tests
#[test]
fn width_plain_ascii() {
    assert_eq!(measure_width("hello"), 5);
}

#[test]
fn width_with_ansi() {
    assert_eq!(measure_width("\x1b[31mhello\x1b[39m"), 5);
}

#[test]
fn width_empty() {
    assert_eq!(measure_width(""), 0);
}

#[test]
fn width_cjk_characters() {
    // CJK characters are 2 columns wide each
    assert_eq!(measure_width("\u{4f60}\u{597d}"), 4); // 你好 = 2+2
}

// wrap_ansi tests
#[test]
fn wrap_short_text_no_wrap() {
    assert_eq!(wrap_ansi("hello world", 80), "hello world");
}

#[test]
fn wrap_at_word_boundary() {
    let result = wrap_ansi("hello world foo", 11);
    assert_eq!(result, "hello world\nfoo");
}

#[test]
fn wrap_preserves_ansi() {
    let input = "\x1b[31mhello world\x1b[39m";
    let result = wrap_ansi(input, 5);
    assert!(result.contains("\x1b[31m"));
}

#[test]
fn wrap_hard_break_long_word() {
    let result = wrap_ansi("abcdefghij", 5);
    assert_eq!(result, "abcde\nfghij");
}

#[test]
fn wrap_empty() {
    assert_eq!(wrap_ansi("", 80), "");
}

#[test]
fn wrap_preserves_existing_newlines() {
    assert_eq!(wrap_ansi("hello\nworld", 80), "hello\nworld");
}
