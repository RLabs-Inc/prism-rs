use crate::timer::format_time;
use std::time::Instant;

/// Pure elapsed timer state machine. Zero I/O.
pub struct Elapsed {
    start: Instant,
}

impl Elapsed {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Formatted elapsed: "42ms", "1.2s", "3m 12s"
    pub fn render(&self) -> String {
        format_time(self.ms())
    }

    /// Raw elapsed in milliseconds
    pub fn ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// Reset timer start point
    pub fn reset(&mut self) {
        self.start = Instant::now();
    }
}

impl Default for Elapsed {
    fn default() -> Self {
        Self::new()
    }
}
