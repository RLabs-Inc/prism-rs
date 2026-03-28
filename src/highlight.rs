use crate::style::s;
use crate::writer::ansi_enabled;

// ---------------------------------------------------------------------------
// Language enum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default)]
pub enum Language {
    #[default]
    Auto,
    TypeScript,
    JavaScript,
    Json,
    Bash,
    Sql,
    GraphQl,
    Rust,
}

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

pub struct HighlightOptions {
    pub language: Language,
    pub line_numbers: bool,
    pub start_line: usize,
}

impl Default for HighlightOptions {
    fn default() -> Self {
        Self {
            language: Language::Auto,
            line_numbers: false,
            start_line: 1,
        }
    }
}

// ---------------------------------------------------------------------------
// Keyword / builtin lists per language
// ---------------------------------------------------------------------------

fn keywords_for(lang: Language) -> &'static [&'static str] {
    match lang {
        Language::Rust => &[
            "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
            "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
            "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super",
            "trait", "true", "type", "unsafe", "use", "where", "while", "box", "do", "final",
            "macro", "override", "priv", "try", "typeof", "unsized", "virtual", "yield",
        ],
        Language::TypeScript => &[
            "abstract",
            "any",
            "as",
            "async",
            "await",
            "boolean",
            "break",
            "case",
            "catch",
            "class",
            "const",
            "constructor",
            "continue",
            "debugger",
            "declare",
            "default",
            "delete",
            "do",
            "else",
            "enum",
            "export",
            "extends",
            "false",
            "finally",
            "for",
            "from",
            "function",
            "get",
            "if",
            "implements",
            "import",
            "in",
            "instanceof",
            "interface",
            "is",
            "keyof",
            "let",
            "module",
            "namespace",
            "never",
            "new",
            "null",
            "number",
            "of",
            "override",
            "package",
            "private",
            "protected",
            "public",
            "readonly",
            "return",
            "set",
            "static",
            "string",
            "super",
            "switch",
            "symbol",
            "this",
            "throw",
            "true",
            "try",
            "type",
            "typeof",
            "undefined",
            "unknown",
            "var",
            "void",
            "while",
            "with",
            "yield",
        ],
        Language::JavaScript => &[
            "async",
            "await",
            "break",
            "case",
            "catch",
            "class",
            "const",
            "continue",
            "debugger",
            "default",
            "delete",
            "do",
            "else",
            "export",
            "extends",
            "false",
            "finally",
            "for",
            "from",
            "function",
            "get",
            "if",
            "import",
            "in",
            "instanceof",
            "let",
            "new",
            "null",
            "of",
            "return",
            "set",
            "static",
            "super",
            "switch",
            "this",
            "throw",
            "true",
            "try",
            "typeof",
            "undefined",
            "var",
            "void",
            "while",
            "with",
            "yield",
        ],
        Language::Json => &["true", "false", "null"],
        Language::Bash => &[
            "if", "then", "else", "elif", "fi", "for", "in", "do", "done", "while", "until",
            "case", "esac", "function", "return", "local", "export", "readonly", "declare",
            "typeset", "unset", "shift", "break", "continue", "exit", "trap", "exec", "eval",
            "source", "set", "unset", "true", "false",
        ],
        Language::Sql => &[
            "SELECT",
            "FROM",
            "WHERE",
            "JOIN",
            "LEFT",
            "RIGHT",
            "INNER",
            "OUTER",
            "FULL",
            "CROSS",
            "ON",
            "AS",
            "INSERT",
            "INTO",
            "VALUES",
            "UPDATE",
            "SET",
            "DELETE",
            "CREATE",
            "TABLE",
            "VIEW",
            "INDEX",
            "DROP",
            "ALTER",
            "ADD",
            "COLUMN",
            "PRIMARY",
            "KEY",
            "FOREIGN",
            "REFERENCES",
            "UNIQUE",
            "NOT",
            "NULL",
            "DEFAULT",
            "CONSTRAINT",
            "CHECK",
            "AND",
            "OR",
            "IN",
            "EXISTS",
            "BETWEEN",
            "LIKE",
            "IS",
            "DISTINCT",
            "ORDER",
            "BY",
            "GROUP",
            "HAVING",
            "LIMIT",
            "OFFSET",
            "UNION",
            "ALL",
            "EXCEPT",
            "INTERSECT",
            "CASE",
            "WHEN",
            "THEN",
            "ELSE",
            "END",
            "BEGIN",
            "COMMIT",
            "ROLLBACK",
            "TRANSACTION",
            "WITH",
            "RECURSIVE",
            // lowercase variants
            "select",
            "from",
            "where",
            "join",
            "left",
            "right",
            "inner",
            "outer",
            "full",
            "cross",
            "on",
            "as",
            "insert",
            "into",
            "values",
            "update",
            "set",
            "delete",
            "create",
            "table",
            "view",
            "index",
            "drop",
            "alter",
            "add",
            "column",
            "primary",
            "key",
            "foreign",
            "references",
            "unique",
            "not",
            "null",
            "default",
            "constraint",
            "check",
            "and",
            "or",
            "in",
            "exists",
            "between",
            "like",
            "is",
            "distinct",
            "order",
            "by",
            "group",
            "having",
            "limit",
            "offset",
            "union",
            "all",
            "except",
            "intersect",
            "case",
            "when",
            "then",
            "else",
            "end",
            "begin",
            "commit",
            "rollback",
            "transaction",
            "with",
            "recursive",
        ],
        Language::GraphQl => &[
            "query",
            "mutation",
            "subscription",
            "fragment",
            "on",
            "type",
            "interface",
            "union",
            "enum",
            "input",
            "schema",
            "directive",
            "extend",
            "implements",
            "scalar",
            "true",
            "false",
            "null",
            "repeatable",
        ],
        Language::Auto => &[],
    }
}

