// prism/args — declarative CLI argument parsing
// thin custom parser with auto-generated help
// uses prism's own display primitives for formatting

use crate::style::s;
use crate::writer::ansi_enabled;
use std::collections::HashMap;

/// Flag definition
#[derive(Clone)]
pub struct FlagDef {
    pub flag_type: FlagType,
    pub short: Option<char>,
    pub description: Option<String>,
    pub default: Option<FlagValue>,
    pub required: bool,
    pub placeholder: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FlagType {
    String,
    Boolean,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FlagValue {
    String(String),
    Boolean(bool),
}

impl FlagValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            FlagValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FlagValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

impl std::fmt::Display for FlagValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlagValue::String(s) => write!(f, "{}", s),
            FlagValue::Boolean(b) => write!(f, "{}", b),
        }
    }
}

/// Command definition
#[derive(Clone)]
pub struct CommandDef {
    pub description: Option<String>,
    pub flags: Vec<(String, FlagDef)>,
    pub usage: Option<String>,
    pub hidden: bool,
}

/// CLI configuration
#[derive(Default)]
pub struct ArgsConfig {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub commands: Vec<(String, CommandDef)>,
    pub flags: Vec<(String, FlagDef)>,
    pub usage: Option<String>,
    pub examples: Vec<String>,
    pub argv: Option<Vec<String>>,
    pub no_exit: bool,
    pub allow_no_command: bool,
}

/// Parsed result
pub struct ArgsResult {
    pub command: Option<String>,
    pub flags: HashMap<String, FlagValue>,
    pub args: Vec<String>,
    config: ArgsConfig,
    command_def: Option<CommandDef>,
}

impl ArgsResult {
    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.flags.get(name).and_then(|v| v.as_str())
    }

    pub fn get_bool(&self, name: &str) -> bool {
        self.flags
            .get(name)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn show_help(&self) {
        print_help(
            &self.config,
            self.command.as_deref(),
            self.command_def.as_ref(),
        );
    }

    pub fn show_version(&self) {
        let version = self.config.version.as_deref().unwrap_or("0.0.0");
        println!("{} {}", self.config.name, version);
    }
}

// --- Help formatter ---

fn format_flag(name: &str, def: &FlagDef) -> (String, String) {
    let short = match def.short {
        Some(c) => format!("-{}, ", c),
        None => "    ".to_string(),
    };
    let long = format!("--{}", name);
    let placeholder = if def.flag_type == FlagType::String {
        format!(" <{}>", def.placeholder.as_deref().unwrap_or(name))
    } else {
        String::new()
    };

    let left = format!("{}{}{}", short, long, placeholder);

    let mut right = def.description.clone().unwrap_or_default();
    if let Some(ref default) = def.default {
        if *default != FlagValue::Boolean(false) {
            let dim = s().dim().paint(&format!("(default: {})", default));
            if right.is_empty() {
                right = dim;
            } else {
                right = format!("{} {}", right, dim);
            }
        }
    }
    if def.required {
        let dim = s().dim().paint("(required)");
        if right.is_empty() {
            right = dim;
        } else {
            right = format!("{} {}", right, dim);
        }
    }

    (left, right)
}

