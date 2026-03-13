use std::fmt;

#[derive(Debug)]
pub enum PrismError {
    /// User pressed Ctrl+C or external cancellation
    Cancelled,
    /// Terminal I/O failure
    Io(std::io::Error),
}

impl fmt::Display for PrismError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrismError::Cancelled => write!(f, "cancelled"),
            PrismError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for PrismError {}

impl From<std::io::Error> for PrismError {
    fn from(err: std::io::Error) -> Self {
        PrismError::Io(err)
    }
}

pub type PrismResult<T> = Result<T, PrismError>;
