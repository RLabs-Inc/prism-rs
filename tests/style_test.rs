use prism::style::{s, rgb, hex, Color, Style, RESET};

// IMPORTANT: Style tests check raw ANSI output. Since paint() respects ansi_enabled(),
// and ansi_enabled() returns false in piped/CI environments, these tests use the Style
// struct directly to verify the open/close codes are correct. They also test paint()
// behavior with FORCE_COLOR=1 set.

/// Helper: build the styled string bypassing ansi_enabled() check.
/// Tests the Style struct's code accumulation, not the runtime TTY check.
fn force_paint(style: Style, text: &str) -> String {
    // Set FORCE_COLOR for this test
    std::env::set_var("FORCE_COLOR", "1");
    let result = style.paint(text);
    std::env::remove_var("FORCE_COLOR");
    result
}

#[test]
fn style_paint_no_modifiers() {
    let result = s().paint("hello");
    assert_eq!(result, "hello");
}

#[test]
fn style_bold() {
    let result = force_paint(s().bold(), "bold");
    assert!(result.contains("\x1b[1m"));
    assert!(result.contains("bold"));
    assert!(result.contains("\x1b[22m"));
}

#[test]
fn style_red_foreground() {
    let result = force_paint(s().red(), "error");
    assert!(result.contains("\x1b[31m"));
    assert!(result.contains("error"));
    assert!(result.contains("\x1b[39m"));
}

#[test]
fn style_chained_bold_red() {
    let result = force_paint(s().bold().red(), "critical");
    assert!(result.contains("\x1b[1m"));
    assert!(result.contains("\x1b[31m"));
    assert!(result.contains("critical"));
}

#[test]
fn style_background_color() {
    let result = force_paint(s().bg_blue(), "bg");
    assert!(result.contains("\x1b[44m"));
    assert!(result.contains("\x1b[49m"));
}

#[test]
fn style_rgb_foreground() {
    let result = force_paint(s().fg(rgb(255, 87, 51)), "orange");
    assert!(result.contains("\x1b[38;2;255;87;51m"));
}

#[test]
fn style_hex_foreground() {
    let result = force_paint(s().fg(hex(0xFF5733)), "hex");
    assert!(result.contains("\x1b[38;2;255;87;51m"));
}

#[test]
fn style_rgb_background() {
    let result = force_paint(s().bg_color(rgb(30, 30, 30)), "dark");
    assert!(result.contains("\x1b[48;2;30;30;30m"));
}

#[test]
fn style_bright_variants() {
    let result = force_paint(s().bright_red(), "bright");
    assert!(result.contains("\x1b[91m"));
}

#[test]
fn style_dim_italic_strikethrough() {
    let r1 = force_paint(s().dim(), "dim");
    assert!(r1.contains("\x1b[2m"));
    let r2 = force_paint(s().italic(), "italic");
    assert!(r2.contains("\x1b[3m"));
    let r3 = force_paint(s().strikethrough(), "strike");
    assert!(r3.contains("\x1b[9m"));
}

#[test]
fn style_reset_constant() {
    assert_eq!(RESET, "\x1b[0m");
}

#[test]
fn style_underline_inverse() {
    let r1 = force_paint(s().underline(), "under");
    assert!(r1.contains("\x1b[4m"));
    let r2 = force_paint(s().inverse(), "inv");
    assert!(r2.contains("\x1b[7m"));
}

#[test]
fn style_empty_text() {
    let result = force_paint(s().bold().red(), "");
    assert!(result.contains("\x1b[1m"));
    assert!(result.contains("\x1b[31m"));
}