fn builtins_for(lang: Language) -> &'static [&'static str] {
    match lang {
        Language::Rust => &[
            "println",
            "print",
            "eprintln",
            "eprint",
            "panic",
            "assert",
            "assert_eq",
            "assert_ne",
            "debug_assert",
            "debug_assert_eq",
            "debug_assert_ne",
            "todo",
            "unimplemented",
            "unreachable",
            "vec",
            "format",
            "write",
            "writeln",
            "concat",
            "include",
            "include_str",
            "include_bytes",
            "stringify",
            "env",
            "cfg",
            "option_env",
            "file",
            "line",
            "column",
            "module_path",
            "Option",
            "Result",
            "Some",
            "None",
            "Ok",
            "Err",
            "Vec",
            "String",
            "Box",
            "Rc",
            "Arc",
            "Cell",
            "RefCell",
            "HashMap",
            "HashSet",
            "BTreeMap",
            "BTreeSet",
            "i8",
            "i16",
            "i32",
            "i64",
            "i128",
            "isize",
            "u8",
            "u16",
            "u32",
            "u64",
            "u128",
            "usize",
            "f32",
            "f64",
            "bool",
            "char",
            "str",
        ],
        Language::TypeScript | Language::JavaScript => &[
            "console",
            "process",
            "require",
            "module",
            "exports",
            "global",
            "window",
            "document",
            "navigator",
            "location",
            "history",
            "Math",
            "JSON",
            "Object",
            "Array",
            "String",
            "Number",
            "Boolean",
            "RegExp",
            "Date",
            "Error",
            "Map",
            "Set",
            "WeakMap",
            "WeakSet",
            "Promise",
            "Symbol",
            "Proxy",
            "Reflect",
            "Intl",
            "parseInt",
            "parseFloat",
            "isNaN",
            "isFinite",
            "encodeURI",
            "decodeURI",
            "encodeURIComponent",
            "decodeURIComponent",
            "setTimeout",
            "setInterval",
            "clearTimeout",
            "clearInterval",
            "fetch",
            "alert",
            "confirm",
            "prompt",
        ],
        Language::Bash => &[
            "echo", "printf", "read", "cd", "ls", "pwd", "mkdir", "rm", "cp", "mv", "chmod",
            "chown", "grep", "sed", "awk", "cut", "sort", "uniq", "wc", "head", "tail", "cat",
            "less", "more", "find", "xargs", "curl", "wget", "ssh", "scp", "git", "make", "sudo",
            "su", "which", "type", "alias",
        ],
        Language::Sql => &[
            "COUNT",
            "SUM",
            "AVG",
            "MAX",
            "MIN",
            "COALESCE",
            "NULLIF",
            "CAST",
            "CONVERT",
            "NOW",
            "CURRENT_DATE",
            "CURRENT_TIME",
            "CURRENT_TIMESTAMP",
            "DATE",
            "DATEADD",
            "DATEDIFF",
            "SUBSTRING",
            "TRIM",
            "UPPER",
            "LOWER",
            "LENGTH",
            "LEN",
            "CONCAT",
            "ISNULL",
            "IFNULL",
            "NVL",
            "DECODE",
            "ROUND",
            "FLOOR",
            "CEIL",
            "ROW_NUMBER",
            "RANK",
            "DENSE_RANK",
            "NTILE",
            "LAG",
            "LEAD",
            "FIRST_VALUE",
            "LAST_VALUE",
            // lowercase
            "count",
            "sum",
            "avg",
            "max",
            "min",
            "coalesce",
            "nullif",
            "cast",
            "convert",
            "now",
            "current_date",
            "current_time",
            "current_timestamp",
            "date",
            "dateadd",
            "datediff",
            "substring",
            "trim",
            "upper",
            "lower",
            "length",
            "len",
            "concat",
            "isnull",
            "ifnull",
            "nvl",
            "decode",
            "round",
            "floor",
            "ceil",
            "row_number",
            "rank",
            "dense_rank",
            "ntile",
            "lag",
            "lead",
            "first_value",
            "last_value",
        ],
        Language::GraphQl => &["String", "Int", "Float", "Boolean", "ID"],
        Language::Json | Language::Auto => &[],
    }
}

