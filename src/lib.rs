pub mod ansi;
pub mod badge;
pub mod cursor;
pub mod elapsed;
pub mod error;
pub mod log;
pub mod style;
pub mod text;
pub mod timer;
pub mod unicode;
pub mod writer;

pub use error::{PrismError, PrismResult};
pub use style::{s, style, color, rgb, hex, Color, Style, RESET};
pub use writer::{is_tty, interactive_tty, ansi_enabled, term_width};
