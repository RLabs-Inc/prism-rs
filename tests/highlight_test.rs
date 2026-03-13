use prism::highlight::*;

#[test]
fn highlight_rust_keyword() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = highlight("fn main() {}", &HighlightOptions { language: Language::Rust, ..Default::default() });
    assert!(result.contains("\x1b[")); // has ANSI codes
    assert!(result.contains("fn"));
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn highlight_with_line_numbers() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = highlight("let x = 1;\nlet y = 2;", &HighlightOptions {
        language: Language::Rust, line_numbers: true, ..Default::default()
    });
    assert!(result.contains("│")); // line number separator
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn highlight_auto_detect_rust() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = highlight("fn main() {\n    let mut x = 5;\n}", &HighlightOptions::default());
    assert!(result.contains("fn"));
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn highlight_json() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = highlight("{\"key\": \"value\", \"num\": 42}", &HighlightOptions { language: Language::Json, ..Default::default() });
    assert!(result.contains("key"));
    std::env::remove_var("FORCE_COLOR");
}
