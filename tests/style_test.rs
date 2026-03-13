use prism::style::{hex, rgb, s, RESET};

#[test]
fn style_paint_no_modifiers() {
    let result = s().render("hello");
    assert_eq!(result, "hello");
}

#[test]
fn style_bold() {
    let result = s().bold().render("bold");
    assert!(result.contains("\x1b[1m"));
    assert!(result.contains("bold"));
    assert!(result.contains("\x1b[22m"));
}

#[test]
fn style_red_foreground() {
    let result = s().red().render("error");
    assert!(result.contains("\x1b[31m"));
    assert!(result.contains("error"));
    assert!(result.contains("\x1b[39m"));
}

#[test]
fn style_chained_bold_red() {
    let result = s().bold().red().render("critical");
    assert!(result.contains("\x1b[1m"));
    assert!(result.contains("\x1b[31m"));
    assert!(result.contains("critical"));
}

#[test]
fn style_background_color() {
    let result = s().bg_blue().render("bg");
    assert!(result.contains("\x1b[44m"));
    assert!(result.contains("\x1b[49m"));
}

#[test]
fn style_rgb_foreground() {
    let result = s().fg(rgb(255, 87, 51)).render("orange");
    assert!(result.contains("\x1b[38;2;255;87;51m"));
}

#[test]
fn style_hex_foreground() {
    let result = s().fg(hex(0xFF5733)).render("hex");
    assert!(result.contains("\x1b[38;2;255;87;51m"));
}

#[test]
fn style_rgb_background() {
    let result = s().bg_color(rgb(30, 30, 30)).render("dark");
    assert!(result.contains("\x1b[48;2;30;30;30m"));
}

#[test]
fn style_bright_variants() {
    let result = s().bright_red().render("bright");
    assert!(result.contains("\x1b[91m"));
}

#[test]
fn style_dim_italic_strikethrough() {
    let r1 = s().dim().render("dim");
    assert!(r1.contains("\x1b[2m"));
    let r2 = s().italic().render("italic");
    assert!(r2.contains("\x1b[3m"));
    let r3 = s().strikethrough().render("strike");
    assert!(r3.contains("\x1b[9m"));
}

#[test]
fn style_reset_constant() {
    assert_eq!(RESET, "\x1b[0m");
}

#[test]
fn style_underline_inverse() {
    let r1 = s().underline().render("under");
    assert!(r1.contains("\x1b[4m"));
    let r2 = s().inverse().render("inv");
    assert!(r2.contains("\x1b[7m"));
}

#[test]
fn style_empty_text() {
    let result = s().bold().red().render("");
    assert!(result.contains("\x1b[1m"));
    assert!(result.contains("\x1b[31m"));
}
