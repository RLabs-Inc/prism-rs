// prism/prompt — interactive terminal input
// confirm, input, password, select, multiselect — composable input primitives
// composes: keypress, input_line, block, cursor

use crate::block::{live_block, BlockRender, LiveBlockOptions};
use crate::cursor::{hide_cursor, show_cursor};
use crate::error::{PrismError, PrismResult};
use crate::input_line::{InputLine, InputLineOptions, PromptSource};
use crate::keypress::keypress_stream;
use crate::style::s;
use crate::writer::interactive_tty;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

// --- Confirm (y/n) ---

/// Options for confirm prompt
#[derive(Default)]
pub struct ConfirmOptions<'a> {
    pub default: Option<bool>,
    pub cancelled: Option<&'a AtomicBool>,
}

/// Ask a yes/no question. Returns true for yes, false for no.
pub fn confirm(message: &str, options: ConfirmOptions) -> PrismResult<bool> {
    if let Some(flag) = options.cancelled {
        if flag.load(Ordering::SeqCst) {
            return Err(PrismError::Cancelled);
        }
    }

    let default_yes = options.default == Some(true);
    let hint = if default_yes {
        s().dim().paint(" (Y/n)")
    } else {
        s().dim().paint(" (y/N)")
    };

    print!("{} {}{} ", s().cyan().paint("?"), message, hint);
    use std::io::Write;
    std::io::stdout().flush().unwrap();

    if !interactive_tty() {
        println!();
        return Ok(options.default.unwrap_or(false));
    }

    let mut result: Option<bool> = None;
    let mut cancelled = false;
    let cancel_flag = options.cancelled;
    let msg = message.to_string();

    keypress_stream(|key| {
        if let Some(flag) = cancel_flag {
            if flag.load(Ordering::SeqCst) {
                cancelled = true;
                return true;
            }
        }

        if key.key == "enter" {
            let answer = options.default.unwrap_or(false);
            let label = if answer { "yes" } else { "no" };
            print!(
                "\r\x1b[2K{} {} {}\n",
                s().green().paint("✓"),
                msg,
                s().dim().paint(label)
            );
            result = Some(answer);
            return true;
        }
        if key.key == "y" || key.key == "Y" {
            print!(
                "\r\x1b[2K{} {} {}\n",
                s().green().paint("✓"),
                msg,
                s().dim().paint("yes")
            );
            result = Some(true);
            return true;
        }
        if key.key == "n" || key.key == "N" {
            print!(
                "\r\x1b[2K{} {} {}\n",
                s().green().paint("✓"),
                msg,
                s().dim().paint("no")
            );
            result = Some(false);
            return true;
        }
        if key.ctrl && key.key == "c" {
            println!();
            cancelled = true;
            return true;
        }
        false
    })?;

    if cancelled {
        return Err(PrismError::Cancelled);
    }
    Ok(result.unwrap_or(options.default.unwrap_or(false)))
}

// --- Text Input ---

/// Validation callback type
pub type ValidateFn = Box<dyn Fn(&str) -> Result<(), String>>;

/// Options for text input prompt
#[derive(Default)]
pub struct InputOptions<'a> {
    pub default: Option<String>,
    pub placeholder: Option<String>,
    pub validate: Option<ValidateFn>,
    pub cancelled: Option<&'a AtomicBool>,
}

/// Prompt for text input with inline editing.
pub fn input(message: &str, options: InputOptions) -> PrismResult<String> {
    run_text_prompt(message, options, None)
}

// --- Password ---

/// Options for password prompt
#[derive(Default)]
pub struct PasswordOptions<'a> {
    pub cancelled: Option<&'a AtomicBool>,
}

/// Prompt for password input (characters shown as dots).
pub fn password(message: &str, options: PasswordOptions) -> PrismResult<String> {
    run_text_prompt(
        message,
        InputOptions {
            cancelled: options.cancelled,
            ..Default::default()
        },
        Some("●".to_string()),
    )
}

