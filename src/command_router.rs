// prism/command_router — pure command parsing and dispatch
// zero I/O — matches input strings to commands, returns matches

use std::collections::HashMap;

/// Command definition (metadata only — handlers are wired by consumers)
pub struct Command {
    pub description: Option<String>,
    pub aliases: Vec<String>,
    pub hidden: bool,
}

/// Result of matching input to a command
pub struct CommandMatch<'a> {
    pub command: &'a Command,
    pub name: String,
    pub args: String,
}

/// Routes input strings to commands with prefix matching, aliases, and completions
pub struct CommandRouter {
    commands: Vec<(String, Command)>,
    lookup: HashMap<String, usize>,
    prefix: String,
}

impl CommandRouter {
    pub fn new(commands: Vec<(String, Command)>, prefix: &str) -> Self {
        let mut lookup = HashMap::new();
        for (i, (name, cmd)) in commands.iter().enumerate() {
            lookup.insert(name.clone(), i);
            for alias in &cmd.aliases {
                lookup.insert(alias.clone(), i);
            }
        }
        Self {
            commands,
            lookup,
            prefix: prefix.to_string(),
        }
    }

    /// Try to match input to a command. Returns None if no match.
    pub fn match_input(&self, input: &str) -> Option<CommandMatch<'_>> {
        if !input.starts_with(&self.prefix) {
            return None;
        }

        let rest = &input[self.prefix.len()..];
        let (cmd_name, args) = match rest.find(' ') {
            Some(idx) => (&rest[..idx], rest[idx + 1..].trim()),
            None => (rest, ""),
        };

        let &index = self.lookup.get(cmd_name)?;
        let (canonical_name, command) = &self.commands[index];

        Some(CommandMatch {
            command,
            name: canonical_name.clone(),
            args: args.to_string(),
        })
    }

    /// Return completions for partial input
    pub fn completions(&self, partial: &str) -> Vec<String> {
        if !partial.starts_with(&self.prefix) {
            return vec![];
        }

        let typed = &partial[self.prefix.len()..];
        self.commands
            .iter()
            .filter(|(name, cmd)| name.starts_with(typed) && !cmd.hidden)
            .map(|(name, _)| format!("{}{}", self.prefix, name))
            .collect()
    }

    /// Generate help text listing all visible commands
    pub fn help_text(&self) -> String {
        let visible: Vec<_> = self
            .commands
            .iter()
            .filter(|(_, cmd)| !cmd.hidden)
            .collect();

        if visible.is_empty() {
            return String::new();
        }

        let max_len = visible.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
        let pad_width = max_len + self.prefix.len() + 2;

        let lines: Vec<String> = visible
            .iter()
            .map(|(name, cmd)| {
                let alias_str = if cmd.aliases.is_empty() {
                    String::new()
                } else {
                    let aliases: Vec<String> = cmd
                        .aliases
                        .iter()
                        .map(|a| format!("{}{}", self.prefix, a))
                        .collect();
                    format!(" ({})", aliases.join(", "))
                };
                let desc = cmd.description.as_deref().unwrap_or("");
                let label = format!("{}{}", self.prefix, name);
                format!("  {:width$}{}{}", label, desc, alias_str, width = pad_width)
            })
            .collect();

        lines.join("\n")
    }
}
