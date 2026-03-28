// Integration test — verify the flat public API is accessible via `use prism::*`
#![allow(unused_imports)]

// Foundation
use prism::{
    ansi_enabled, color, ensure_cursor_visible, grapheme_segments, hex, hide_cursor, indent,
    interactive_tty, is_tty, link, measure_width, next_grapheme_boundary,
    normalize_grapheme_boundary, pad, pipe_aware, previous_grapheme_boundary, rgb, s, strip_ansi,
    style, term_width, truncate, visual_rows, wrap, wrap_ansi, write_err, Color, GraphemeSegment,
    PrismError, PrismResult, Style, RESET,
};

// Composition Layer
use prism::{
    activity, layout, live_block, section, statusbar, statusbar_render, stream, stream_with,
    Activity, ActivityOptions, BlockRender, Layout, LayoutActivityOptions, LayoutOptions,
    LayoutSectionOptions, LayoutStreamOptions, LiveBlock, LiveBlockOptions, Section,
    SectionOptions, Segment, StatusBarConfig, Stream, StreamOptions,
};

// Display Components
use prism::{
    banner, columns, diff, divider, file_preview, frame, header, highlight, kv, list, md, table,
    tree, Align, BannerColorFn, BannerOptions, BannerStyle, BorderChars, BorderStyle, Column,
    ColumnsOptions, DiffOptions, FilePreviewOptions, FrameOptions, HighlightOptions, KvOptions,
    Language, ListOptions, ListStyle, TableOptions, TreeNode,
};

// Animation & Feedback
use prism::{
    all_spinner_names, badge, bench, countdown, format_time, get_spinner, progress,
    render_progress_bar, stopwatch, BadgeVariant, BarStyle, BenchResult, Countdown,
    CountdownOptions, Elapsed, ProgressBar, ProgressOptions, RenderOptions, SpinnerDef, Stopwatch,
};

// Interactive Input
use prism::{
    args, confirm, input, keypress_stream, multiselect, password, raw_mode, raw_mode_reset,
    readline, repl, select, ArgsConfig, ArgsResult, Command, CommandDef, CommandMatch,
    CommandRouter, ConfirmOptions, Exec, ExecOptions, FlagDef, FlagType, FlagValue, InputLine,
    InputLineOptions, InputLineRender, InputOptions, KeyEvent, LineEditor, LineEditorState,
    MultiSelectOptions, PasswordOptions, PromptSource, ReadlineOptions, ReplCommand, ReplOptions,
    SelectOptions,
};

#[test]
fn foundation_accessible() {
    let _ = is_tty();
    let _ = ansi_enabled();
    let _ = term_width();
    let _ = interactive_tty();

    let styled = s().bold().red().paint("test");
    assert!(!styled.is_empty());

    let styled2 = style().green().paint("ok");
    assert!(!styled2.is_empty());

    let stripped = strip_ansi("\x1b[31mred\x1b[39m");
    assert_eq!(stripped, "red");

    assert_eq!(measure_width("hello"), 5);
    assert_eq!(measure_width("\x1b[1mbold\x1b[0m"), 4);

    let wrapped = wrap_ansi("hello world", 5);
    assert!(!wrapped.is_empty());

    let c = color("text", rgb(255, 0, 0), None);
    assert!(!c.is_empty());

    let _ = hex(0xFF0000);
    assert!(!RESET.is_empty());
}

#[test]
fn text_utilities_accessible() {
    let t = truncate("hello world", 8, "...");
    assert!(t.len() <= 11);

    let i = indent("line1\nline2", 2, "  ");
    assert!(i.contains("    line1"));

    let p = pad("hi", 10, "center");
    assert_eq!(measure_width(&p), 10);

    let l = link("click", "https://example.com", false);
    assert!(l.contains("click"));

    let w = wrap("short", 80);
    assert_eq!(w, "short");
}

#[test]
fn unicode_accessible() {
    let segs = grapheme_segments("abc");
    assert_eq!(segs.len(), 3);

    let prev = previous_grapheme_boundary("abc", 2);
    assert_eq!(prev, 1);

    let next = next_grapheme_boundary("abc", 1);
    assert_eq!(next, 2);

    let norm = normalize_grapheme_boundary("abc", 1);
    assert_eq!(norm, 1);
}

#[test]
fn display_components_accessible() {
    let f = frame("content", &FrameOptions::default());
    assert!(f.contains("content"));

    let d = divider("-", 20);
    assert!(!d.is_empty());

    let h = header("Title", 40);
    assert!(h.contains("Title"));

    let data = vec![vec![("name", "Alice"), ("age", "30")]];
    let t = table(&data, &TableOptions::default());
    assert!(t.contains("Alice"));

    let l = list(&["a", "b", "c"], &ListOptions::default());
    assert!(l.contains("a"));

    let k = kv(&[("key", "val")], &KvOptions::default());
    assert!(k.contains("key"));

    let c = columns(&["col1", "col2", "col3"], &ColumnsOptions::default());
    assert!(!c.is_empty());

    let d = diff("old\n", "new\n", &DiffOptions::default());
    assert!(!d.is_empty());

    let fp = file_preview("fn main() {}", &FilePreviewOptions::default());
    assert!(!fp.is_empty());

    let b = banner("HI", &BannerOptions::default());
    assert!(!b.is_empty());

    let m = md("# Hello\n\n**bold** text");
    assert!(!m.is_empty());

    let hl = highlight("let x = 42;", &HighlightOptions::default());
    assert!(!hl.is_empty());
}

#[test]
fn feedback_accessible() {
    let b = badge("OK", BadgeVariant::Bracket, None);
    assert!(b.contains("OK"));

    let ft = format_time(12345);
    assert!(!ft.is_empty());

    let names = all_spinner_names();
    assert!(names.len() >= 44);

    let dots = get_spinner("dots");
    assert!(dots.is_some());

    let bar = render_progress_bar(
        50,
        &RenderOptions {
            total: 100,
            ..Default::default()
        },
    );
    assert!(!bar.is_empty());

    let e = Elapsed::new();
    let r = e.render();
    assert!(!r.is_empty());
}

#[test]
fn interactive_types_accessible() {
    // Just verify types are constructible
    let _ = LineEditor::new();
    let editor = LineEditor::with_history(vec!["prev".into()]);
    assert_eq!(editor.buffer(), "");

    let _ = CommandRouter::new(
        vec![(
            "test".into(),
            Command {
                description: Some("test cmd".into()),
                aliases: vec![],
                hidden: false,
            },
        )],
        "/",
    );

    let config = ArgsConfig {
        name: "test".into(),
        version: None,
        description: None,
        flags: vec![],
        commands: vec![],
        argv: Some(vec!["test".into()]),
        no_exit: true,
        ..Default::default()
    };
    let result = args(config);
    assert!(result.command.is_none());
}

#[test]
fn pipe_aware_strips_ansi() {
    let colored = s().red().paint("hello");
    let plain = pipe_aware(&colored);
    // In a test environment, this depends on TTY detection
    // but the function should not panic
    assert!(!plain.is_empty());
}

#[test]
fn visual_rows_calculates_wrapping() {
    assert_eq!(visual_rows("short", 80), 1);
    assert_eq!(visual_rows("", 80), 1);
    // Long line wraps
    let long = "a".repeat(160);
    assert_eq!(visual_rows(&long, 80), 2);
}