// ---------------------------------------------------------------------------
// Language auto-detection
// ---------------------------------------------------------------------------

pub fn detect_language(code: &str) -> Language {
    let trimmed = code.trim_start();

    // JSON: starts with { or [
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        // quick check: does it have "key": pattern?
        if trimmed.contains("\":") || trimmed == "{}" || trimmed == "[]" {
            return Language::Json;
        }
    }

    // Bash: shebang or common patterns
    if trimmed.starts_with("#!/bin/bash")
        || trimmed.starts_with("#!/bin/sh")
        || trimmed.starts_with("#!/usr/bin/env bash")
    {
        return Language::Bash;
    }

    // SQL: starts with SELECT/INSERT/CREATE/etc
    let lower = trimmed.to_lowercase();
    if lower.starts_with("select ")
        || lower.starts_with("insert ")
        || lower.starts_with("create ")
        || lower.starts_with("update ")
        || lower.starts_with("delete ")
        || lower.starts_with("with ")
    {
        return Language::Sql;
    }

    // GraphQL: type / query / mutation / schema keywords at start of lines
    if lower.starts_with("type ")
        || lower.starts_with("query ")
        || lower.starts_with("mutation ")
        || lower.starts_with("schema ")
        || lower.starts_with("fragment ")
    {
        return Language::GraphQl;
    }

    // Rust: fn / let / impl / pub / mod
    if code.contains("fn ")
        && (code.contains("let ") || code.contains("impl ") || code.contains("-> "))
    {
        return Language::Rust;
    }

    // TypeScript: type annotations, interface, declare, import ... from
    if code.contains(": string")
        || code.contains(": number")
        || code.contains(": boolean")
        || code.contains("interface ")
        || code.contains("declare ")
        || (code.contains("import ") && code.contains(" from "))
    {
        return Language::TypeScript;
    }

    // JavaScript fallback for function / const / let / var
    if code.contains("function ") || code.contains("const ") || code.contains("var ") {
        return Language::JavaScript;
    }

    Language::Rust // default
}

// ---------------------------------------------------------------------------
// Highlighter core
// ---------------------------------------------------------------------------

