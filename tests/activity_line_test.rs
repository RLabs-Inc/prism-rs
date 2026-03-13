use prism::activity_line::{ActivityLine, ActivityLineOptions, Icon};
use prism::ansi::strip_ansi;

#[test]
fn default_render_has_spinner_frame_and_text() {
    let al = ActivityLine::new("Loading...", ActivityLineOptions::default());
    let lines = al.render();
    assert_eq!(lines.len(), 1);
    let plain = strip_ansi(&lines[0]);
    // First frame of dots spinner is "⠋"
    assert!(plain.starts_with("⠋"), "expected dots frame, got: {}", plain);
    assert!(plain.contains("Loading..."), "expected text, got: {}", plain);
}

#[test]
fn text_changes_message() {
    let mut al = ActivityLine::new("first", ActivityLineOptions::default());
    al.text("second");
    let plain = strip_ansi(&al.render()[0]);
    assert!(plain.contains("second"));
    assert!(!plain.contains("first"));
}

#[test]
fn tick_changes_spinner_frame() {
    let mut al = ActivityLine::new("test", ActivityLineOptions::default());
    let frame0 = strip_ansi(&al.render()[0]);
    al.tick();
    let frame1 = strip_ansi(&al.render()[0]);
    // Dots spinner: frame 0 = "⠋", frame 1 = "⠙" — they differ
    assert_ne!(frame0, frame1, "tick should change the frame");
}

#[test]
fn freeze_returns_frozen_line_with_icon() {
    let mut al = ActivityLine::new("Processing", ActivityLineOptions::default());
    let lines = al.freeze("✓", None, None);
    assert_eq!(lines.len(), 1);
    let plain = strip_ansi(&lines[0]);
    assert!(plain.contains("✓"), "expected icon, got: {}", plain);
    assert!(plain.contains("Processing"), "expected text, got: {}", plain);
}

#[test]
fn freeze_with_custom_message() {
    let mut al = ActivityLine::new("Working", ActivityLineOptions::default());
    let lines = al.freeze("✓", Some("Done!"), None);
    let plain = strip_ansi(&lines[0]);
    assert!(plain.contains("Done!"));
    assert!(!plain.contains("Working"));
}

#[test]
fn freeze_with_custom_color() {
    fn red(text: &str) -> String {
        prism::s().red().render(text)
    }
    let mut al = ActivityLine::new("test", ActivityLineOptions::default());
    let lines = al.freeze("✗", None, Some(red));
    // The icon should be wrapped in red ANSI codes
    assert!(lines[0].contains("\x1b[31m✗"));
}

#[test]
fn custom_static_icon() {
    let al = ActivityLine::new("test", ActivityLineOptions {
        icon: Some(Icon::Static("→".to_string())),
        ..ActivityLineOptions::default()
    });
    let plain = strip_ansi(&al.render()[0]);
    assert!(plain.starts_with("→"), "expected static icon, got: {}", plain);
}

#[test]
fn custom_frames() {
    let mut al = ActivityLine::new("test", ActivityLineOptions {
        icon: Some(Icon::Frames(vec!["A".to_string(), "B".to_string()])),
        ..ActivityLineOptions::default()
    });
    let p0 = strip_ansi(&al.render()[0]);
    assert!(p0.starts_with("A"));
    al.tick();
    let p1 = strip_ansi(&al.render()[0]);
    assert!(p1.starts_with("B"));
    al.tick();
    let p2 = strip_ansi(&al.render()[0]);
    assert!(p2.starts_with("A")); // wraps around
}

#[test]
fn metrics_callback() {
    let al = ActivityLine::new("test", ActivityLineOptions {
        metrics: Some(Box::new(|| "42 items".to_string())),
        ..ActivityLineOptions::default()
    });
    let plain = strip_ansi(&al.render()[0]);
    assert!(plain.contains("42 items"), "expected metrics, got: {}", plain);
}

#[test]
fn interval_ms_from_spinner() {
    let al = ActivityLine::new("test", ActivityLineOptions {
        icon: Some(Icon::Spinner("star")),
        ..ActivityLineOptions::default()
    });
    assert_eq!(al.interval_ms(), 100); // star spinner is 100ms
}

#[test]
fn interval_ms_override() {
    let al = ActivityLine::new("test", ActivityLineOptions {
        interval_ms: Some(200),
        ..ActivityLineOptions::default()
    });
    assert_eq!(al.interval_ms(), 200);
}
