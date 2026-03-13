use prism::writer;

#[test]
fn term_width_returns_positive() {
    assert!(writer::term_width() > 0);
}

#[test]
fn visual_rows_empty_line() {
    assert_eq!(writer::visual_rows("", 80), 1);
}

#[test]
fn visual_rows_short_line() {
    assert_eq!(writer::visual_rows("hello", 80), 1);
}

#[test]
fn visual_rows_exact_width() {
    let line = "x".repeat(80);
    assert_eq!(writer::visual_rows(&line, 80), 1);
}

#[test]
fn visual_rows_wrapping() {
    let line = "x".repeat(81);
    assert_eq!(writer::visual_rows(&line, 80), 2);
}

#[test]
fn visual_rows_double_wrap() {
    let line = "x".repeat(161);
    assert_eq!(writer::visual_rows(&line, 80), 3);
}