/// Highlight a single line of `code` given the language's keyword/builtin lists.
/// `in_multiline_string` tracks backtick template literal state (JS/TS).
/// Returns `(highlighted_line, new_in_multiline_string)`.
pub fn highlight_line(
    line: &str,
    lang: Language,
    keywords: &[&str],
    builtins: &[&str],
    in_multiline_string: &mut bool,
) -> String {
    if !ansi_enabled() {
        return line.to_string();
    }

    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut out = String::with_capacity(line.len() * 2);
    let mut i = 0;

    // If we're inside a multiline backtick string from a previous line
    if *in_multiline_string {
        // scan for closing backtick
        let mut j = 0;
        while j < len {
            if chars[j] == '`' {
                *in_multiline_string = false;
                // emit everything up to and including the backtick as string
                let s_text: String = chars[..=j].iter().collect();
                out.push_str(&s().green().paint(&s_text));
                i = j + 1;
                break;
            }
            j += 1;
        }
        if *in_multiline_string {
            // whole line is inside string
            out.push_str(&s().green().paint(line));
            return out;
        }
    }

    while i < len {
        let ch = chars[i];

        // --- Comment detection ---
        match lang {
            Language::Bash | Language::GraphQl => {
                if ch == '#' {
                    let rest: String = chars[i..].iter().collect();
                    out.push_str(&s().dim().paint(&rest));
                    break;
                }
            }
            Language::Sql => {
                if ch == '-' && i + 1 < len && chars[i + 1] == '-' {
                    let rest: String = chars[i..].iter().collect();
                    out.push_str(&s().dim().paint(&rest));
                    break;
                }
                // Also handle /* ... */  (single line only here)
                if ch == '/' && i + 1 < len && chars[i + 1] == '*' {
                    let rest: String = chars[i..].iter().collect();
                    out.push_str(&s().dim().paint(&rest));
                    break;
                }
            }
            _ => {
                // // line comments
                if ch == '/' && i + 1 < len && chars[i + 1] == '/' {
                    let rest: String = chars[i..].iter().collect();
                    out.push_str(&s().dim().paint(&rest));
                    break;
                }
            }
        }

        // --- String detection ---
        if ch == '"' || ch == '\'' || ch == '`' {
            let quote = ch;
            let mut string_buf = String::new();
            string_buf.push(quote);
            i += 1;

            loop {
                if i >= len {
                    // Unterminated string — for backtick, set multiline state
                    if quote == '`' {
                        *in_multiline_string = true;
                    }
                    break;
                }
                let sc = chars[i];
                string_buf.push(sc);
                i += 1;
                if sc == '\\' && i < len {
                    // escape next char
                    string_buf.push(chars[i]);
                    i += 1;
                    continue;
                }
                if sc == quote {
                    break;
                }
            }
            out.push_str(&s().green().paint(&string_buf));
            continue;
        }

        // --- Numbers ---
        if ch.is_ascii_digit() {
            // check preceding context: whitespace, operator, open paren/bracket
            let prev = if i > 0 { chars[i - 1] } else { ' ' };
            let is_start = prev == ' '
                || prev == '\t'
                || prev == '('
                || prev == '['
                || prev == ','
                || prev == ':'
                || prev == '='
                || prev == '+'
                || prev == '-'
                || prev == '*'
                || prev == '/'
                || prev == '<'
                || prev == '>'
                || prev == '!'
                || i == 0;
            if is_start {
                let mut num_buf = String::new();
                // hex
                if ch == '0' && i + 1 < len && (chars[i + 1] == 'x' || chars[i + 1] == 'X') {
                    num_buf.push(chars[i]);
                    num_buf.push(chars[i + 1]);
                    i += 2;
                    while i < len && chars[i].is_ascii_hexdigit() {
                        num_buf.push(chars[i]);
                        i += 1;
                    }
                } else {
                    while i < len
                        && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == '_')
                    {
                        num_buf.push(chars[i]);
                        i += 1;
                    }
                    // optional suffix for Rust (u8, i32, f64, usize, etc)
                    if matches!(lang, Language::Rust)
                        && i < len
                        && (chars[i] == 'u' || chars[i] == 'i' || chars[i] == 'f')
                    {
                        while i < len && chars[i].is_alphanumeric() {
                            num_buf.push(chars[i]);
                            i += 1;
                        }
                    }
                }
                out.push_str(&s().yellow().paint(&num_buf));
                continue;
            }
        }

        // --- Identifiers / keywords / builtins ---
        if ch.is_alphanumeric() || ch == '_' {
            let start = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();

            if keywords.contains(&word.as_str()) {
                out.push_str(&s().magenta().paint(&word));
            } else if builtins.contains(&word.as_str()) {
                out.push_str(&s().cyan().paint(&word));
            } else {
                out.push_str(&word);
            }
            continue;
        }

        // --- Operators / punctuation ---
        if "{}[]()=<>!&|+*/%^~;,.".contains(ch) {
            out.push_str(&s().dim().paint(&ch.to_string()));
            i += 1;
            continue;
        }

        // Default: pass through
        out.push(ch);
        i += 1;
    }

    out
}

