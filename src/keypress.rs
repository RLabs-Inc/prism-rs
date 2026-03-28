// prism/keypress — raw keyboard input
// reads individual keypresses without waiting for Enter
// foundation for prompt, select, and interactive components
//
// Uses libc termios directly for raw mode instead of crossterm::terminal.
// Reason: crossterm's enable_raw_mode() calls cfmakeraw() which clears OPOST,
// disabling output \n → \r\n translation. This breaks all terminal rendering.
// We only set raw INPUT flags, preserving output processing — matching how
// Node/Bun's setRawMode(true) works in the TypeScript original.

use crossterm::event::{self, Event, KeyCode, KeyEvent as CtKeyEvent, KeyEventKind, KeyModifiers};
use std::os::unix::io::AsRawFd;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

static RAW_MODE_REFS: AtomicUsize = AtomicUsize::new(0);
static ORIGINAL_TERMIOS: OnceLock<libc::termios> = OnceLock::new();

/// A parsed keyboard event
#[derive(Debug, Clone)]
pub struct KeyEvent {
    /// Key name: "a", "enter", "up", "tab", "space", "backspace", "f1", "wordleft", etc.
    pub key: String,
    /// The printable character, if any
    pub char_val: Option<char>,
    /// Ctrl modifier was held
    pub ctrl: bool,
    /// Shift modifier was held
    pub shift: bool,
    /// Alt/Option modifier was held
    pub meta: bool,
    /// Description of the key event
    pub sequence: String,
}

/// Map a crossterm KeyCode + modifiers to our key name and char_val
fn map_key(code: &KeyCode, modifiers: KeyModifiers) -> (String, Option<char>) {
    let ctrl = modifiers.contains(KeyModifiers::CONTROL);

    match code {
        KeyCode::Enter => ("enter".into(), None),
        KeyCode::Tab => ("tab".into(), None),
        KeyCode::BackTab => ("tab".into(), None), // shift+tab
        KeyCode::Backspace => ("backspace".into(), None),
        KeyCode::Esc => ("escape".into(), None),
        KeyCode::Up => ("up".into(), None),
        KeyCode::Down => ("down".into(), None),
        KeyCode::Left => {
            if ctrl {
                ("wordleft".into(), None)
            } else {
                ("left".into(), None)
            }
        }
        KeyCode::Right => {
            if ctrl {
                ("wordright".into(), None)
            } else {
                ("right".into(), None)
            }
        }
        KeyCode::Home => ("home".into(), None),
        KeyCode::End => ("end".into(), None),
        KeyCode::Insert => ("insert".into(), None),
        KeyCode::Delete => ("delete".into(), None),
        KeyCode::PageUp => ("pageup".into(), None),
        KeyCode::PageDown => ("pagedown".into(), None),
        KeyCode::F(n) => (format!("f{}", n), None),
        KeyCode::Char(' ') => ("space".into(), Some(' ')),
        KeyCode::Char(c) => {
            if ctrl {
                // ctrl+a..z: key is the lowercase letter
                (c.to_lowercase().to_string(), None)
            } else {
                (c.to_string(), Some(*c))
            }
        }
        KeyCode::Null => ("null".into(), None),
        _ => ("unknown".into(), None),
    }
}

/// Convert a crossterm key event to our KeyEvent
fn from_crossterm(ct: &CtKeyEvent) -> KeyEvent {
    let ctrl = ct.modifiers.contains(KeyModifiers::CONTROL);
    let shift = ct.modifiers.contains(KeyModifiers::SHIFT);
    let meta = ct.modifiers.contains(KeyModifiers::ALT);

    let (key, char_val) = map_key(&ct.code, ct.modifiers);

    let sequence = format!("{:?}", ct.code);

    KeyEvent {
        key,
        char_val,
        ctrl,
        shift,
        meta,
        sequence,
    }
}

