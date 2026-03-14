// prism — CLI primitives library for Rust
// Full port of @rlabs-inc/prism TypeScript library

// Foundation
pub mod ansi;
pub mod cursor;
pub mod error;
pub mod style;
pub mod text;
pub mod unicode;
pub mod writer;

// Composition Layer (Two-Zone Architecture)
pub mod activity_line;
pub mod block;
pub mod layout;
pub mod live;
pub mod section_block;
pub mod statusbar;
pub mod stream;

// Display Components
pub mod banner;
pub mod columns;
pub mod diff;
pub mod file_preview;
pub mod frame;
pub mod highlight;
pub mod list;
pub mod markdown;
pub mod table;

// Animation & Feedback
pub mod badge;
pub mod elapsed;
pub mod log;
pub mod progress;
pub mod progress_bar;
pub mod spinner;
pub mod timer;

// Interactive Input
pub mod args;
pub mod command_router;
pub mod exec;
pub mod input_line;
pub mod keypress;
pub mod line_editor;
pub mod prompt;
pub mod repl;

// ── Foundation re-exports ────────────────────────────────────────────

pub use ansi::{measure_width, strip_ansi, wrap_ansi};
pub use cursor::{ensure_cursor_visible, hide_cursor, show_cursor};
pub use error::{PrismError, PrismResult};
pub use style::{color, hex, rgb, s, style, Color, Style, RESET};
pub use text::{indent, link, pad, truncate, wrap};
pub use unicode::{
    grapheme_segments, next_grapheme_boundary, normalize_grapheme_boundary,
    previous_grapheme_boundary, GraphemeSegment,
};
pub use writer::{
    ansi_enabled, interactive_tty, is_tty, pipe_aware, term_width, visual_rows, write, write_err,
    writeln,
};

// ── Composition Layer re-exports ─────────────────────────────────────

pub use block::{live_block, BlockRender, LiveBlock, LiveBlockOptions};
pub use layout::{
    layout, ActiveFrame, Layout, LayoutActivityOptions, LayoutOptions, LayoutSectionOptions,
    LayoutStreamOptions,
};
pub use live::{activity, section, Activity, ActivityOptions, Section, SectionOptions};
pub use statusbar::{statusbar, statusbar_render, Segment, StatusBarConfig};
pub use stream::{stream, stream_with, Stream, StreamOptions};

// ── Display Component re-exports ─────────────────────────────────────

pub use banner::{banner, BannerColor, BannerOptions, BannerStyle};
pub use columns::{columns, ColumnsOptions};
pub use diff::{diff, DiffOptions};
pub use file_preview::{file_preview, FilePreviewOptions};
pub use frame::{divider, frame, header, BorderChars, BorderStyle, FrameOptions};
pub use highlight::{detect_language, highlight, highlight_line, HighlightOptions, Language};
pub use list::{kv, list, tree, KvOptions, ListOptions, ListStyle, TreeNode};
pub use markdown::md;
pub use table::{table, Align, Column, TableOptions};

// ── Animation & Feedback re-exports ──────────────────────────────────

pub use badge::{badge, BadgeVariant};
pub use elapsed::Elapsed;
pub use progress::{progress, ProgressBar, ProgressOptions};
pub use progress_bar::{render_progress_bar, BarStyle, RenderOptions};
pub use spinner::{all_spinner_names, get_spinner, SpinnerDef};
pub use timer::{
    bench, countdown, format_time, stopwatch, BenchResult, Countdown, CountdownOptions, Stopwatch,
};

// ── Interactive Input re-exports ─────────────────────────────────────

pub use args::{args, ArgsConfig, ArgsResult, CommandDef, FlagDef, FlagType, FlagValue};
pub use command_router::{Command, CommandMatch, CommandRouter};
pub use exec::{Exec, ExecOptions};
pub use input_line::{InputLine, InputLineOptions, InputLineRender, PromptSource};
pub use keypress::{keypress, keypress_poll, keypress_stream, raw_mode, raw_mode_reset, KeyEvent};
pub use line_editor::{LineEditor, LineEditorState};
pub use prompt::{
    confirm, input, multiselect, password, select, ConfirmOptions, InputOptions,
    MultiSelectOptions, PasswordOptions, SelectOptions,
};
pub use repl::{readline, repl, ReadlineOptions, ReplCommand, ReplOptions};
