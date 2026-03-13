use prism::badge::*;

#[test]
fn badge_bracket_default() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = badge("OK", BadgeVariant::Bracket, None);
    // Should contain the text
    assert!(result.contains("OK"));
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn badge_dot() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = badge("Active", BadgeVariant::Dot, None);
    assert!(result.contains("●"));
    assert!(result.contains("Active"));
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn badge_pill() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = badge("NEW", BadgeVariant::Pill, None);
    assert!(result.contains("NEW"));
    std::env::remove_var("FORCE_COLOR");
}
