use std::sync::atomic::{AtomicUsize, Ordering};

const HIDE: &str = "\x1b[?25l";
const SHOW: &str = "\x1b[?25h";

static REF_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Hide the terminal cursor. Ref-counted: each hide needs a matching show.
pub fn hide_cursor() {
    let prev = REF_COUNT.fetch_add(1, Ordering::SeqCst);
    if prev == 0 {
        crate::writer::write(HIDE);
    }
}

/// Show the terminal cursor. Only actually shows when all hides are balanced.
pub fn show_cursor() {
    let prev = REF_COUNT.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
        if v > 0 { Some(v - 1) } else { None }
    });
    if prev == Ok(1) {
        crate::writer::write(SHOW);
    }
}

/// Force-reset cursor visibility (for exit handlers / panic recovery)
pub fn ensure_cursor_visible() {
    REF_COUNT.store(0, Ordering::SeqCst);
    crate::writer::write(SHOW);
}

/// Current hide ref count (useful for testing)
pub fn cursor_ref_count() -> usize {
    REF_COUNT.load(Ordering::SeqCst)
}
