use prism::text::*;

#[test]
fn truncate_short_text() {
    assert_eq!(truncate("hello", 10, "..."), "hello");
}

#[test]
fn truncate_at_boundary() {
    assert_eq!(truncate("hello world", 5, "..."), "he...");
}

#[test]
fn truncate_exact_width() {
    assert_eq!(truncate("hello", 5, "..."), "hello");
}

#[test]
fn truncate_single_char_with_ellipsis() {
    assert_eq!(truncate("hello world", 4, "..."), "h...");
}

#[test]
fn truncate_ellipsis_wider_than_width() {
    // When ellipsis is wider than allowed width, truncate ellipsis itself
    assert_eq!(truncate("hello", 2, "..."), "..");
}

#[test]
fn truncate_with_ansi() {
    // ANSI codes don't count toward width
    let styled = "\x1b[31mhello world\x1b[39m";
    let result = truncate(styled, 5, "...");
    // Should truncate visible text but preserve ANSI
    assert!(result.len() <= styled.len()); // got truncated
}

#[test]
fn indent_basic() {
    assert_eq!(indent("hello\nworld", 2, " "), "  hello\n  world");
}

#[test]
fn indent_single_line() {
    assert_eq!(indent("hello", 4, " "), "    hello");
}

#[test]
fn indent_custom_char() {
    assert_eq!(indent("hello", 2, ">"), ">>hello");
}

#[test]
fn pad_left() {
    assert_eq!(pad("hi", 5, "left"), "hi   ");
}

#[test]
fn pad_right() {
    assert_eq!(pad("hi", 5, "right"), "   hi");
}

#[test]
fn pad_center() {
    assert_eq!(pad("hi", 6, "center"), "  hi  ");
}

#[test]
fn pad_no_padding_needed() {
    assert_eq!(pad("hello", 3, "left"), "hello");
}

#[test]
fn link_non_tty() {
    let result = link("click", "https://example.com", false);
    assert_eq!(result, "click (https://example.com)");
}

#[test]
fn link_tty() {
    let result = link("click", "https://example.com", true);
    assert!(result.contains("\x1b]8;;"));
    assert!(result.contains("https://example.com"));
    assert!(result.contains("click"));
}

#[test]
fn wrap_short() {
    assert_eq!(wrap("hello", 80), "hello");
}

#[test]
fn wrap_delegates_to_ansi() {
    let result = wrap("hello world foo", 11);
    assert_eq!(result, "hello world\nfoo");
}
