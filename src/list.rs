use crate::ansi::measure_width;
use crate::style::s;

// ── List ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Default)]
pub enum ListStyle {
    #[default]
    Bullet,
    Dash,
    Numbered,
    Alpha,
    Arrow,
    Star,
    Check,
}

pub struct ListOptions {
    pub style: ListStyle,
    pub indent: usize,
}

impl Default for ListOptions {
    fn default() -> Self {
        Self { style: ListStyle::Bullet, indent: 0 }
    }
}

/// Render a list with the given style. Returns a multi-line String without a
/// trailing newline.
pub fn list(items: &[&str], options: &ListOptions) -> String {
    if items.is_empty() {
        return String::new();
    }

    let indent = " ".repeat(options.indent);

    // For numbered / alpha we need a fixed-width marker column.
    let max_index_width = match options.style {
        ListStyle::Numbered => {
            let label = format!("{}.", items.len());
            measure_width(&label)
        }
        ListStyle::Alpha => {
            // e.g. "z." for 26 items — simplify to 2 chars
            let last = (b'a' + (items.len().saturating_sub(1) as u8).min(25)) as char;
            measure_width(&format!("{}.", last))
        }
        _ => 0,
    };

    let lines: Vec<String> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let marker = match options.style {
                ListStyle::Bullet => s().cyan().paint("•"),
                ListStyle::Dash => s().gray().paint("-"),
                ListStyle::Arrow => s().cyan().paint("→"),
                ListStyle::Star => s().yellow().paint("★"),
                ListStyle::Check => s().green().paint("✓"),
                ListStyle::Numbered => {
                    let label = format!("{}.", i + 1);
                    let padded = format!("{:>width$}", label, width = max_index_width);
                    s().bold().paint(&padded)
                }
                ListStyle::Alpha => {
                    let ch = (b'a' + (i as u8).min(25)) as char;
                    let label = format!("{}.", ch);
                    let padded = format!("{:>width$}", label, width = max_index_width);
                    s().bold().paint(&padded)
                }
            };
            format!("{}{} {}", indent, marker, item)
        })
        .collect();

    lines.join("\n")
}

// ── Key-Value ─────────────────────────────────────────────────────────────────

pub struct KvOptions {
    pub separator: String,
    pub indent: usize,
}

impl Default for KvOptions {
    fn default() -> Self {
        Self { separator: "  ".to_string(), indent: 0 }
    }
}

/// Render key-value pairs with aligned keys. Returns a multi-line String
/// without a trailing newline.
pub fn kv(pairs: &[(&str, &str)], options: &KvOptions) -> String {
    if pairs.is_empty() {
        return String::new();
    }

    let max_key_width = pairs
        .iter()
        .map(|(k, _)| measure_width(k))
        .max()
        .unwrap_or(0);

    let indent = " ".repeat(options.indent);

    let lines: Vec<String> = pairs
        .iter()
        .map(|(key, val)| {
            let key_width = measure_width(key);
            let padding = " ".repeat(max_key_width - key_width);
            let styled_key = s().bold().paint(key);
            format!("{}{}{}{}{}", indent, styled_key, padding, options.separator, val)
        })
        .collect();

    lines.join("\n")
}

// ── Tree ──────────────────────────────────────────────────────────────────────

pub enum TreeNode {
    File(String),
    Dir(String, Vec<TreeNode>),
}

impl TreeNode {
    pub fn file(name: &str) -> Self {
        TreeNode::File(name.to_string())
    }

    pub fn dir(name: &str, children: Vec<TreeNode>) -> Self {
        TreeNode::Dir(name.to_string(), children)
    }
}

/// Render a tree of nodes with guide lines. Returns a multi-line String
/// without a trailing newline.
pub fn tree(nodes: &[TreeNode]) -> String {
    let mut out = String::new();
    render_nodes(nodes, "", &mut out);
    // Remove trailing newline if present
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

fn render_nodes(nodes: &[TreeNode], prefix: &str, out: &mut String) {
    let last_idx = nodes.len().saturating_sub(1);
    for (i, node) in nodes.iter().enumerate() {
        let is_last = i == last_idx;
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last { "    " } else { "│   " };

        match node {
            TreeNode::File(name) => {
                out.push_str(prefix);
                out.push_str(connector);
                out.push_str(name);
                out.push('\n');
            }
            TreeNode::Dir(name, children) => {
                out.push_str(prefix);
                out.push_str(connector);
                // Bold directory name with trailing slash
                let dir_label = s().bold().paint(&format!("{}/", name));
                out.push_str(&dir_label);
                out.push('\n');
                let new_prefix = format!("{}{}", prefix, child_prefix);
                render_nodes(children, &new_prefix, out);
            }
        }
    }
}
