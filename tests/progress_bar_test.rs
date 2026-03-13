use prism::progress_bar::*;

#[test]
fn bar_at_zero() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = render_progress_bar(0, &RenderOptions::default());
    // Should have some content
    assert!(!result.is_empty());
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn bar_at_100_percent() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = render_progress_bar(100, &RenderOptions::default());
    assert!(!result.is_empty());
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn bar_at_50_percent() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = render_progress_bar(50, &RenderOptions::default());
    assert!(!result.is_empty());
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn bar_classic_style() {
    std::env::set_var("FORCE_COLOR", "1");
    let opts = RenderOptions { style: BarStyle::Classic, ..Default::default() };
    let result = render_progress_bar(50, &opts);
    assert!(result.contains("["));
    assert!(result.contains("]"));
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn all_styles_render() {
    std::env::set_var("FORCE_COLOR", "1");
    let styles = [
        BarStyle::Bar, BarStyle::Blocks, BarStyle::Shades, BarStyle::Classic,
        BarStyle::Arrows, BarStyle::Smooth, BarStyle::Dots, BarStyle::Square,
        BarStyle::Circle, BarStyle::Pipe,
    ];
    for style in styles {
        let opts = RenderOptions { style, ..Default::default() };
        let result = render_progress_bar(50, &opts);
        assert!(!result.is_empty(), "Style {:?} produced empty output", style);
    }
    std::env::remove_var("FORCE_COLOR");
}
