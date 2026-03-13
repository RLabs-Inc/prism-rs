pub fn strip_ansi(text: &str) -> String {
    // stub — will be implemented in Task 0.4
    text.to_string()
}

pub fn measure_width(text: &str) -> usize {
    // stub — will be implemented in Task 0.4
    unicode_width::UnicodeWidthStr::width(strip_ansi(text).as_str())
}

pub fn wrap_ansi(_text: &str, _width: usize) -> String {
    // stub
    _text.to_string()
}
