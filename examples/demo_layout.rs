use prism::{layout, s, writeln, LayoutActivityOptions, LayoutSectionOptions};
use std::thread;
use std::time::Duration;

fn main() {
    writeln("=== Prism Layout Demo (Two-Zone Architecture) ===\n");

    let ly = layout(None);

    // Activity: scanning
    let scan = ly.activity(
        "Scanning networks...",
        Some(LayoutActivityOptions {
            timer: true,
            ..Default::default()
        }),
    );

    for i in 1..=5 {
        thread::sleep(Duration::from_millis(400));
        ly.print(&format!(
            "  {} Found AP: {}",
            s().dim().paint(&format!("[{}]", i)),
            s().cyan().paint(&format!("Network-{}", i))
        ));
    }

    scan.done(Some("Found 5 networks"));

    // Section: attack progress
    let section = ly.section(
        "Running PMKID attack...",
        Some(LayoutSectionOptions {
            timer: true,
            ..Default::default()
        }),
    );

    thread::sleep(Duration::from_millis(600));
    section.add(&format!(
        "{} Sending auth to Network-1",
        s().dim().paint(">>")
    ));
    thread::sleep(Duration::from_millis(600));
    section.add(&format!("{} PMKID captured!", s().green().paint(">>")));
    thread::sleep(Duration::from_millis(400));
    section.done(Some("1 PMKID captured"));

    ly.print("");
    ly.print(&format!(
        "{} All operations complete.",
        s().green().bold().paint("✓")
    ));

    ly.close(None);
}
