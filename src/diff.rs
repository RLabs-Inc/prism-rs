use crate::style::s;
use crate::writer::ansi_enabled;

#[derive(Debug, Default)]
pub struct DiffOptions {
    pub filename: Option<String>,
    pub context: usize, // default 3
}

impl DiffOptions {
    pub fn new() -> Self {
        Self {
            filename: None,
            context: 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum DiffKind {
    Equal,
    Add,
    Remove,
}

#[derive(Debug, Clone)]
struct DiffLine {
    kind: DiffKind,
    old_num: Option<usize>,
    new_num: Option<usize>,
    text: String,
}

/// Compute LCS DP table for two sequences of lines.
fn compute_lcs<'a>(old: &[&'a str], new: &[&'a str]) -> Vec<Vec<usize>> {
    let m = old.len();
    let n = new.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if old[i - 1] == new[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    dp
}

/// Backtrack LCS table to produce diff hunks.
#[allow(clippy::too_many_arguments)]
fn backtrack<'a>(
    dp: &[Vec<usize>],
    old: &[&'a str],
    new: &[&'a str],
    i: usize,
    j: usize,
    result: &mut Vec<DiffLine>,
    old_counter: &mut usize,
    new_counter: &mut usize,
) {
    if i == 0 && j == 0 {
        return;
    }
    if i > 0 && j > 0 && old[i - 1] == new[j - 1] {
        backtrack(dp, old, new, i - 1, j - 1, result, old_counter, new_counter);
        *old_counter += 1;
        *new_counter += 1;
        result.push(DiffLine {
            kind: DiffKind::Equal,
            old_num: Some(*old_counter),
            new_num: Some(*new_counter),
            text: old[i - 1].to_string(),
        });
    } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
        backtrack(dp, old, new, i, j - 1, result, old_counter, new_counter);
        *new_counter += 1;
        result.push(DiffLine {
            kind: DiffKind::Add,
            old_num: None,
            new_num: Some(*new_counter),
            text: new[j - 1].to_string(),
        });
    } else {
        backtrack(dp, old, new, i - 1, j, result, old_counter, new_counter);
        *old_counter += 1;
        result.push(DiffLine {
            kind: DiffKind::Remove,
            old_num: Some(*old_counter),
            new_num: None,
            text: old[i - 1].to_string(),
        });
    }
}

/// Compute the full diff between two multi-line strings.
fn compute_diff(old_text: &str, new_text: &str) -> Vec<DiffLine> {
    let old_lines: Vec<&str> = old_text.split('\n').collect();
    let new_lines: Vec<&str> = new_text.split('\n').collect();

    let dp = compute_lcs(&old_lines, &new_lines);

    let m = old_lines.len();
    let n = new_lines.len();

    let mut result = Vec::new();
    let mut old_counter = 0usize;
    let mut new_counter = 0usize;

    backtrack(
        &dp,
        &old_lines,
        &new_lines,
        m,
        n,
        &mut result,
        &mut old_counter,
        &mut new_counter,
    );

    result
}

/// Format a diff line number for display (right-padded to width 4).
fn fmt_num(num: Option<usize>) -> String {
    match num {
        Some(n) => format!("{:4}", n),
        None => "    ".to_string(),
    }
}

/// Generate a unified-diff style string between old_text and new_text.
pub fn diff(old_text: &str, new_text: &str, options: &DiffOptions) -> String {
    let context = if options.context == 0 { 3 } else { options.context };
    let lines = compute_diff(old_text, new_text);

    let has_changes = lines.iter().any(|l| l.kind != DiffKind::Equal);

    let mut out = String::new();

    // Header
    if let Some(ref fname) = options.filename {
        if ansi_enabled() {
            out.push_str(&s().bold().paint(&format!("--- {}\n+++ {}\n", fname, fname)));
        } else {
            out.push_str(&format!("--- {}\n+++ {}\n", fname, fname));
        }
    }

    if !has_changes {
        if ansi_enabled() {
            out.push_str(&s().dim().paint("(no changes)"));
        } else {
            out.push_str("(no changes)");
        }
        return out;
    }

    // Find indices of changed lines for context window filtering.
    let changed_indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.kind != DiffKind::Equal)
        .map(|(i, _)| i)
        .collect();

    // Build a set of line indices to include (changed lines + context around them).
    let mut included = vec![false; lines.len()];
    for &ci in &changed_indices {
        let start = ci.saturating_sub(context);
        let end = (ci + context + 1).min(lines.len());
        for item in included.iter_mut().take(end).skip(start) {
            *item = true;
        }
    }

    let mut prev_included = false;
    for (i, line) in lines.iter().enumerate() {
        if !included[i] {
            prev_included = false;
            continue;
        }

        // Separator between non-contiguous hunks
        if !prev_included && i > 0 {
            if ansi_enabled() {
                out.push_str(&s().cyan().paint("@@ ... @@\n"));
            } else {
                out.push_str("@@ ... @@\n");
            }
        }
        prev_included = true;

        let old_n = fmt_num(line.old_num);
        let new_n = fmt_num(line.new_num);

        let formatted = match line.kind {
            DiffKind::Add => {
                let prefix = format!("{} {} + ", old_n, new_n);
                let content = format!("{}{}", prefix, line.text);
                if ansi_enabled() {
                    s().green().paint(&content)
                } else {
                    content
                }
            }
            DiffKind::Remove => {
                let prefix = format!("{} {} - ", old_n, new_n);
                let content = format!("{}{}", prefix, line.text);
                if ansi_enabled() {
                    s().red().paint(&content)
                } else {
                    content
                }
            }
            DiffKind::Equal => {
                let prefix = format!("{} {}   ", old_n, new_n);
                let content = format!("{}{}", prefix, line.text);
                if ansi_enabled() {
                    s().dim().paint(&content)
                } else {
                    content
                }
            }
        };

        out.push_str(&formatted);
        out.push('\n');
    }

    // Remove trailing newline
    if out.ends_with('\n') {
        out.pop();
    }

    out
}