fn print_help(config: &ArgsConfig, command: Option<&str>, command_def: Option<&CommandDef>) {
    let out = |text: &str| println!("{}", text);

    // Header
    let version = config
        .version
        .as_ref()
        .map(|v| format!(" {}", s().dim().paint(&format!("v{}", v))))
        .unwrap_or_default();
    let desc = config
        .description
        .as_ref()
        .map(|d| s().dim().paint(&format!(" — {}", d)))
        .unwrap_or_default();

    if let (Some(cmd), Some(cmd_def)) = (command, command_def) {
        let cmd_desc = cmd_def
            .description
            .as_ref()
            .map(|d| s().dim().paint(&format!(" — {}", d)))
            .unwrap_or_default();
        out(&format!(
            "\n  {} {}{}",
            s().bold().paint(&config.name),
            s().cyan().paint(cmd),
            cmd_desc
        ));
    } else {
        out(&format!(
            "\n  {}{}{}",
            s().bold().paint(&config.name),
            version,
            desc
        ));
    }

    // Usage
    println!();
    out(&format!("  {}", s().dim().paint("USAGE")));
    if let (Some(cmd), Some(cmd_def)) = (command, command_def) {
        let cmd_usage = cmd_def
            .usage
            .as_ref()
            .map(|u| format!(" {}", u))
            .unwrap_or_default();
        out(&format!("    {} {}{} [flags]", config.name, cmd, cmd_usage));
    } else if !config.commands.is_empty() {
        let usage = config
            .usage
            .as_ref()
            .map(|u| format!(" {}", u))
            .unwrap_or_default();
        out(&format!("    {} <command>{} [flags]", config.name, usage));
    } else {
        let usage = config
            .usage
            .as_ref()
            .map(|u| format!(" {}", u))
            .unwrap_or_default();
        out(&format!("    {}{} [flags]", config.name, usage));
    }

    // Commands (only for top-level help)
    if command.is_none() && !config.commands.is_empty() {
        let visible: Vec<_> = config
            .commands
            .iter()
            .filter(|(_, def)| !def.hidden)
            .collect();
        if !visible.is_empty() {
            println!();
            out(&format!("  {}", s().dim().paint("COMMANDS")));
            let max_len = visible.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
            for (name, def) in &visible {
                let desc = def
                    .description
                    .as_ref()
                    .map(|d| s().dim().paint(d))
                    .unwrap_or_default();
                out(&format!(
                    "    {}  {}",
                    s().cyan()
                        .paint(&format!("{:width$}", name, width = max_len + 2)),
                    desc
                ));
            }
        }
    }

    // Flags
    let print_flags = |label: &str, flags: &[(String, FlagDef)]| {
        if flags.is_empty() {
            return;
        }
        let formatted: Vec<_> = flags
            .iter()
            .map(|(name, def)| format_flag(name, def))
            .collect();
        let max_left = formatted
            .iter()
            .map(|(left, _)| {
                if ansi_enabled() {
                    crate::ansi::measure_width(left)
                } else {
                    left.len()
                }
            })
            .max()
            .unwrap_or(0);

        println!();
        out(&format!("  {}", s().dim().paint(label)));
        for (left, right) in &formatted {
            let left_width = if ansi_enabled() {
                crate::ansi::measure_width(left)
            } else {
                left.len()
            };
            let padding = " ".repeat(max_left.saturating_sub(left_width) + 2);
            out(&format!(
                "    {}{}  {}",
                s().yellow().paint(left),
                padding,
                right
            ));
        }
    };

    if command.is_some() {
        if let Some(cmd_def) = command_def {
            print_flags("FLAGS", &cmd_def.flags);
        }
        if !config.flags.is_empty() {
            print_flags("GLOBAL FLAGS", &config.flags);
        }
    } else {
        let mut all_flags = config.flags.clone();
        if !all_flags.iter().any(|(n, _)| n == "help") {
            all_flags.push((
                "help".into(),
                FlagDef {
                    flag_type: FlagType::Boolean,
                    short: Some('h'),
                    description: Some("Show help".into()),
                    default: None,
                    required: false,
                    placeholder: None,
                },
            ));
        }
        if config.version.is_some() && !all_flags.iter().any(|(n, _)| n == "version") {
            all_flags.push((
                "version".into(),
                FlagDef {
                    flag_type: FlagType::Boolean,
                    short: None,
                    description: Some("Show version".into()),
                    default: None,
                    required: false,
                    placeholder: None,
                },
            ));
        }
        print_flags("FLAGS", &all_flags);
    }

    // Examples
    if command.is_none() && !config.examples.is_empty() {
        println!();
        out(&format!("  {}", s().dim().paint("EXAMPLES")));
        for example in &config.examples {
            out(&format!("    {} {}", s().dim().paint("$"), example));
        }
    }

    // Footer
    if command.is_none() && !config.commands.is_empty() {
        println!();
        out(&s().dim().paint(&format!(
            "  Run '{} <command> --help' for command-specific flags.",
            config.name
        )));
    }

    println!();
}

// --- Parser ---

