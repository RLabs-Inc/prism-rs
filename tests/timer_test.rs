use prism::timer::format_time;

#[test]
fn format_time_ms() { assert_eq!(format_time(42), "42ms"); }

#[test]
fn format_time_seconds() { assert_eq!(format_time(1200), "1.2s"); }

#[test]
fn format_time_minutes() { assert_eq!(format_time(90000), "1m 30s"); }

#[test]
fn format_time_hours() { assert_eq!(format_time(3_660_000), "1h 1m"); }

#[test]
fn format_time_zero() { assert_eq!(format_time(0), "0ms"); }

#[test]
fn format_time_exact_second() { assert_eq!(format_time(1000), "1.0s"); }
