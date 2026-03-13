use prism::exec::{Exec, ExecOptions};

#[test]
fn write_basic() {
    let mut ex = Exec::new(
        "echo hello",
        ExecOptions {
            width: Some(60),
            ..Default::default()
        },
    );
    ex.write("hello\n");
    assert_eq!(ex.line_count(), 1);
    assert!(ex.running());
}

#[test]
fn write_multiple_lines() {
    let mut ex = Exec::new(
        "ls",
        ExecOptions {
            width: Some(60),
            ..Default::default()
        },
    );
    ex.write("file1.txt\nfile2.txt\nfile3.txt\n");
    assert_eq!(ex.line_count(), 3);
}

#[test]
fn write_partial_line() {
    let mut ex = Exec::new(
        "echo",
        ExecOptions {
            width: Some(60),
            ..Default::default()
        },
    );
    ex.write("hel");
    assert_eq!(ex.line_count(), 1); // partial counts
    ex.write("lo\n");
    assert_eq!(ex.line_count(), 1); // completed line
}

#[test]
fn done_sets_stopped() {
    let mut ex = Exec::new(
        "echo",
        ExecOptions {
            width: Some(60),
            ..Default::default()
        },
    );
    ex.write("output\n");
    ex.done(0);
    assert!(!ex.running());
}

#[test]
fn fail_sets_stopped() {
    let mut ex = Exec::new(
        "bad",
        ExecOptions {
            width: Some(60),
            ..Default::default()
        },
    );
    ex.fail("command not found");
    assert!(!ex.running());
}

#[test]
fn write_after_done_is_ignored() {
    let mut ex = Exec::new(
        "echo",
        ExecOptions {
            width: Some(60),
            ..Default::default()
        },
    );
    ex.done(0);
    ex.write("more output\n");
    assert_eq!(ex.line_count(), 0);
}

#[test]
fn scroll_down_up() {
    let mut ex = Exec::new(
        "ls",
        ExecOptions {
            max_height: 3,
            width: Some(60),
            ..Default::default()
        },
    );
    for i in 0..10 {
        ex.write(&format!("line {}\n", i));
    }
    assert!(ex.scrollable());

    // Scroll up
    ex.scroll(-3);
    assert!(ex.scroll_offset() < 7);
}

#[test]
fn render_returns_lines() {
    let mut ex = Exec::new(
        "echo hello",
        ExecOptions {
            width: Some(40),
            ..Default::default()
        },
    );
    ex.write("hello world\n");
    let lines = ex.render();
    // Header + command + output + footer = at least 4 lines
    assert!(lines.len() >= 4);
}

#[test]
fn freeze_shows_all_lines() {
    let mut ex = Exec::new(
        "ls",
        ExecOptions {
            max_height: 3,
            width: Some(60),
            ..Default::default()
        },
    );
    for i in 0..10 {
        ex.write(&format!("line {}\n", i));
    }
    ex.done(0);
    let frozen = ex.freeze();
    // freeze shows ALL lines (header + cmd + 10 lines + footer = 13)
    assert!(frozen.len() > ex.render().len());
}

#[test]
fn carriage_return_processing() {
    let mut ex = Exec::new(
        "progress",
        ExecOptions {
            width: Some(60),
            ..Default::default()
        },
    );
    ex.write("Loading...\rDone!     \n");
    // The line should be "Done!     " (last segment after \r)
    assert_eq!(ex.line_count(), 1);
}

#[test]
fn not_scrollable_when_few_lines() {
    let mut ex = Exec::new(
        "echo",
        ExecOptions {
            width: Some(60),
            ..Default::default()
        },
    );
    ex.write("one\ntwo\n");
    assert!(!ex.scrollable());
}

#[test]
fn done_flushes_partial() {
    let mut ex = Exec::new(
        "echo",
        ExecOptions {
            width: Some(60),
            ..Default::default()
        },
    );
    ex.write("partial output");
    assert_eq!(ex.line_count(), 1); // partial shows as line
    ex.done(0);
    assert_eq!(ex.line_count(), 1); // flushed to real line
}