/// Save original termios and enable raw INPUT mode.
/// Preserves OPOST so \n → \r\n translation works on output.
fn enable_raw_input() {
    let fd = std::io::stdin().as_raw_fd();
    let mut termios = unsafe { std::mem::zeroed::<libc::termios>() };
    unsafe { libc::tcgetattr(fd, &mut termios) };
    ORIGINAL_TERMIOS.get_or_init(|| termios);

    // Raw INPUT only — matches Node/Bun's setRawMode(true)
    termios.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
    termios.c_cflag |= libc::CS8;
    termios.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);
    // DO NOT touch c_oflag — OPOST stays on, \n → \r\n works naturally
    termios.c_cc[libc::VMIN] = 0;
    termios.c_cc[libc::VTIME] = 0;

    unsafe { libc::tcsetattr(fd, libc::TCSAFLUSH, &termios) };
}

/// Restore original terminal settings.
fn disable_raw_input() {
    if let Some(original) = ORIGINAL_TERMIOS.get() {
        let fd = std::io::stdin().as_raw_fd();
        unsafe { libc::tcsetattr(fd, libc::TCSAFLUSH, original) };
    }
}

/// Enable or disable raw mode with reference counting.
/// First enable turns on raw mode, last disable turns it off.
pub fn raw_mode(enable: bool) {
    if enable {
        let prev = RAW_MODE_REFS.fetch_add(1, Ordering::SeqCst);
        if prev == 0 {
            enable_raw_input();
        }
    } else {
        // CAS loop to avoid TOCTOU race
        let _ = RAW_MODE_REFS.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
            if current == 0 {
                None // don't underflow
            } else {
                Some(current - 1)
            }
        });
        if RAW_MODE_REFS.load(Ordering::SeqCst) == 0 {
            disable_raw_input();
        }
    }
}

/// Reset raw mode ref count and disable (for panic recovery)
pub fn raw_mode_reset() {
    RAW_MODE_REFS.store(0, Ordering::SeqCst);
    disable_raw_input();
}

/// Read a single keypress. Blocks until a key is pressed.
/// Enables raw mode for the read, disables after.
pub fn keypress() -> crate::error::PrismResult<KeyEvent> {
    raw_mode(true);
    let result = loop {
        match event::read() {
            Ok(Event::Key(ct)) => {
                // Only process Press events (skip Release/Repeat)
                if ct.kind == KeyEventKind::Press {
                    break Ok(from_crossterm(&ct));
                }
            }
            Ok(_) => {
                // Mouse, resize, etc — keep reading
            }
            Err(e) => {
                break Err(crate::error::PrismError::Io(e));
            }
        }
    };
    raw_mode(false);
    result
}

/// Read keypresses continuously. Calls the callback for each keypress.
/// If the callback returns `true`, reading stops.
/// Raw mode is enabled for the duration.
pub fn keypress_stream<F>(mut callback: F) -> crate::error::PrismResult<()>
where
    F: FnMut(&KeyEvent) -> bool,
{
    raw_mode(true);
    let result = loop {
        match event::read() {
            Ok(Event::Key(ct)) => {
                if ct.kind == KeyEventKind::Press {
                    let key = from_crossterm(&ct);
                    if callback(&key) {
                        break Ok(());
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                break Err(crate::error::PrismError::Io(e));
            }
        }
    };
    raw_mode(false);
    result
}

/// Read a single keypress with timeout. Returns None if no key within timeout.
///
/// Unlike `keypress()`, this does NOT manage raw mode automatically — the
/// caller must enable raw mode before entering the event loop and disable
/// it after. This is the primitive for building custom event loops that
/// need to refresh the display at a fixed rate regardless of input.
///
/// Handles both Press and Repeat events (held keys produce events).
/// Release events are filtered out.
pub fn keypress_poll(timeout: std::time::Duration) -> crate::error::PrismResult<Option<KeyEvent>> {
    if event::poll(timeout).map_err(crate::error::PrismError::Io)? {
        match event::read().map_err(crate::error::PrismError::Io)? {
            Event::Key(ct) if ct.kind != KeyEventKind::Release => {
                Ok(Some(from_crossterm(&ct)))
            }
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}