fn run_text_prompt(
    message: &str,
    options: InputOptions,
    mask: Option<String>,
) -> PrismResult<String> {
    if let Some(flag) = options.cancelled {
        if flag.load(Ordering::SeqCst) {
            return Err(PrismError::Cancelled);
        }
    }

    let default_hint = options
        .default
        .as_ref()
        .map(|d| s().dim().paint(&format!(" ({})", d)))
        .unwrap_or_default();

    if !interactive_tty() {
        println!("{} {}{}", s().cyan().paint("?"), message, default_hint);
        return Ok(options.default.unwrap_or_default());
    }

    let msg = message.to_string();
    let msg_for_prompt = msg.clone();
    let default_hint_clone = default_hint.clone();
    let placeholder = options.placeholder.clone();

    // Keep a copy of the mask for use when closing the prompt
    let close_mask = mask.clone();

    // Shared state between render callback and keypress handler
    let state = Rc::new(RefCell::new(InputLine::new(InputLineOptions {
        prompt: PromptSource::Dynamic(Box::new(move || {
            format!("? {}{} ", msg_for_prompt, default_hint_clone)
        })),
        prompt_color: Box::new(|t| t.to_string()),
        mask,
        ..Default::default()
    })));

    let error_text: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

    let state_render = state.clone();
    let placeholder_render = placeholder.clone();
    let error_render = error_text.clone();

    let mut block = live_block(LiveBlockOptions {
        render: Box::new(move || {
            let inp = state_render.borrow();
            let rendered = inp.render();
            let mut lines = rendered.lines;
            if inp.buffer().is_empty() {
                if let Some(ref ph) = placeholder_render {
                    lines[0] = format!("{}{}", lines[0], s().dim().paint(ph));
                }
            }
            let err = error_render.borrow();
            if !err.is_empty() {
                lines.push(format!("{} {}", s().red().paint("✗"), &*err));
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

    let mut result_value: Option<String> = None;
    let mut cancelled = false;
    let cancel_flag = options.cancelled;

    keypress_stream(|key| {
        if let Some(flag) = cancel_flag {
            if flag.load(Ordering::SeqCst) {
                cancelled = true;
                return true;
            }
        }

        // Clear error on any keypress
        error_text.borrow_mut().clear();

        if key.ctrl && key.key == "c" {
            block.close(None);
            println!();
            cancelled = true;
            return true;
        }

        if key.key == "enter" {
            let inp = state.borrow();
            let value = if inp.buffer().is_empty() {
                options.default.clone().unwrap_or_default()
            } else {
                inp.buffer().to_string()
            };
            drop(inp);

            if let Some(ref validate) = options.validate {
                if let Err(err) = validate(&value) {
                    *error_text.borrow_mut() = err;
                    block.update();
                    return false;
                }
            }

            // When mask is set (password mode), show mask characters instead of raw text
            let raw = state.borrow().buffer().to_string();
            let display = match &close_mask {
                Some(m) => m.repeat(raw.chars().count()),
                None => raw,
            };
            block.close(Some(&format!(
                "{} {} {}",
                s().green().paint("✓"),
                msg,
                s().dim().paint(&display)
            )));
            result_value = Some(value);
            return true;
        }

        // Editing keys
        {
            let mut inp = state.borrow_mut();
            match key.key.as_str() {
                "backspace" => inp.backspace(),
                "delete" => inp.delete_char(),
                "left" => inp.cursor_left(),
                "right" => inp.cursor_right(),
                "home" => inp.home(),
                "end" => inp.end(),
                "wordleft" => inp.word_left(),
                "wordright" => inp.word_right(),
                _ if key.ctrl && key.key == "d" => inp.delete_char(),
                _ if key.ctrl && key.key == "a" => inp.home(),
                _ if key.ctrl && key.key == "e" => inp.end(),
                _ if key.ctrl && key.key == "w" => inp.delete_word(),
                _ if key.ctrl && key.key == "u" => inp.clear_before(),
                _ if key.ctrl && key.key == "k" => inp.clear_after(),
                _ if key.meta && key.key == "b" => inp.word_left(),
                _ if key.meta && key.key == "f" => inp.word_right(),
                _ => {
                    if let Some(c) = key.char_val {
                        if !key.ctrl && !key.meta {
                            let ch = c.to_string().replace(['\n', '\r'], " ");
                            inp.insert_char(&ch);
                        }
                    }
                }
            }
        }

        block.update();
        false
    })?;

    if cancelled {
        return Err(PrismError::Cancelled);
    }
    Ok(result_value.unwrap_or_default())
}

// --- Select ---

/// Options for select prompt
pub struct SelectOptions<'a> {
    pub page_size: usize,
    pub cancelled: Option<&'a AtomicBool>,
}

impl Default for SelectOptions<'_> {
    fn default() -> Self {
        Self {
            page_size: 7,
            cancelled: None,
        }
    }
}

/// Choose one item from a list using arrow keys.
pub fn select(message: &str, choices: &[&str], options: SelectOptions) -> PrismResult<String> {
    if let Some(flag) = options.cancelled {
        if flag.load(Ordering::SeqCst) {
            return Err(PrismError::Cancelled);
        }
    }

    if choices.is_empty() {
        println!("{} {}", s().cyan().paint("?"), message);
        return Ok(String::new());
    }

    if !interactive_tty() {
        println!("{} {}", s().cyan().paint("?"), message);
        return Ok(choices[0].to_string());
    }

    let page_size = options.page_size;
    let msg = message.to_string();
    let choices_owned: Vec<String> = choices.iter().map(|s| s.to_string()).collect();

    // Shared state
    let selected = Rc::new(RefCell::new(0usize));
    let scroll_offset = Rc::new(RefCell::new(0usize));

    let sel_render = selected.clone();
    let scroll_render = scroll_offset.clone();
    let choices_render = choices_owned.clone();
    let msg_render = msg.clone();

    hide_cursor();

    let mut block = live_block(LiveBlockOptions {
        render: Box::new(move || {
            let sel = *sel_render.borrow();
            let mut scroll = *scroll_render.borrow();
            let visible_count = page_size.min(choices_render.len());

            let mut lines = vec![format!(
                "{} {} {}",
                s().cyan().paint("?"),
                msg_render,
                s().dim().paint("(↑/↓ to navigate, enter to select)")
            )];

            if sel < scroll {
                scroll = sel;
            }
            if sel >= scroll + visible_count {
                scroll = sel - visible_count + 1;
            }
            *scroll_render.borrow_mut() = scroll;

            #[allow(clippy::needless_range_loop)]
            for i in scroll..(scroll + visible_count).min(choices_render.len()) {
                if i == sel {
                    lines.push(format!(
                        "  {} {}",
                        s().cyan().paint("›"),
                        s().bold().paint(&choices_render[i])
                    ));
                } else {
                    lines.push(format!("    {}", s().dim().paint(&choices_render[i])));
                }
            }

            if choices_render.len() > page_size {
                lines.push(
                    s().dim()
                        .paint(&format!("  ({}/{})", sel + 1, choices_render.len())),
                );
            }

            BlockRender {
                lines,
                cursor: None,
            }
        }),
        on_close: None,
        tty: None,
    });

    block.update();

    let mut result_value: Option<String> = None;
    let mut cancelled = false;
    let cancel_flag = options.cancelled;

    keypress_stream(|key| {
        if let Some(flag) = cancel_flag {
            if flag.load(Ordering::SeqCst) {
                cancelled = true;
                return true;
            }
        }

        match key.key.as_str() {
            "up" | "k" => {
                let mut sel = selected.borrow_mut();
                *sel = (*sel + choices_owned.len() - 1) % choices_owned.len();
            }
            "down" | "j" => {
                let mut sel = selected.borrow_mut();
                *sel = (*sel + 1) % choices_owned.len();
            }
            "enter" => {
                let sel = *selected.borrow();
                block.close(Some(&format!(
                    "{} {} {}",
                    s().green().paint("✓"),
                    msg,
                    s().dim().paint(&choices_owned[sel])
                )));
                show_cursor();
                result_value = Some(choices_owned[sel].clone());
                return true;
            }
            _ if key.ctrl && key.key == "c" => {
                block.close(None);
                show_cursor();
                println!();
                cancelled = true;
                return true;
            }
            _ => {}
        }

        block.update();
        false
    })?;

    if cancelled {
        return Err(PrismError::Cancelled);
    }
    Ok(result_value.unwrap_or_default())
}

// --- Multi-Select ---

/// Options for multiselect prompt
pub struct MultiSelectOptions<'a> {
    pub page_size: usize,
    pub min: usize,
    pub max: Option<usize>,
    pub cancelled: Option<&'a AtomicBool>,
}

impl Default for MultiSelectOptions<'_> {
    fn default() -> Self {
        Self {
            page_size: 7,
            min: 0,
            max: None,
            cancelled: None,
        }
    }
}

/// Choose multiple items from a list using arrow keys + space to toggle.
pub fn multiselect(
    message: &str,
    choices: &[&str],
    options: MultiSelectOptions,
) -> PrismResult<Vec<String>> {
    if let Some(flag) = options.cancelled {
        if flag.load(Ordering::SeqCst) {
            return Err(PrismError::Cancelled);
        }
    }

    if choices.is_empty() {
        println!("{} {}", s().cyan().paint("?"), message);
        return Ok(vec![]);
    }

    if !interactive_tty() {
        println!("{} {}", s().cyan().paint("?"), message);
        return Ok(vec![]);
    }

    let page_size = options.page_size;
    let max_sel = options.max.unwrap_or(choices.len());
    let min_sel = options.min;
    let msg = message.to_string();
    let choices_owned: Vec<String> = choices.iter().map(|s| s.to_string()).collect();

    // Shared state
    let cursor = Rc::new(RefCell::new(0usize));
    let scroll_offset = Rc::new(RefCell::new(0usize));
    let selected_set = Rc::new(RefCell::new(vec![false; choices.len()]));

    let cur_render = cursor.clone();
    let scroll_render = scroll_offset.clone();
    let sel_render = selected_set.clone();
    let choices_render = choices_owned.clone();
    let msg_render = msg.clone();

    hide_cursor();

    let mut block = live_block(LiveBlockOptions {
        render: Box::new(move || {
            let cur = *cur_render.borrow();
            let mut scroll = *scroll_render.borrow();
            let sel = sel_render.borrow();
            let visible_count = page_size.min(choices_render.len());

            let mut lines = vec![format!(
                "{} {} {}",
                s().cyan().paint("?"),
                msg_render,
                s().dim().paint("(space to toggle, enter to confirm)")
            )];

            if cur < scroll {
                scroll = cur;
            }
            if cur >= scroll + visible_count {
                scroll = cur - visible_count + 1;
            }
            let _ = scroll_render.replace(scroll);

            for i in scroll..(scroll + visible_count).min(choices_render.len()) {
                let is_sel = sel[i];
                let is_cur = i == cur;
                let checkbox = if is_sel {
                    s().green().paint("◉")
                } else {
                    s().dim().paint("○")
                };
                let label = if is_cur {
                    s().bold().paint(&choices_render[i])
                } else {
                    s().dim().paint(&choices_render[i])
                };
                let pointer = if is_cur {
                    s().cyan().paint("›")
                } else {
                    " ".to_string()
                };
                lines.push(format!("  {} {} {}", pointer, checkbox, label));
            }

            if choices_render.len() > page_size {
                let count = sel.iter().filter(|&&x| x).count();
                lines.push(s().dim().paint(&format!(
                    "  ({}/{}, {} selected)",
                    cur + 1,
                    choices_render.len(),
                    count
                )));
            }

            BlockRender {
                lines,
                cursor: None,
            }
        }),
        on_close: None,
        tty: None,
    });

    block.update();

    let mut result_value: Option<Vec<String>> = None;
    let mut cancelled = false;
    let cancel_flag = options.cancelled;

    keypress_stream(|key| {
        if let Some(flag) = cancel_flag {
            if flag.load(Ordering::SeqCst) {
                cancelled = true;
                return true;
            }
        }

        match key.key.as_str() {
            "up" | "k" => {
                let mut cur = cursor.borrow_mut();
                *cur = (*cur + choices_owned.len() - 1) % choices_owned.len();
            }
            "down" | "j" => {
                let mut cur = cursor.borrow_mut();
                *cur = (*cur + 1) % choices_owned.len();
            }
            "space" => {
                let cur = *cursor.borrow();
                let mut sel = selected_set.borrow_mut();
                if sel[cur] {
                    sel[cur] = false;
                } else {
                    let count = sel.iter().filter(|&&x| x).count();
                    if count < max_sel {
                        sel[cur] = true;
                    }
                }
            }
            "a" => {
                let mut sel = selected_set.borrow_mut();
                let all_selected = sel.iter().all(|&x| x);
                if all_selected {
                    sel.fill(false);
                } else {
                    for (i, v) in sel.iter_mut().enumerate() {
                        *v = i < max_sel;
                    }
                }
            }
            "enter" => {
                let sel = selected_set.borrow();
                let count = sel.iter().filter(|&&x| x).count();
                if count < min_sel {
                    drop(sel);
                    block.update();
                    return false;
                }
                let chosen: Vec<String> = choices_owned
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| sel[*i])
                    .map(|(_, c)| c.clone())
                    .collect();
                drop(sel);
                block.close(Some(&format!(
                    "{} {} {}",
                    s().green().paint("✓"),
                    msg,
                    s().dim().paint(&chosen.join(", "))
                )));
                show_cursor();
                result_value = Some(chosen);
                return true;
            }
            _ if key.ctrl && key.key == "c" => {
                block.close(None);
                show_cursor();
                println!();
                cancelled = true;
                return true;
            }
            _ => {}
        }

        block.update();
        false
    })?;

    if cancelled {
        return Err(PrismError::Cancelled);
    }
    Ok(result_value.unwrap_or_default())
}
