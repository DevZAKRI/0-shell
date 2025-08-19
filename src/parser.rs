use crate::error::ShellError;

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
}

pub struct CommandParser;

impl CommandParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, input: &str) -> Result<Option<Command>, ShellError> {
        // TODO: Parse user input into Command struct
        // - Split input on whitespace
        // - Extract command name and arguments
        // - Handle empty input
        // - Return None for empty input, Some(Command) for valid input
        todo!("Implement command parsing")
    }
}
