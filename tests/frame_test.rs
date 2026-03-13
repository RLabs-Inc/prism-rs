use prism::frame::*;

#[test]
fn frame_basic() {
    let result = frame("hello", &FrameOptions::default());
    assert!(result.contains("hello"));
    assert!(result.contains("│")); // single border vertical
    assert!(result.contains("─")); // single border horizontal
}

#[test]
fn frame_with_title() {
    let opts = FrameOptions {
        title: Some("Title".to_string()),
        ..Default::default()
    };
    let result = frame("content", &opts);
    assert!(result.contains("Title"));
    assert!(result.contains("content"));
}

#[test]
fn frame_double_border() {
    let opts = FrameOptions {
        border: BorderStyle::Double,
        ..Default::default()
    };
    let result = frame("test", &opts);
    assert!(result.contains("║"));
    assert!(result.contains("═"));
}

#[test]
fn divider_default() {
    let result = divider("─", 40);
    assert_eq!(result.len(), "─".len() * 40); // unicode char repeated
}

#[test]
fn header_basic() {
    let result = header("Section", 40);
    assert!(result.contains("Section"));
    assert!(result.contains("─"));
}

#[test]
fn all_border_styles_work() {
    for style in [
        BorderStyle::Single,
        BorderStyle::Double,
        BorderStyle::Rounded,
        BorderStyle::Heavy,
    ] {
        let opts = FrameOptions {
            border: style,
            width: Some(30),
            ..Default::default()
        };
        let result = frame("test", &opts);
        assert!(!result.is_empty());
    }
}
