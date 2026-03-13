use prism::statusbar::*;
use prism::ansi::measure_width;

#[test]
fn statusbar_left_only() {
    let result = statusbar_render(
        &StatusBarConfig {
            left: vec![Segment::Text("hello".into())],
            ..Default::default()
        },
        40,
    );
    assert!(result.contains("hello"));
}

#[test]
fn statusbar_left_right() {
    let result = statusbar_render(
        &StatusBarConfig {
            left: vec![Segment::Text("left".into())],
            right: Some(Segment::Text("right".into())),
            ..Default::default()
        },
        40,
    );
    assert!(result.contains("left"));
    assert!(result.contains("right"));
}

#[test]
fn statusbar_separator() {
    let result = statusbar_render(
        &StatusBarConfig {
            left: vec![Segment::Text("a".into()), Segment::Text("b".into())],
            separator: Some(" | ".into()),
            ..Default::default()
        },
        40,
    );
    let stripped = prism::ansi::strip_ansi(&result);
    assert!(stripped.contains("a") && stripped.contains("b"));
}

#[test]
fn statusbar_indent() {
    let result = statusbar_render(
        &StatusBarConfig {
            left: vec![Segment::Text("test".into())],
            indent: Some(4),
            ..Default::default()
        },
        40,
    );
    assert!(result.starts_with("    "));
}

#[test]
fn statusbar_truncation() {
    let result = statusbar_render(
        &StatusBarConfig {
            left: vec![Segment::Text(
                "this is a very long segment that should be truncated".into(),
            )],
            ..Default::default()
        },
        20,
    );
    assert!(measure_width(&result) <= 20);
}
