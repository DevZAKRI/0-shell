use std::io;
use std::fmt;


#[derive(Debug)]
pub enum ShellError {
    IoError(io::Error),
    ParseError(String),
    CommandNotFound(String),
    ExecutionError(String),
    FileSystemError(String),
    IncompleteInput(char), // For unclosed quotes
    InvalidOption(String),
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShellError::IoError(e) => write!(f, "I/O error: {}", e),
            ShellError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ShellError::CommandNotFound(cmd) => write!(f, "Command '{}' not found", cmd),
            ShellError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            ShellError::FileSystemError(msg) => write!(f, "File system error: {}", msg),
            ShellError::IncompleteInput(ch) => write!(f, "Incomplete input: unclosed quote '{}'", ch),
            ShellError::InvalidOption(msg) => write!(f, "Invalid option: {}", msg),
        }
    }
}

impl From<io::Error> for ShellError {
    fn from(err: io::Error) -> Self {
        ShellError::IoError(err)
    }
}

impl std::error::Error for ShellError {}