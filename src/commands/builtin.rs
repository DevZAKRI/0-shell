use crate::commands::CommandExecutor;
use crate::error::ShellError;

pub struct EchoCommand;
pub struct ExitCommand;

impl CommandExecutor for EchoCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        if args.is_empty() {
            println!();
        } else {
            println!("{}", args.join(" "));
        }
        Ok(())
    }

    fn help(&self) -> &str {
        "echo [text] - Display a line of text"
    }
}

impl CommandExecutor for ExitCommand {
    fn execute(&self, _args: &[String]) -> Result<(), ShellError> {
        std::process::exit(0);
    }

    fn help(&self) -> &str {
        "exit - Exit the shell"
    }
}
