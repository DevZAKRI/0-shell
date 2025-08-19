use crate::commands::CommandExecutor;
use crate::error::ShellError;

pub struct EchoCommand;
pub struct ExitCommand;

impl CommandExecutor for EchoCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        // TODO: Implement echo command
        // - Display arguments joined by spaces
        // - Handle empty arguments (print newline)
        todo!("Implement echo command")
    }

    fn help(&self) -> &str {
        "echo [text] - Display a line of text"
    }
}

impl CommandExecutor for ExitCommand {
    fn execute(&self, _args: &[String]) -> Result<(), ShellError> {
        // TODO: Implement exit command
        // - Exit shell cleanly
        todo!("Implement exit command")
    }

    fn help(&self) -> &str {
        "exit - Exit the shell"
    }
}
