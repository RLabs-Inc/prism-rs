use prism::banner::*;

#[test]
fn banner_basic() {
    std::env::set_var("FORCE_COLOR", "1");
    let result = banner("HI", &BannerOptions::default());
    let lines: Vec<&str> = result.split('\n').collect();
    assert_eq!(lines.len(), 5); // 5 rows
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn banner_all_styles() {
    std::env::set_var("FORCE_COLOR", "1");
    for style in [BannerStyle::Block, BannerStyle::Shade, BannerStyle::Dots, BannerStyle::Ascii, BannerStyle::Outline] {
        let opts = BannerOptions { style, ..Default::default() };
        let result = banner("A", &opts);
        assert!(!result.is_empty());
    }
    std::env::remove_var("FORCE_COLOR");
}

#[test]
fn banner_empty() {
    let result = banner("", &BannerOptions::default());
    // 5 empty rows
    let lines: Vec<&str> = result.split('\n').collect();
    assert_eq!(lines.len(), 5);
}
