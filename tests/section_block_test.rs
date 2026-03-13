use prism::ansi::strip_ansi;
use prism::section_block::{SectionBlock, SectionBlockOptions};

#[test]
fn default_render_has_title_line() {
    let sb = SectionBlock::new("Building", SectionBlockOptions::default());
    let lines = sb.render();
    assert_eq!(lines.len(), 1);
    let plain = strip_ansi(&lines[0]);
    assert!(plain.contains("Building"), "expected title, got: {}", plain);
    // Default indent is 2 spaces
    assert!(plain.starts_with("  "), "expected indent, got: {:?}", plain);
}

#[test]
fn add_creates_items_with_connector() {
    let mut sb = SectionBlock::new("Section", SectionBlockOptions::default());
    sb.add("item one");
    sb.add("item two");
    let lines = sb.render();
    assert_eq!(lines.len(), 3);
    let plain1 = strip_ansi(&lines[1]);
    assert!(plain1.contains("item one"), "got: {}", plain1);
    // Connector ⎿ should be present
    assert!(plain1.contains("\u{23BF}"), "expected connector, got: {}", plain1);
    let plain2 = strip_ansi(&lines[2]);
    assert!(plain2.contains("item two"), "got: {}", plain2);
}

#[test]
fn body_replaces_all_items() {
    let mut sb = SectionBlock::new("Section", SectionBlockOptions::default());
    sb.add("old item");
    sb.body("line1\nline2\nline3");
    let lines = sb.render();
    assert_eq!(lines.len(), 4); // title + 3 body lines
    let plain1 = strip_ansi(&lines[1]);
    assert!(plain1.contains("line1"));
    assert!(!plain1.contains("old item"));
}

#[test]
fn body_empty_clears_items() {
    let mut sb = SectionBlock::new("Section", SectionBlockOptions::default());
    sb.add("item");
    sb.body("");
    let lines = sb.render();
    assert_eq!(lines.len(), 1); // title only
}

#[test]
fn freeze_with_collapse_hides_items() {
    let mut sb = SectionBlock::new("Building", SectionBlockOptions {
        collapse_on_done: true,
        ..SectionBlockOptions::default()
    });
    sb.add("step 1");
    sb.add("step 2");
    let lines = sb.freeze("✓", None, None);
    assert_eq!(lines.len(), 1, "collapse should hide items");
    let plain = strip_ansi(&lines[0]);
    assert!(plain.contains("✓"));
    assert!(plain.contains("Building"));
}

#[test]
fn freeze_without_collapse_keeps_items() {
    let mut sb = SectionBlock::new("Building", SectionBlockOptions::default());
    sb.add("step 1");
    sb.add("step 2");
    let lines = sb.freeze("✓", None, None);
    assert_eq!(lines.len(), 3, "should keep title + 2 items");
    let plain0 = strip_ansi(&lines[0]);
    assert!(plain0.contains("✓"));
    let plain1 = strip_ansi(&lines[1]);
    assert!(plain1.contains("step 1"));
}

#[test]
fn freeze_with_message_replaces_title() {
    let mut sb = SectionBlock::new("Working", SectionBlockOptions::default());
    let lines = sb.freeze("✓", Some("Complete"), None);
    let plain = strip_ansi(&lines[0]);
    assert!(plain.contains("Complete"));
    assert!(!plain.contains("Working"));
}

#[test]
fn tick_changes_spinner_frame() {
    let mut sb = SectionBlock::new("test", SectionBlockOptions::default());
    let frame0 = strip_ansi(&sb.render()[0]);
    sb.tick();
    let frame1 = strip_ansi(&sb.render()[0]);
    assert_ne!(frame0, frame1, "tick should change the frame");
}

#[test]
fn title_and_text_update_message() {
    let mut sb = SectionBlock::new("original", SectionBlockOptions::default());
    sb.title("via title");
    let plain = strip_ansi(&sb.render()[0]);
    assert!(plain.contains("via title"));

    sb.text("via text");
    let plain = strip_ansi(&sb.render()[0]);
    assert!(plain.contains("via text"));
}

#[test]
fn custom_indent_and_connector() {
    let mut sb = SectionBlock::new("Title", SectionBlockOptions {
        indent: 4,
        connector: "|".to_string(),
        ..SectionBlockOptions::default()
    });
    sb.add("child");
    let lines = sb.render();
    let plain0 = strip_ansi(&lines[0]);
    assert!(plain0.starts_with("    "), "expected 4-space indent");
    let plain1 = strip_ansi(&lines[1]);
    assert!(plain1.contains("|"), "expected custom connector");
}

#[test]
fn interval_ms_from_spinner() {
    let sb = SectionBlock::new("test", SectionBlockOptions {
        spinner: "star",
        ..SectionBlockOptions::default()
    });
    assert_eq!(sb.interval_ms(), 100);
}

#[test]
fn freeze_with_custom_color() {
    fn green(text: &str) -> String {
        prism::s().green().render(text)
    }
    let mut sb = SectionBlock::new("test", SectionBlockOptions::default());
    let lines = sb.freeze("✓", None, Some(green));
    assert!(lines[0].contains("\x1b[32m✓"), "expected green icon");
}
