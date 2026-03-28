use prism::{activity, writeln, ActivityOptions};
use std::thread;
use std::time::Duration;

fn main() {
    writeln("=== Prism Spinner Demo ===\n");

    // Default spinner (dots)
    let act = activity(
        "Loading configuration...",
        ActivityOptions {
            timer: true,
            ..Default::default()
        },
    );
    thread::sleep(Duration::from_secs(2));
    act.done(Some("Configuration loaded"));

    // Updating text
    let act = activity(
        "Compiling modules...",
        ActivityOptions {
            timer: true,
            ..Default::default()
        },
    );
    thread::sleep(Duration::from_millis(1500));
    act.text("Almost there...");
    thread::sleep(Duration::from_millis(1000));
    act.done(Some("3 modules compiled"));

    // Failure case
    let act = activity(
        "Connecting to server...",
        ActivityOptions {
            timer: true,
            ..Default::default()
        },
    );
    thread::sleep(Duration::from_secs(2));
    act.fail(Some("Connection refused"));

    // Warning
    let act = activity(
        "Checking dependencies...",
        ActivityOptions {
            timer: true,
            ..Default::default()
        },
    );
    thread::sleep(Duration::from_millis(1500));
    act.warn(Some("2 outdated packages"));

    writeln("\nDone!");
}
