// prism/repl — interactive prompt loop
// composes: input_line + block + keypress_stream + command_router
//
// two exports:
//   readline() — single prompt with full line editing, history, completion
//   repl()     — persistent prompt loop with slash commands and abort support

use crate::block::{live_block, BlockRender, LiveBlockOptions};
use crate::command_router::{Command, CommandRouter};
use crate::error::PrismResult;
use crate::input_line::{InputLine, InputLineOptions, PromptSource};
use crate::keypress::keypress_stream;
use crate::style::s;
use crate::writer::{interactive_tty, write, writeln};
use std::cell::RefCell;
use std::io::BufRead;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ── Type aliases ──────────────────────────────────────────

/// Completion callback: (word_at_cursor, full_line) -> candidates
type CompletionFn = Box<dyn Fn(&str, &str) -> Vec<String>>;

/// Handler callback: (args, cancelled_flag) -> optional output text
pub type HandlerFn = Box<dyn FnMut(&str, &AtomicBool) -> Option<String>>;

// ── Types ─────────────────────────────────────────────────

/// Either a static prompt string or a function that returns one
pub enum PromptFn {
    Static(String),
    Dynamic(Box<dyn Fn() -> String + Send + Sync>),
}

impl PromptFn {
    fn resolve(&self) -> String {
        match self {
            PromptFn::Static(s) => s.clone(),
            PromptFn::Dynamic(f) => f(),
        }
    }
}

impl Default for PromptFn {
    fn default() -> Self {
        PromptFn::Static("> ".into())
    }
}

/// Options for [`readline`]
pub struct ReadlineOptions {
    /// Prompt string or function for dynamic prompts
    pub prompt: PromptFn,
    /// Default value pre-filled in the input
    pub default: Option<String>,
    /// Shared history array (entries prepended on submit)
    pub history: Vec<String>,
    /// Max history entries (default: 500)
    pub history_size: usize,
    /// Tab completion: return candidates for the partial word
    pub completion: Option<CompletionFn>,
    /// Prompt color function (default: cyan)
    pub prompt_color: Box<dyn Fn(&str) -> String + Send + Sync>,
    /// Mask character for sensitive input (e.g., "●")
    pub mask: Option<String>,
}

impl Default for ReadlineOptions {
    fn default() -> Self {
        Self {
            prompt: PromptFn::default(),
            default: None,
            history: Vec::new(),
            history_size: 500,
            completion: None,
            prompt_color: Box::new(|t| s().cyan().paint(t)),
            mask: None,
        }
    }
}

/// Handler for repl commands
pub struct ReplCommand {
    pub description: Option<String>,
    pub aliases: Vec<String>,
    /// Handler receives args string and cancelled flag.
    /// Return Some(text) to auto-print output.
    pub handler: HandlerFn,
}

/// Options for [`repl`]
pub struct ReplOptions {
    /// Prompt string or function for dynamic prompts
    pub prompt: PromptFn,
    /// Called when user submits non-command input. Return Some(text) to auto-print.
    pub on_input: HandlerFn,
    /// Greeting shown when repl starts
    pub greeting: Option<String>,
    /// Slash commands
    pub commands: Vec<(String, ReplCommand)>,
    /// Prefix for commands (default: "/")
    pub command_prefix: String,
    /// Strings that exit the repl (default: ["exit", "quit"])
    pub exit_commands: Vec<String>,
    /// Enable input history (default: true)
    pub history: bool,
    /// Max history entries (default: 500)
    pub history_size: usize,
    /// Tab completion for non-command input
    pub completion: Option<CompletionFn>,
    /// Called before each prompt
    pub before_prompt: Option<Box<dyn FnMut()>>,
    /// Called on exit
    pub on_exit: Option<Box<dyn FnOnce()>>,
    /// Prompt color function (default: cyan)
    pub prompt_color: Box<dyn Fn(&str) -> String + Send + Sync>,
}