pub fn args(config: ArgsConfig) -> ArgsResult {
    let argv = config
        .argv
        .clone()
        .unwrap_or_else(|| std::env::args().skip(1).collect());

    // Build short flag lookup
    let mut short_to_long: HashMap<char, String> = HashMap::new();
    let mut flag_types: HashMap<String, FlagType> = HashMap::new();

    for (name, def) in &config.flags {
        if let Some(c) = def.short {
            short_to_long.insert(c, name.clone());
        }
        flag_types.insert(name.clone(), def.flag_type.clone());
    }

    // Also register command-specific flags for type info
    for (_, cmd_def) in &config.commands {
        for (name, def) in &cmd_def.flags {
            if let Some(c) = def.short {
                short_to_long.entry(c).or_insert_with(|| name.clone());
            }
            flag_types
                .entry(name.clone())
                .or_insert_with(|| def.flag_type.clone());
        }
    }

    // Built-in flags
    if !flag_types.contains_key("help") {
        flag_types.insert("help".into(), FlagType::Boolean);
        short_to_long.entry('h').or_insert_with(|| "help".into());
    }
    if config.version.is_some() && !flag_types.contains_key("version") {
        flag_types.insert("version".into(), FlagType::Boolean);
    }

    // Parse argv
    let mut flags: HashMap<String, FlagValue> = HashMap::new();
    let mut positionals: Vec<String> = Vec::new();
    let mut i = 0;
    let mut past_double_dash = false;

    while i < argv.len() {
        let arg = &argv[i];

        if past_double_dash {
            positionals.push(arg.clone());
            i += 1;
            continue;
        }

        if arg == "--" {
            past_double_dash = true;
            i += 1;
            continue;
        }

        if let Some(rest) = arg.strip_prefix("--") {
            // Long flag
            if let Some((name, value)) = rest.split_once('=') {
                flags.insert(name.to_string(), FlagValue::String(value.to_string()));
            } else {
                let name = rest.to_string();
                match flag_types.get(&name) {
                    Some(FlagType::String) => {
                        i += 1;
                        if i < argv.len() {
                            flags.insert(name, FlagValue::String(argv[i].clone()));
                        }
                    }
                    _ => {
                        flags.insert(name, FlagValue::Boolean(true));
                    }
                }
            }
        } else if let Some(rest) = arg.strip_prefix('-') {
            // Short flags
            let chars: Vec<char> = rest.chars().collect();
            for (j, &c) in chars.iter().enumerate() {
                if let Some(long) = short_to_long.get(&c) {
                    match flag_types.get(long) {
                        Some(FlagType::String) => {
                            // Rest of chars is the value, or next arg
                            if j + 1 < chars.len() {
                                let value: String = chars[j + 1..].iter().collect();
                                flags.insert(long.clone(), FlagValue::String(value));
                            } else {
                                i += 1;
                                if i < argv.len() {
                                    flags.insert(long.clone(), FlagValue::String(argv[i].clone()));
                                }
                            }
                            break;
                        }
                        _ => {
                            flags.insert(long.clone(), FlagValue::Boolean(true));
                        }
                    }
                } else {
                    // Unknown short flag — treat as boolean
                    flags.insert(c.to_string(), FlagValue::Boolean(true));
                }
            }
        } else {
            positionals.push(arg.clone());
        }

        i += 1;
    }

    // Detect command
    let mut command: Option<String> = None;
    let mut command_def: Option<CommandDef> = None;

    if !config.commands.is_empty() {
        if let Some(first) = positionals.first() {
            if let Some((_, def)) = config.commands.iter().find(|(n, _)| n == first) {
                command = Some(first.clone());
                command_def = Some(def.clone());
            }
        }
    }

    // Apply defaults
    for (name, def) in &config.flags {
        if let Some(ref default) = def.default {
            flags.entry(name.clone()).or_insert_with(|| default.clone());
        }
    }
    if let Some(ref cmd_def) = command_def {
        for (name, def) in &cmd_def.flags {
            if let Some(ref default) = def.default {
                flags.entry(name.clone()).or_insert_with(|| default.clone());
            }
        }
    }

    // Strip command from positionals
    let args_positionals =
        if command.is_some() && positionals.first().map(|s| s.as_str()) == command.as_deref() {
            positionals[1..].to_vec()
        } else {
            positionals
        };

    let result = ArgsResult {
        command: command.clone(),
        flags,
        args: args_positionals,
        config,
        command_def: command_def.clone(),
    };

    // Auto-handle --version
    if result.get_bool("version") && result.config.version.is_some() {
        result.show_version();
        if !result.config.no_exit {
            std::process::exit(0);
        }
        return result;
    }

    // Auto-handle --help
    if result.get_bool("help") {
        result.show_help();
        if !result.config.no_exit {
            std::process::exit(0);
        }
        return result;
    }

    // No command given but commands defined → show help
    if !result.config.commands.is_empty()
        && command.is_none()
        && result.args.is_empty()
        && !result.config.allow_no_command
    {
        result.show_help();
        if !result.config.no_exit {
            std::process::exit(0);
        }
        return result;
    }

    // Unknown command
    if !result.config.commands.is_empty() && command.is_none() && !result.args.is_empty() {
        let unknown = &result.args[0];
        let available: Vec<_> = result
            .config
            .commands
            .iter()
            .filter(|(_, d)| !d.hidden)
            .map(|(n, _)| n.as_str())
            .collect();
        eprintln!(
            "{} Unknown command: {}",
            s().red().paint("✗"),
            s().bold().paint(unknown)
        );
        eprintln!(
            "{}",
            s().dim()
                .paint(&format!("  Available: {}", available.join(", ")))
        );
        eprintln!(
            "{}",
            s().dim()
                .paint(&format!("  Run '{} --help' for usage.", result.config.name))
        );
        if !result.config.no_exit {
            std::process::exit(1);
        }
        return result;
    }

    // Validate required flags
    let effective_flags: Vec<_> = {
        let mut all = result.config.flags.clone();
        if let Some(ref cmd_def) = command_def {
            all.extend(cmd_def.flags.clone());
        }
        all
    };

    for (name, def) in &effective_flags {
        if def.required && !result.flags.contains_key(name) {
            eprintln!(
                "{} Missing required flag: {}",
                s().red().paint("✗"),
                s().yellow().paint(&format!("--{}", name))
            );
            let cmd_str = command
                .as_ref()
                .map(|c| format!(" {}", c))
                .unwrap_or_default();
            eprintln!(
                "{}",
                s().dim().paint(&format!(
                    "  Run '{}{} --help' for usage.",
                    result.config.name, cmd_str
                ))
            );
            if !result.config.no_exit {
                std::process::exit(1);
            }
            return result;
        }
    }

    result
}
