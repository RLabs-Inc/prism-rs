use prism::unicode::*;

#[test]
fn segments_ascii() {
    let segs = grapheme_segments("hello");
    assert_eq!(segs.len(), 5);
    assert_eq!(segs[0].segment, "h");
}

#[test]
fn segments_empty() {
    assert!(grapheme_segments("").is_empty());
}

#[test]
fn previous_boundary_basic() {
    assert_eq!(previous_grapheme_boundary("hello", 3), 2);
    assert_eq!(previous_grapheme_boundary("hello", 0), 0);
}

#[test]
fn next_boundary_basic() {
    assert_eq!(next_grapheme_boundary("hello", 2), 3);
    assert_eq!(next_grapheme_boundary("hello", 5), 5);
}

#[test]
fn normalize_boundary_at_boundary() {
    assert_eq!(normalize_grapheme_boundary("hello", 3), 3);
}

#[test]
fn segments_emoji() {
    // Family emoji is a single grapheme cluster
    let segs = grapheme_segments("a\u{1F468}\u{200D}\u{1F469}b");
    // Should be 3 segments: "a", the family emoji, "b"
    assert_eq!(segs.len(), 3);
}

#[test]
fn previous_boundary_at_start() {
    assert_eq!(previous_grapheme_boundary("hello", 0), 0);
}

#[test]
fn next_boundary_at_end() {
    assert_eq!(next_grapheme_boundary("hello", 5), 5);
}

#[test]
fn normalize_empty() {
    assert_eq!(normalize_grapheme_boundary("", 0), 0);
}