impl Default for ReplOptions {
    fn default() -> Self {
        Self {
            prompt: PromptFn::default(),
            on_input: Box::new(|_, _| None),
            greeting: None,
            commands: Vec::new(),
            command_prefix: "/".into(),
            exit_commands: vec!["exit".into(), "quit".into()],
            history: true,
            history_size: 500,
            completion: None,
            before_prompt: None,
            on_exit: None,
            prompt_color: Box::new(|t| s().cyan().paint(t)),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────

/// Find the word being typed at cursor position
pub fn word_at_cursor(buffer: &str, cursor: usize) -> (&str, usize) {
    let bytes = buffer.as_bytes();
    let mut start = cursor;
    while start > 0 && bytes[start - 1] != b' ' {
        start -= 1;
    }
    (&buffer[start..cursor], start)
}

/// Longest common prefix of strings
pub fn common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    let first = &strings[0];
    let mut len = first.len();
    for s in &strings[1..] {
        len = len.min(s.len());
        for (i, (a, b)) in first.bytes().zip(s.bytes()).enumerate() {
            if a != b {
                len = len.min(i);
                break;
            }
        }
    }
    first[..len].to_string()
}

// ── Input actions ─────────────────────────────────────────

enum InputAction {
    Submit(String),
    Cancel,
    Eof,
}

// ── Core input reader ─────────────────────────────────────

struct InputConfig {
    prompt: PromptFn,
    prompt_color: Box<dyn Fn(&str) -> String + Send + Sync>,
    initial: String,
    history: Vec<String>,
    history_size: usize,
    completion: Option<CompletionFn>,
    mask: Option<String>,
    clear_on_cancel: bool,
}

fn read_input(config: InputConfig) -> PrismResult<(InputAction, Vec<String>)> {
    if !interactive_tty() {
        return Ok((InputAction::Cancel, config.history));
    }

    // Task 1: Pass the PromptFn through as PromptSource without resolving Dynamic to Static.
    // The Dynamic variant is preserved so the prompt is re-evaluated on every render.
    let prompt_source = match config.prompt {
        PromptFn::Static(s) => PromptSource::Static(s),
        PromptFn::Dynamic(f) => PromptSource::Dynamic(f),
    };

    let inp = Rc::new(RefCell::new(InputLine::new(InputLineOptions {
        prompt: prompt_source,
        prompt_color: config.prompt_color,
        history: config.history,
        history_size: Some(config.history_size),
        mask: config.mask,
    })));

    if !config.initial.is_empty() {
        inp.borrow_mut().insert_char(&config.initial);
    }

    let hint: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

    let inp_render = inp.clone();
    let hint_render = hint.clone();

    let mut block = live_block(LiveBlockOptions {
        render: Box::new(move || {
            let i = inp_render.borrow();
            let rendered = i.render();
            let mut lines = rendered.lines;
            let h = hint_render.borrow();
            if !h.is_empty() {
                lines[0] = format!("{}  {}", lines[0], s().dim().paint(&h));
            }
            BlockRender {
                lines,
                cursor: Some((rendered.cursor.0 as u16, rendered.cursor.1 as u16)),
            }
        }),
        on_close: None,
        tty: None,
    });

    block.update();

    let mut result: Option<InputAction> = None;
    let completion = config.completion;
    let clear_on_cancel = config.clear_on_cancel;

    keypress_stream(|key| {
        *hint.borrow_mut() = String::new();

        // Enter: submit
        if key.key == "enter" {
            let value = inp.borrow().buffer().to_string();
            let final_line = inp.borrow().render().lines[0].clone();
            inp.borrow_mut().submit();
            block.close(Some(&final_line));
            result = Some(InputAction::Submit(value));
            return true;
        }

        // Ctrl+C
        if key.ctrl && key.key == "c" {
            if clear_on_cancel && !inp.borrow().buffer().is_empty() {
                block.print(&s().dim().paint("^C"));
                inp.borrow_mut().clear_line();
                block.update();
                return false;
            }
            block.close(None);
            result = Some(InputAction::Cancel);
            return true;
        }

        // Ctrl+D
        if key.ctrl && key.key == "d" {
            if inp.borrow().buffer().is_empty() {
                write("\n");
                block.close(None);
                result = Some(InputAction::Eof);
                return true;
            }
            inp.borrow_mut().delete_char();
            block.update();
            return false;
        }

        // Tab: completion
        if key.key == "tab" {
            if let Some(ref comp) = completion {
                let i = inp.borrow();
                let (word, start) = word_at_cursor(i.buffer(), i.cursor());
                let word_owned = word.to_string();
                let line_owned = i.buffer().to_string();
                drop(i);

                let candidates = comp(&word_owned, &line_owned);

                if candidates.is_empty() {
                    return false;
                }

                if candidates.len() == 1 {
                    let i = inp.borrow();
                    let new_buf = format!(
                        "{}{}{}",
                        &i.buffer()[..start],
                        candidates[0],
                        &i.buffer()[i.cursor()..]
                    );
                    let new_pos = start + candidates[0].len();
                    drop(i);
                    inp.borrow_mut().set_value(&new_buf, Some(new_pos));
                    block.update();
                    return false;
                }

                // Multiple matches: insert common prefix, show candidates as hint
                let prefix = common_prefix(&candidates);
                if prefix.len() > word_owned.len() {
                    let i = inp.borrow();
                    let new_buf = format!(
                        "{}{}{}",
                        &i.buffer()[..start],
                        prefix,
                        &i.buffer()[i.cursor()..]
                    );
                    let new_pos = start + prefix.len();
                    drop(i);
                    inp.borrow_mut().set_value(&new_buf, Some(new_pos));
                }

                let max_show = 8;
                let mut display: String = candidates[..max_show.min(candidates.len())].join(", ");
                if candidates.len() > max_show {
                    display.push_str(&format!(", +{} more", candidates.len() - max_show));
                }
                *hint.borrow_mut() = display;
                block.update();
                return false;
            }
            return false;
        }

        // Arrow keys
        match key.key.as_str() {
            "up" => {
                inp.borrow_mut().history_up();
                block.update();
                return false;
            }
            "down" => {
                inp.borrow_mut().history_down();
                block.update();
                return false;
            }
            "right" => {
                inp.borrow_mut().cursor_right();
                block.update();
                return false;
            }
            "left" => {
                inp.borrow_mut().cursor_left();
                block.update();
                return false;
            }
            "home" => {
                inp.borrow_mut().home();
                block.update();
                return false;
            }
            "end" => {
                inp.borrow_mut().end();
                block.update();
                return false;
            }
            "wordleft" => {
                inp.borrow_mut().word_left();
                block.update();
                return false;
            }
            "wordright" => {
                inp.borrow_mut().word_right();
                block.update();
                return false;
            }
            "backspace" => {
                inp.borrow_mut().backspace();
                block.update();
                return false;
            }
            "delete" => {
                inp.borrow_mut().delete_char();
                block.update();
                return false;
            }
            _ => {}
        }

        // Ctrl shortcuts
        if key.ctrl {
            match key.key.as_str() {
                "a" => inp.borrow_mut().home(),
                "e" => inp.borrow_mut().end(),
                "w" => inp.borrow_mut().delete_word(),
                "u" => inp.borrow_mut().clear_before(),
                "k" => inp.borrow_mut().clear_after(),
                "l" => {
                    write("\x1b[2J\x1b[H");
                    block.update();
                    return false;
                }
                _ => return false,
            }
            block.update();
            return false;
        }

        // Meta (Alt) shortcuts
        if key.meta {
            match key.key.as_str() {
                "b" => inp.borrow_mut().word_left(),
                "f" => inp.borrow_mut().word_right(),
                _ => return false,
            }
            block.update();
            return false;
        }

        // Regular characters / paste
        if let Some(c) = key.char_val {
            let clean = c.to_string().replace(['\n', '\r'], " ");
            inp.borrow_mut().insert_char(&clean);
            block.update();
            return false;
        }

        false
    })?;

    // Extract history from the InputLine before dropping
    let final_history = inp.borrow().editor().history().to_vec();

    Ok((result.unwrap_or(InputAction::Cancel), final_history))
}

// ── Public: readline ──────────────────────────────────────

/// Read a single line of input with full line editing.
///
/// Features: cursor movement, word jumping (Ctrl+Left/Right),
/// history (Up/Down), tab completion, paste handling.
///
/// Returns the submitted string, or "" on Ctrl+C / Ctrl+D.
pub fn readline(options: ReadlineOptions) -> PrismResult<String> {
    // Task 2: Non-TTY mode reads a line from stdin instead of returning default
    if !interactive_tty() {
        let prompt = options.prompt.resolve();
        if !prompt.is_empty() {
            write(&prompt);
        }
        let stdin = std::io::stdin();
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) | Err(_) => {
                // EOF or error — return default or empty
                return Ok(options.default.unwrap_or_default());
            }
            Ok(_) => {
                let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
                if trimmed.is_empty() {
                    return Ok(options.default.unwrap_or_default());
                }
                return Ok(trimmed.to_string());
            }
        }
    }

