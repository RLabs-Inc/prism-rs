# prism-cli

CLI primitives for hackers — light through a prism, data through the terminal.

A comprehensive Rust library for building interactive command-line applications. No async runtime. No alternate screen. Just clean, composable primitives that respect your terminal.

## Install

```toml
[dependencies]
rlabs-prism = "0.1"
```

## Philosophy

- **CLI, not TUI.** Output stays in scrollback. Pipes work. No alternate screen.
- **Two-zone layout.** Streaming output above, pinned status bar below.
- **Pipe-aware.** Auto-detect TTY. Strip colors when piped. Links show URLs.
- **Zero async.** Threads, not tokio. This is systems code.
- **Minimal deps.** Only crossterm, libc, unicode-width, unicode-segmentation.

## What's Inside

### Foundation
ANSI escape handling, cursor control, 256/RGB color styling, text utilities (truncate, pad, wrap, indent, hyperlinks), Unicode grapheme segmentation, terminal detection (TTY, pipe, width/height).

### Composition Layer
Two-zone `Layout` with output zone + pinned status bar. `LiveBlock` for atomic redraws with DEC 2026 synchronized output. `Stream` for append-only scrolling output. `Activity` and `Section` for spinner-driven live status blocks.

### Display Components
`table` — aligned columns with borders and separators.
`frame` — bordered boxes (single, double, rounded, bold, ASCII).
`flex` — responsive multi-block layout (start, center, stretch, space-between, space-evenly).
`scroll` — fixed-header scrollable tables and views.
`banner` — ASCII art text with multiple font styles.
`braille` — Unicode braille plotting from boolean grids.
`highlight` — syntax highlighting for 15+ languages.
`diff` — unified diff with +/- coloring.
`columns` — side-by-side text columns.
`list` — bullet, numbered, alpha, arrow, tree, key-value lists.
`markdown` — terminal markdown rendering.

### Animation & Feedback
`spinner` — 44 spinner animations (dots, line, arc, bounce, braille, etc.).
`progress` / `progress_bar` — 10 bar styles with percentage and ETA.
`badge` — labeled status badges (pill, bracket, dot).
`timer` — stopwatch, countdown, benchmarking, time formatting.
`number` — thousand separators, compact notation (1.2M), byte formatting, rate display.

### Interactive Input
`repl` — full REPL with tab completion, history, slash commands.
`prompt` — confirm, input, password, select, multiselect prompts.
`line_editor` — readline-style editing with word navigation and history.
`keypress` — raw keyboard input, escape sequence parsing.
`args` — lightweight argument parser with commands, flags, and positionals.
`command_router` — slash command routing with aliases and help generation.
`exec` — live process output capture with scrollable viewport.

## Quick Example

```rust
use prism::{s, table, Column, Align, TableOptions, frame, FrameOptions, BorderStyle};

// Styled text
let title = s().bold().cyan().paint("scan results");
println!("{title}");

// Data table
let cols = vec![
    Column { title: "BSSID".into(), align: Align::Left, width: None },
    Column { title: "SSID".into(), align: Align::Left, width: None },
    Column { title: "Ch".into(), align: Align::Right, width: None },
    Column { title: "dBm".into(), align: Align::Right, width: None },
];
let rows = vec![
    vec!["7C:10:C9:03:10:E0", "RL-WiFi", "11", "-42"],
    vec!["AA:BB:CC:DD:EE:FF", "Neighbor", "6", "-67"],
];
let output = table(&cols, &rows, &TableOptions::default());
for line in &output {
    println!("{line}");
}

// Framed output
let framed = frame(&output, &FrameOptions {
    border: BorderStyle::Rounded,
    ..Default::default()
});
for line in &framed {
    println!("{line}");
}
```

## Used By

- [wifikit](https://github.com/RLabs-Inc/wifikit) — WiFi pentesting engine & interactive CLI

## License

MIT
