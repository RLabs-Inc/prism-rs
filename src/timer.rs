use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crate::style::s;
use crate::writer::{is_tty, write, writeln};

/// Format milliseconds into a human-readable time string.
///
/// - < 1000ms  → "42ms"
/// - < 60_000ms → "1.2s" (one decimal)
/// - < 3_600_000ms → "1m 30s"
/// - >= 3_600_000ms → "1h 1m"
pub fn format_time(ms: u64) -> String {
    if ms < 1_000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        let secs = ms as f64 / 1_000.0;
        format!("{:.1}s", secs)
    } else if ms < 3_600_000 {
        let total_secs = ms / 1_000;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{}m {}s", mins, secs)
    } else {
        let total_mins = ms / 60_000;
        let hours = total_mins / 60;
        let mins = total_mins % 60;
        format!("{}h {}m", hours, mins)
    }
}

// ─── Stopwatch ────────────────────────────────────────────────────────────────

pub struct Stopwatch {
    label: Option<String>,
    start: Instant,
    lap_start: Instant,
    lap_count: u32,
}

impl Stopwatch {
    fn new(label: Option<&str>) -> Self {
        let now = Instant::now();
        Self {
            label: label.map(|s| s.to_string()),
            start: now,
            lap_start: now,
            lap_count: 0,
        }
    }

    /// Elapsed since start, in milliseconds.
    pub fn elapsed(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// Formatted elapsed since start.
    pub fn elapsed_str(&self) -> String {
        format_time(self.elapsed())
    }

    /// Mark a lap: prints lap time and returns the lap duration in ms.
    pub fn lap(&mut self) -> u64 {
        self.lap_count += 1;
        let lap_ms = self.lap_start.elapsed().as_millis() as u64;
        self.lap_start = Instant::now();

        let lap_label = format!(
            "Lap {}{}",
            self.lap_count,
            self.label
                .as_deref()
                .map(|l| format!(" [{}]", l))
                .unwrap_or_default()
        );
        let time_str = format_time(lap_ms);
        writeln(&format!(
            "  {} {}",
            s().dim().paint(&lap_label),
            s().cyan().bold().paint(&time_str)
        ));
        lap_ms
    }

    /// Stop the stopwatch and print total time. Returns elapsed ms.
    pub fn stop(&self) -> u64 {
        let ms = self.elapsed();
        self.done();
        ms
    }

    /// Print final elapsed (without stopping internal state).
    pub fn done(&self) {
        let ms = self.elapsed();
        let time_str = format_time(ms);
        let prefix = match &self.label {
            Some(l) => format!("{} done", l),
            None => "done".to_string(),
        };
        writeln(&format!(
            "{} {}",
            s().dim().paint(&prefix),
            s().green().bold().paint(&time_str)
        ));
    }
}

/// Create a new stopwatch, optionally with a label.
pub fn stopwatch(label: Option<&str>) -> Stopwatch {
    Stopwatch::new(label)
}

// ─── Countdown ────────────────────────────────────────────────────────────────

pub struct CountdownOptions {
    /// Update interval in milliseconds (default 1000).
    pub interval_ms: u64,
    /// Whether to show inline \r updates (default true when TTY).
    pub inline: Option<bool>,
}

impl Default for CountdownOptions {
    fn default() -> Self {
        Self {
            interval_ms: 1_000,
            inline: None,
        }
    }
}

pub struct Countdown {
    cancel_flag: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Countdown {
    /// Cancel the countdown and wait for the thread to finish.
    pub fn cancel(mut self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }

    /// Wait for the countdown to complete naturally.
    pub fn wait(mut self) {
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// Spawn a countdown timer thread.
///
/// Decrements every `options.interval_ms` milliseconds and writes inline
/// updates to stdout. Returns a handle with `cancel()` and `wait()`.
pub fn countdown(seconds: u64, label: &str, options: CountdownOptions) -> Countdown {
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&cancel_flag);
    let label = label.to_string();
    let inline = options.inline.unwrap_or_else(is_tty);
    let interval_ms = options.interval_ms;

    let handle = thread::spawn(move || {
        let ticks = (seconds * 1_000) / interval_ms;
        let mut remaining_ms = seconds * 1_000;

        for _ in 0..=ticks {
            if flag.load(Ordering::Relaxed) {
                break;
            }

            let time_str = format_time(remaining_ms);
            if inline {
                write(&format!(
                    "\r\x1b[2K  {} {}",
                    s().dim().paint(&label),
                    s().yellow().bold().paint(&time_str)
                ));
            } else {
                writeln(&format!(
                    "  {} {}",
                    s().dim().paint(&label),
                    s().yellow().bold().paint(&time_str)
                ));
            }

            if remaining_ms == 0 {
                break;
            }

            thread::sleep(std::time::Duration::from_millis(interval_ms));

            if remaining_ms >= interval_ms {
                remaining_ms -= interval_ms;
            } else {
                remaining_ms = 0;
            }
        }

        if inline && !flag.load(Ordering::Relaxed) {
            // Clear the inline line on completion
            write("\r\x1b[2K");
        }
    });

    Countdown {
        cancel_flag,
        handle: Some(handle),
    }
}

// ─── Bench ────────────────────────────────────────────────────────────────────

pub struct BenchResult {
    pub name: String,
    pub iterations: u64,
    /// Total elapsed in milliseconds (excluding warmup)
    pub total_ms: u64,
    /// Average time per iteration in microseconds
    pub avg_us: f64,
    /// Operations per second
    pub ops_per_sec: f64,
}

impl BenchResult {
    /// Print a formatted summary to stdout.
    pub fn print(&self) {
        let ops_str = if self.ops_per_sec >= 1_000_000.0 {
            format!("{:.2}M ops/sec", self.ops_per_sec / 1_000_000.0)
        } else if self.ops_per_sec >= 1_000.0 {
            format!("{:.2}K ops/sec", self.ops_per_sec / 1_000.0)
        } else {
            format!("{:.2} ops/sec", self.ops_per_sec)
        };

        writeln(&format!(
            "  {} — {} iters, {}, avg {:.2}µs",
            s().bold().paint(&self.name),
            s().cyan().paint(&self.iterations.to_string()),
            s().green().paint(&ops_str),
            self.avg_us
        ));
    }
}

/// Benchmark a function over N iterations (with 10-iteration warmup).
///
/// Returns a `BenchResult` with timing statistics.
pub fn bench(name: &str, f: impl Fn(), iterations: u64) -> BenchResult {
    // Warmup
    for _ in 0..10 {
        f();
    }

    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    let total_ms = elapsed.as_millis() as u64;
    let total_us = elapsed.as_micros() as f64;
    let avg_us = if iterations > 0 {
        total_us / iterations as f64
    } else {
        0.0
    };
    let ops_per_sec = if total_us > 0.0 {
        (iterations as f64 / total_us) * 1_000_000.0
    } else {
        f64::INFINITY
    };

    BenchResult {
        name: name.to_string(),
        iterations,
        total_ms,
        avg_us,
        ops_per_sec,
    }
}