    let (action, _history) = read_input(InputConfig {
        prompt: options.prompt,
        prompt_color: options.prompt_color,
        initial: options.default.unwrap_or_default(),
        history: options.history,
        history_size: options.history_size,
        completion: options.completion,
        mask: options.mask,
        clear_on_cancel: false,
    })?;

    match action {
        InputAction::Submit(value) => Ok(value),
        _ => Ok(String::new()),
    }
}

// ── Public: repl ──────────────────────────────────────────

/// Run an interactive prompt loop.
///
/// Features: slash commands with auto-help, cancellation flag for
/// handlers, history, tab completion (auto-completes command names),
/// Ctrl+C to cancel/exit, Ctrl+D to exit.
///
/// ## Cancellation / SIGINT
///
/// The `cancelled` [`AtomicBool`] is passed to every handler invocation.
/// In TTY mode, Ctrl+C during `read_input` is handled by the keypress
/// loop (clearing the line or exiting). However, while a handler is
/// executing synchronously, we are NOT reading keys, so Ctrl+C during
/// handler execution is not currently intercepted. Long-running handlers
/// should periodically check `cancelled.load(Ordering::SeqCst)` and
/// cooperate with external cancellation (e.g., a separate signal thread
/// that sets the flag). A future enhancement could install a SIGINT
/// handler via `libc` or `ctrlc` crate that sets the flag during handler
/// execution.
pub fn repl(mut options: ReplOptions) -> PrismResult<()> {
    let prefix = options.command_prefix.clone();
    let exit_commands = options.exit_commands.clone();

    // Build command router with auto-help
    let mut router_commands: Vec<(String, Command)> = options
        .commands
        .iter()
        .map(|(name, cmd)| {
            (
                name.clone(),
                Command {
                    description: cmd.description.clone(),
                    aliases: cmd.aliases.clone(),
                    hidden: false,
                },
            )
        })
        .collect();

    let has_help = options.commands.iter().any(|(n, _)| n == "help");
    if !options.commands.is_empty() && !has_help {
        router_commands.push((
            "help".into(),
            Command {
                description: Some("Show available commands".into()),
                aliases: vec!["h".into(), "?".into()],
                hidden: false,
            },
        ));
    }

    let router = if !router_commands.is_empty() {
        Some(CommandRouter::new(router_commands, &prefix))
    } else {
        None
    };

    // Shared history
    let mut history: Vec<String> = Vec::new();

    // Wrap router and completion in Rc for sharing with closures
    let router = Rc::new(router);
    let user_completion: Rc<Option<CompletionFn>> = Rc::new(options.completion.take());
    let prefix = Rc::new(prefix);

    // Greeting
    if let Some(ref greeting) = options.greeting {
        writeln(&format!("{}\n", greeting));
    }

    // Task 3: Non-TTY mode reads all lines from stdin, processes commands and input
    if !interactive_tty() {
        let stdin = std::io::stdin();
        let reader = stdin.lock();

        // Task 6: cancelled flag for non-TTY handlers
        let cancelled = AtomicBool::new(false);

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(_) => break,
            };
            let trimmed = line.trim().to_string();
            if trimmed.is_empty() {
                continue;
            }

            // Exit commands
            if exit_commands
                .iter()
                .any(|e| e.eq_ignore_ascii_case(&trimmed))
            {
                break;
            }

            // Slash commands
            if let Some(ref r) = *router {
                if let Some(cmd_match) = r.match_input(&trimmed) {
                    // Auto-help
                    if cmd_match.name == "help" && !has_help {
                        let text = r.help_text();
                        if text.is_empty() {
                            writeln(&s().dim().paint("  No commands available."));
                        } else {
                            writeln(&format!("\n{}\n", text));
                        }
                        continue;
                    }

                    // Find and call the user's handler
                    cancelled.store(false, Ordering::SeqCst);
                    let matched_name = cmd_match.name.clone();
                    let matched_args = cmd_match.args.clone();

                    if let Some((_, cmd)) = options
                        .commands
                        .iter_mut()
                        .find(|(n, _)| *n == matched_name)
                    {
                        // Task 4: Catch panics from command handlers
                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            (cmd.handler)(&matched_args, &cancelled)
                        })) {
                            Ok(Some(text)) => {
                                if !text.is_empty() {
                                    write(&text);
                                    if !text.ends_with('\n') {
                                        write("\n");
                                    }
                                }
                            }
                            Ok(None) => {}
                            Err(panic_val) => {
                                let msg = panic_message(&panic_val);
                                writeln(&format!(
                                    "{} {}",
                                    s().red().paint("\u{2717}"),
                                    msg
                                ));
                            }
                        }
                    }
                    continue;
                }
            }

            // Regular input: call handler
            cancelled.store(false, Ordering::SeqCst);
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                (options.on_input)(&trimmed, &cancelled)
            })) {
                Ok(Some(text)) => {
                    if !text.is_empty() {
                        write(&text);
                        if !text.ends_with('\n') {
                            write("\n");
                        }
                    }
                }
                Ok(None) => {}
                Err(panic_val) => {
                    let msg = panic_message(&panic_val);
                    writeln(&format!(
                        "{} {}",
                        s().red().paint("\u{2717}"),
                        msg
                    ));
                }
            }
        }

        if let Some(on_exit) = options.on_exit.take() {
            on_exit();
        }
        return Ok(());
    }

    // Task 1: Wrap the dynamic prompt in Arc so we can create PromptSource::Dynamic
    // on each loop iteration without consuming the original PromptFn.
    // For Static, we just clone the string each time.
    // For Dynamic, we wrap the closure in Arc and create a new closure each iteration
    // that delegates to the shared Arc, so the prompt is re-evaluated on every render.
    let shared_prompt: Arc<PromptFn> = Arc::new(options.prompt);

    // Task 5: Capture the user's prompt_color instead of hardcoding cyan.
    // Wrapped in Arc so we can clone it into each loop iteration's closure.
    let shared_prompt_color: Arc<dyn Fn(&str) -> String + Send + Sync> =
        Arc::from(options.prompt_color);

    // Task 6: Shared cancelled flag. Exposed to handlers so they can check it.
    // In the future, a SIGINT handler can set this during handler execution.
    let cancelled = Arc::new(AtomicBool::new(false));

    let mut cancel_count = 0u32;

    loop {
        if let Some(ref mut before) = options.before_prompt {
            before();
        }

        // Build completion that merges command completions + user completions
        let uc = user_completion.clone();
        let rr = router.clone();
        let pr = prefix.clone();

        let completion: Option<CompletionFn> = Some(Box::new(move |word: &str, line: &str| {
            if line.starts_with(pr.as_str()) {
                if let Some(ref r) = *rr {
                    return r.completions(line);
                }
            }
            if let Some(ref comp) = *uc {
                return comp(word, line);
            }
            vec![]
        }));

        // Task 1: Keep Dynamic variant alive — create a new PromptFn::Dynamic
        // that delegates to the shared Arc each time, so the prompt is re-evaluated
        // on every render call.
        let prompt_fn = match &*shared_prompt {
            PromptFn::Static(s) => PromptFn::Static(s.clone()),
            PromptFn::Dynamic(_) => {
                let sp = shared_prompt.clone();
                PromptFn::Dynamic(Box::new(move || sp.resolve()))
            }
        };

        // Task 5: Use the user's prompt_color instead of hardcoding cyan
        let pc = shared_prompt_color.clone();
        let prompt_color: Box<dyn Fn(&str) -> String + Send + Sync> =
            Box::new(move |t: &str| pc(t));

        let (action, returned_history) = read_input(InputConfig {
            prompt: prompt_fn,
            prompt_color,
            initial: String::new(),
            history: history.clone(),
            history_size: options.history_size,
            completion,
            mask: None,
            clear_on_cancel: true,
        })?;

        history = returned_history;

        match action {
            // Ctrl+D: immediate exit
            InputAction::Eof => break,

            // Ctrl+C on empty: exit sequence
            InputAction::Cancel => {
                cancel_count += 1;
                if cancel_count >= 2 {
                    break;
                }
                writeln(&s().dim().paint("(press Ctrl+C again or Ctrl+D to exit)"));
                continue;
            }

            InputAction::Submit(input) => {
                cancel_count = 0;
                let trimmed = input.trim().to_string();

                if trimmed.is_empty() {
                    continue;
                }

                // Exit commands
                if exit_commands
                    .iter()
                    .any(|e| e.eq_ignore_ascii_case(&trimmed))
                {
                    break;
                }

                // Slash commands
                if let Some(ref r) = *router {
                    if let Some(cmd_match) = r.match_input(&trimmed) {
                        // Check if it's the auto-help command
                        if cmd_match.name == "help" && !has_help {
                            let text = r.help_text();
                            if text.is_empty() {
                                writeln(&s().dim().paint("  No commands available."));
                            } else {
                                writeln(&format!("\n{}\n", text));
                            }
                            continue;
                        }

                        // Find and call the user's handler
                        // Task 6: Reset cancelled flag before each handler invocation
                        cancelled.store(false, Ordering::SeqCst);
                        let matched_name = cmd_match.name.clone();
                        let matched_args = cmd_match.args.clone();

                        if let Some((_, cmd)) = options
                            .commands
                            .iter_mut()
                            .find(|(n, _)| *n == matched_name)
                        {
                            // Task 4: Catch panics from command handlers and display errors
                            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                (cmd.handler)(&matched_args, &cancelled)
                            })) {
                                Ok(Some(text)) => {
                                    if !text.is_empty() {
                                        write(&text);
                                        if !text.ends_with('\n') {
                                            write("\n");
                                        }
                                    }
                                }
                                Ok(None) => {}
                                Err(panic_val) => {
                                    let msg = panic_message(&panic_val);
                                    writeln(&format!(
                                        "{} {}",
                                        s().red().paint("\u{2717}"),
                                        msg
                                    ));
                                }
                            }
                            continue;
                        }
                    }

                    // Unknown command
                    if trimmed.starts_with(prefix.as_str()) {
                        let cmd_name = trimmed[prefix.len()..].split(' ').next().unwrap_or("");
                        writeln(&format!(
                            "{} Unknown command: {}",
                            s().red().paint("\u{2717}"),
                            s().bold().paint(&format!("{}{}", *prefix, cmd_name))
                        ));
                        if !options.commands.is_empty() {
                            writeln(
                                &s().dim().paint(&format!(
                                    "  Type {}help for available commands.",
                                    *prefix
                                )),
                            );
                        }
                        continue;
                    }
                }

                // Regular input: call handler
                // Task 6: Reset cancelled flag before handler invocation
                cancelled.store(false, Ordering::SeqCst);

                // Task 4: Catch panics from input handler and display errors
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    (options.on_input)(&trimmed, &cancelled)
                })) {
                    Ok(Some(text)) => {
                        if !text.is_empty() {
                            write(&text);
                            if !text.ends_with('\n') {
                                write("\n");
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(panic_val) => {
                        let msg = panic_message(&panic_val);
                        writeln(&format!(
                            "{} {}",
                            s().red().paint("\u{2717}"),
                            msg
                        ));
                    }
                }
            }
        }
    }

    if let Some(on_exit) = options.on_exit.take() {
        on_exit();
    }

    Ok(())
}

