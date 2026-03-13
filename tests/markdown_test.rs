use prism::markdown::md;

#[test]
fn md_heading() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = md("# Hello");
    assert!(result.contains("Hello"));
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn md_bold() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = md("**bold text**");
    assert!(result.contains("bold text"));
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn md_code_block() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = md("```\ncode here\n```");
    assert!(result.contains("code here"));
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn md_list() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = md("- item one\n- item two");
    assert!(result.contains("item one"));
    assert!(result.contains("item two"));
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn md_blockquote() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = md("> quoted text");
    assert!(result.contains("quoted text"));
    std::env::remove_var("FORCE_COLOR");
}
