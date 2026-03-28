//! Number formatting utilities — thousand separators, compact notation, rates.
//!
//! Every CLI tool needs to display counts, speeds, and sizes in human-readable
//! form. These functions complement `timer::format_time()` for the numeric side.

/// Format a number with thousand separators.
///
/// ```text
/// format_number(1234567)  → "1,234,567"
/// format_number(42)       → "42"
/// format_number(0)        → "0"
/// ```
pub fn format_number(n: u64) -> String {
    if n < 1_000 {
        return n.to_string();
    }

    let s = n.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len + len / 3);

    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(b as char);
    }

    result
}

/// Format a number in compact notation with K/M/G/T suffixes.
///
/// ```text
/// format_compact(42)         → "42"
/// format_compact(1_500)      → "1.5K"
/// format_compact(42_800)     → "42.8K"
/// format_compact(1_234_567)  → "1.2M"
/// format_compact(5_000_000_000) → "5.0G"
/// ```
pub fn format_compact(n: u64) -> String {
    if n < 1_000 {
        return n.to_string();
    }
    if n < 1_000_000 {
        let v = n as f64 / 1_000.0;
        return if v >= 100.0 {
            format!("{:.0}K", v)
        } else {
            format!("{:.1}K", v)
        };
    }
    if n < 1_000_000_000 {
        let v = n as f64 / 1_000_000.0;
        return if v >= 100.0 {
            format!("{:.0}M", v)
        } else {
            format!("{:.1}M", v)
        };
    }
    if n < 1_000_000_000_000 {
        let v = n as f64 / 1_000_000_000.0;
        return if v >= 100.0 {
            format!("{:.0}G", v)
        } else {
            format!("{:.1}G", v)
        };
    }
    let v = n as f64 / 1_000_000_000_000.0;
    if v >= 100.0 {
        format!("{:.0}T", v)
    } else {
        format!("{:.1}T", v)
    }
}

/// Format a rate as compact value per second.
///
/// ```text
/// format_rate(1500, 1.0)      → "1.5K/s"
/// format_rate(42, 2.0)        → "21/s"
/// format_rate(1_000_000, 0.5) → "2.0M/s"
/// format_rate(0, 0.0)         → "0/s"
/// ```
pub fn format_rate(count: u64, elapsed_secs: f64) -> String {
    if elapsed_secs <= 0.0 || count == 0 {
        return "0/s".to_string();
    }
    let per_sec = count as f64 / elapsed_secs;
    let n = per_sec.round() as u64;
    format!("{}/s", format_compact(n))
}

/// Format bytes in human-readable form.
///
/// ```text
/// format_bytes(0)              → "0 B"
/// format_bytes(512)            → "512 B"
/// format_bytes(1_536)          → "1.5 KB"
/// format_bytes(1_048_576)      → "1.0 MB"
/// format_bytes(1_073_741_824)  → "1.0 GB"
/// ```
pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1_024 {
        return format!("{} B", bytes);
    }
    if bytes < 1_048_576 {
        let v = bytes as f64 / 1_024.0;
        return if v >= 100.0 {
            format!("{:.0} KB", v)
        } else {
            format!("{:.1} KB", v)
        };
    }
    if bytes < 1_073_741_824 {
        let v = bytes as f64 / 1_048_576.0;
        return if v >= 100.0 {
            format!("{:.0} MB", v)
        } else {
            format!("{:.1} MB", v)
        };
    }
    let v = bytes as f64 / 1_073_741_824.0;
    if v >= 100.0 {
        format!("{:.0} GB", v)
    } else {
        format!("{:.1} GB", v)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── format_number ────────────────────────────────────────────────────

    #[test]
    fn test_format_number_small() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(42), "42");
        assert_eq!(format_number(999), "999");
    }

    #[test]
    fn test_format_number_thousands() {
        assert_eq!(format_number(1_000), "1,000");
        assert_eq!(format_number(1_234), "1,234");
        assert_eq!(format_number(12_345), "12,345");
        assert_eq!(format_number(123_456), "123,456");
    }

    #[test]
    fn test_format_number_millions() {
        assert_eq!(format_number(1_000_000), "1,000,000");
        assert_eq!(format_number(1_234_567), "1,234,567");
        assert_eq!(format_number(123_456_789), "123,456,789");
    }

    #[test]
    fn test_format_number_large() {
        assert_eq!(format_number(1_000_000_000), "1,000,000,000");
    }

    // ── format_compact ───────────────────────────────────────────────────

    #[test]
    fn test_format_compact_small() {
        assert_eq!(format_compact(0), "0");
        assert_eq!(format_compact(42), "42");
        assert_eq!(format_compact(999), "999");
    }

    #[test]
    fn test_format_compact_k() {
        assert_eq!(format_compact(1_000), "1.0K");
        assert_eq!(format_compact(1_500), "1.5K");
        assert_eq!(format_compact(42_800), "42.8K");
        assert_eq!(format_compact(999_999), "1000K");
    }

    #[test]
    fn test_format_compact_m() {
        assert_eq!(format_compact(1_000_000), "1.0M");
        assert_eq!(format_compact(1_234_567), "1.2M");
        assert_eq!(format_compact(42_000_000), "42.0M");
    }

    #[test]
    fn test_format_compact_g() {
        assert_eq!(format_compact(1_000_000_000), "1.0G");
        assert_eq!(format_compact(5_500_000_000), "5.5G");
    }

    // ── format_rate ──────────────────────────────────────────────────────

    #[test]
    fn test_format_rate_zero() {
        assert_eq!(format_rate(0, 1.0), "0/s");
        assert_eq!(format_rate(100, 0.0), "0/s");
    }

    #[test]
    fn test_format_rate_normal() {
        assert_eq!(format_rate(1500, 1.0), "1.5K/s");
        assert_eq!(format_rate(42, 1.0), "42/s");
        assert_eq!(format_rate(1_000_000, 1.0), "1.0M/s");
    }

    #[test]
    fn test_format_rate_elapsed() {
        assert_eq!(format_rate(100, 2.0), "50/s");
        assert_eq!(format_rate(10_000, 0.5), "20.0K/s");
    }

    // ── format_bytes ─────────────────────────────────────────────────────

    #[test]
    fn test_format_bytes_small() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1_023), "1023 B");
    }

    #[test]
    fn test_format_bytes_kb() {
        assert_eq!(format_bytes(1_024), "1.0 KB");
        assert_eq!(format_bytes(1_536), "1.5 KB");
        assert_eq!(format_bytes(102_400), "100 KB");
    }

    #[test]
    fn test_format_bytes_mb() {
        assert_eq!(format_bytes(1_048_576), "1.0 MB");
        assert_eq!(format_bytes(52_428_800), "50.0 MB");
    }

    #[test]
    fn test_format_bytes_gb() {
        assert_eq!(format_bytes(1_073_741_824), "1.0 GB");
        assert_eq!(format_bytes(5_368_709_120), "5.0 GB");
    }
}