// ── Panic message extraction ──────────────────────────────

/// Extract a human-readable message from a panic payload
fn panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "handler panicked".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_at_cursor_basic() {
        let (word, start) = word_at_cursor("hello world", 5);
        assert_eq!(word, "hello");
        assert_eq!(start, 0);
    }

    #[test]
    fn word_at_cursor_second_word() {
        let (word, start) = word_at_cursor("hello world", 11);
        assert_eq!(word, "world");
        assert_eq!(start, 6);
    }

    #[test]
    fn word_at_cursor_middle() {
        let (word, start) = word_at_cursor("hello world", 8);
        assert_eq!(word, "wo");
        assert_eq!(start, 6);
    }

    #[test]
    fn word_at_cursor_empty() {
        let (word, start) = word_at_cursor("", 0);
        assert_eq!(word, "");
        assert_eq!(start, 0);
    }

    #[test]
    fn common_prefix_basic() {
        let result = common_prefix(&["hello".into(), "help".into(), "helm".into()]);
        assert_eq!(result, "hel");
    }

    #[test]
    fn common_prefix_none() {
        let result = common_prefix(&["abc".into(), "xyz".into()]);
        assert_eq!(result, "");
    }

    #[test]
    fn common_prefix_full_match() {
        let result = common_prefix(&["same".into(), "same".into()]);
        assert_eq!(result, "same");
    }

    #[test]
    fn common_prefix_empty() {
        let result = common_prefix(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn common_prefix_single() {
        let result = common_prefix(&["only".into()]);
        assert_eq!(result, "only");
    }
}
