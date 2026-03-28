use std::sync::Mutex;

use crate::style::s;
use crate::writer::{ansi_enabled, writeln as wln};

/// Global log configuration (timestamp, prefix).
#[derive(Clone, Debug, Default)]
pub struct LogConfig {
    /// Prepend HH:MM:SS timestamp to every log line
    pub timestamp: bool,
    /// Prepend [prefix] before the icon
    pub prefix: Option<String>,
}

/// Per-call overrides for log functions.
#[derive(Clone, Debug, Default)]
pub struct LogOptions {
    /// Override global timestamp setting for this call
    pub timestamp: Option<bool>,
    /// Override global prefix for this call
    pub prefix: Option<String>,
}

static DEFAULTS: Mutex<LogConfig> = Mutex::new(LogConfig {
    timestamp: false,
    prefix: None,
});

/// Set global log defaults (timestamp, prefix). Applies to all subsequent log calls.
pub fn configure(config: LogConfig) {
    let mut defaults = DEFAULTS.lock().unwrap();
    defaults.timestamp = config.timestamp;
    if config.prefix.is_some() {
        defaults.prefix = config.prefix;
    }
}

/// Format HH:MM:SS timestamp (dim styled).
fn ts() -> String {
    let now = local_time_hms();
    s().dim().paint(&now)
}

/// Get current local time as HH:MM:SS without chrono dependency.
fn local_time_hms() -> String {
    use std::time::SystemTime;
    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = dur.as_secs();

    #[cfg(unix)]
    let local_secs = {
        extern "C" {
            fn localtime_r(timep: *const i64, result: *mut LibcTm) -> *mut LibcTm;
        }

        #[repr(C)]
        #[allow(non_camel_case_types)]
        struct LibcTm {
            tm_sec: i32,
            tm_min: i32,
            tm_hour: i32,
            _tm_mday: i32,
            _tm_mon: i32,
            _tm_year: i32,
            _tm_wday: i32,
            _tm_yday: i32,
            _tm_isdst: i32,
            tm_gmtoff: i64,
            _tm_zone: *const u8,
        }

        let time_val = total_secs as i64;
        let mut tm = std::mem::MaybeUninit::<LibcTm>::zeroed();
        unsafe {
            localtime_r(&time_val, tm.as_mut_ptr());
            let tm = tm.assume_init();
            (total_secs as i64 + tm.tm_gmtoff) as u64
        }
    };

    #[cfg(not(unix))]
    let local_secs = total_secs;

    let secs_in_day = local_secs % 86400;
    let h = secs_in_day / 3600;
    let m = (secs_in_day % 3600) / 60;
    let sec = secs_in_day % 60;
    format!("{:02}:{:02}:{:02}", h, m, sec)
}

/// Internal format helper — merges defaults with per-call options, formats and writes.
fn fmt(icon: &str, color_fn: fn(&str) -> String, msg: &str, opts: Option<&LogOptions>) {
    let defaults = DEFAULTS.lock().unwrap().clone();

    let use_timestamp = opts
        .and_then(|o| o.timestamp)
        .unwrap_or(defaults.timestamp);
    let use_prefix = opts
        .and_then(|o| o.prefix.clone())
        .or(defaults.prefix);

    let mut parts: Vec<String> = Vec::new();

    if use_timestamp {
        parts.push(ts());
    }
    if let Some(ref pfx) = use_prefix {
        parts.push(s().dim().paint(&format!("[{}]", pfx)));
    }

    let styled_icon = if ansi_enabled() {
        color_fn(icon)
    } else {
        icon.to_string()
    };
    parts.push(styled_icon);
    parts.push(msg.to_string());

    wln(&parts.join(" "));
}

fn blue(t: &str) -> String {
    s().blue().paint(t)
}
fn yellow(t: &str) -> String {
    s().yellow().paint(t)
}
fn red(t: &str) -> String {
    s().red().paint(t)
}
fn green(t: &str) -> String {
    s().green().paint(t)
}
fn dim(t: &str) -> String {
    s().dim().paint(t)
}
fn cyan(t: &str) -> String {
    s().cyan().paint(t)
}

/// Structured CLI logging with icons and colors.
/// ℹ informational
pub fn info(msg: &str, options: Option<&LogOptions>) {
    fmt("\u{2139}", blue, msg, options);
}

/// ⚠ warning
pub fn warn(msg: &str, options: Option<&LogOptions>) {
    fmt("\u{26A0}", yellow, msg, options);
}

/// ✖ error
pub fn error(msg: &str, options: Option<&LogOptions>) {
    fmt("\u{2716}", red, msg, options);
}

/// ✔ success
pub fn success(msg: &str, options: Option<&LogOptions>) {
    fmt("\u{2714}", green, msg, options);
}

/// ● debug
pub fn debug(msg: &str, options: Option<&LogOptions>) {
    fmt("\u{25CF}", dim, msg, options);
}

/// ▸ step/action
pub fn step(msg: &str, options: Option<&LogOptions>) {
    fmt("\u{25B8}", cyan, msg, options);
}
