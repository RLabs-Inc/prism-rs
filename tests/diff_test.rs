use prism::diff::*;

#[test]
fn diff_identical() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = diff("hello\nworld", "hello\nworld", &DiffOptions::default());
    assert!(result.contains("no changes"));
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn diff_added_line() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = diff("hello", "hello\nworld", &DiffOptions::default());
    assert!(result.contains("world"));
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn diff_removed_line() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = diff("hello\nworld", "hello", &DiffOptions::default());
    assert!(result.contains("world"));
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn diff_with_filename() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = diff("a", "b", &DiffOptions { filename: Some("test.rs".to_string()), ..Default::default() });
    assert!(result.contains("test.rs"));
    std::env::remove_var("FORCE_COLOR");
}
