use crate::frame::{BorderStyle, FrameOptions};
use crate::highlight::{highlight, HighlightOptions, Language};

// ---------------------------------------------------------------------------
// FilePreviewOptions
// ---------------------------------------------------------------------------

pub struct FilePreviewOptions {
    pub filename: Option<String>,
    pub language: Language,
    pub line_numbers: bool,
    pub start_line: usize,
    pub border: BorderStyle,
    pub width: Option<usize>,
}

impl Default for FilePreviewOptions {
    fn default() -> Self {
        Self {
            filename: None,
            language: Language::Auto,
            line_numbers: true,
            start_line: 1,
            border: BorderStyle::Single,
            width: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Render `content` as a syntax-highlighted file preview wrapped in a border.
///
/// The title bar shows `options.filename` if provided.
/// Content is highlighted according to `options.language` (Auto-detects by default).
/// Line numbers are shown when `options.line_numbers` is true (default).
pub fn file_preview(content: &str, options: &FilePreviewOptions) -> String {
    let highlighted = highlight(
        content,
        &HighlightOptions {
            language: options.language,
            line_numbers: options.line_numbers,
            start_line: options.start_line,
        },
    );

    crate::frame::frame(
        &highlighted,
        &FrameOptions {
            title: options.filename.clone(),
            border: options.border,
            width: options.width,
            ..Default::default()
        },
    )
}
