pub mod ansi;
pub mod error;
pub mod style;
pub mod unicode;
pub mod writer;

pub use error::{PrismError, PrismResult};
pub use style::{s, style, color, rgb, hex, Color, Style, RESET};
pub use writer::{is_tty, interactive_tty, ansi_enabled, term_width};