// ---------------------------------------------------------------------------
// JSON highlighter (dedicated, because JSON is structural)
// ---------------------------------------------------------------------------

fn highlight_json(line: &str) -> String {
    if !ansi_enabled() {
        return line.to_string();
    }

    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut out = String::with_capacity(line.len() * 2);
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        // String (key or value)
        if ch == '"' {
            let mut string_buf = String::new();
            string_buf.push('"');
            i += 1;
            while i < len {
                let sc = chars[i];
                string_buf.push(sc);
                i += 1;
                if sc == '\\' && i < len {
                    string_buf.push(chars[i]);
                    i += 1;
                    continue;
                }
                if sc == '"' {
                    break;
                }
            }
            // Peek ahead (skip whitespace + colon) to see if this is a key
            let mut j = i;
            while j < len && chars[j] == ' ' {
                j += 1;
            }
            let is_key = j < len && chars[j] == ':';
            if is_key {
                // key: bold white
                out.push_str(&s().bold().paint(&string_buf));
            } else {
                // string value: green
                out.push_str(&s().green().paint(&string_buf));
            }
            continue;
        }

        // Numbers
        if ch.is_ascii_digit() || (ch == '-' && i + 1 < len && chars[i + 1].is_ascii_digit()) {
            let mut num_buf = String::new();
            if ch == '-' {
                num_buf.push('-');
                i += 1;
            }
            while i < len
                && (chars[i].is_ascii_digit()
                    || chars[i] == '.'
                    || chars[i] == 'e'
                    || chars[i] == 'E'
                    || chars[i] == '+'
                    || chars[i] == '-')
            {
                num_buf.push(chars[i]);
                i += 1;
            }
            out.push_str(&s().yellow().paint(&num_buf));
            continue;
        }

        // true / false / null
        if ch.is_alphabetic() {
            let start = i;
            while i < len && chars[i].is_alphabetic() {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            match word.as_str() {
                "true" | "false" => out.push_str(&s().magenta().paint(&word)),
                "null" => out.push_str(&s().dim().paint(&word)),
                _ => out.push_str(&word),
            }
            continue;
        }

        // Structural: colon, comma, braces, brackets
        if ":,{}[]".contains(ch) {
            out.push_str(&s().dim().paint(&ch.to_string()));
            i += 1;
            continue;
        }

        out.push(ch);
        i += 1;
    }

    out
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Syntax-highlight `code` and return the highlighted string.
///
/// When `ansi_enabled()` is false (piped output), returns `code` unchanged.
pub fn highlight(code: &str, options: &HighlightOptions) -> String {
    let lang = match options.language {
        Language::Auto => detect_language(code),
        other => other,
    };

    let keywords = keywords_for(lang);
    let builtins = builtins_for(lang);

    let lines: Vec<&str> = code.split('\n').collect();
    let total_lines = lines.len();
    let num_width = total_lines.to_string().len().max(2);

    let mut out = String::new();
    let mut in_multiline_string = false;

    for (idx, line) in lines.iter().enumerate() {
        let line_num = options.start_line + idx;

        let highlighted = if matches!(lang, Language::Json) {
            highlight_json(line)
        } else {
            highlight_line(line, lang, keywords, builtins, &mut in_multiline_string)
        };

        if options.line_numbers {
            let num_str = format!("{:>width$}", line_num, width = num_width);
            let gutter = s().dim().paint(&format!("{} │ ", num_str));
            out.push_str(&gutter);
        }

        out.push_str(&highlighted);

        if idx + 1 < total_lines {
            out.push('\n');
        }
    }

    out
}
