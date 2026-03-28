use prism::{color, hex, rgb, s, writeln, RESET};

fn main() {
    writeln("=== Prism Style Demo ===\n");

    // ANSI 16 colors — respects terminal theme
    writeln(&format!(
        "{}  {}  {}  {}  {}",
        s().red().paint("red"),
        s().green().paint("green"),
        s().blue().paint("blue"),
        s().yellow().paint("yellow"),
        s().cyan().paint("cyan"),
    ));

    // Bright variants
    writeln(&format!(
        "{}  {}  {}  {}  {}",
        s().bright_red().paint("bright_red"),
        s().bright_green().paint("bright_green"),
        s().bright_blue().paint("bright_blue"),
        s().bright_yellow().paint("bright_yellow"),
        s().bright_cyan().paint("bright_cyan"),
    ));

    // Modifiers
    writeln(&format!(
        "\n{}  {}  {}  {}  {}",
        s().bold().paint("bold"),
        s().dim().paint("dim"),
        s().italic().paint("italic"),
        s().underline().paint("underline"),
        s().strikethrough().paint("strikethrough"),
    ));

    // Composable chains
    writeln(&s().bold().red().underline().paint("bold + red + underline"));
    writeln(&s().bg_blue().white().bold().paint(" BADGE "));

    // Exact RGB colors
    writeln(&format!(
        "\n{}",
        s().fg(rgb(255, 87, 51)).paint("exact orange (255, 87, 51)")
    ));
    writeln(&s().fg(hex(0x7C3AED)).paint("exact purple (#7C3AED)"));
    writeln(
        &s().bg_black()
            .fg(rgb(200, 200, 200))
            .paint(" dark bg + light fg "),
    );

    // color() convenience function
    writeln(&format!(
        "\n{}",
        color("convenience color()", rgb(0, 200, 100), None)
    ));

    // RESET constant
    writeln(&format!("\nRESET sequence: {:?}", RESET));
}
