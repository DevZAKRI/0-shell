use crate::commands::CommandExecutor;
use crate::error::ShellError;
use crate::commands::filesystem::*;

pub struct EchoCommand;
pub struct ExitCommand;
pub struct HelpCommand;

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


impl CommandExecutor for HelpCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        if args.is_empty() || args.len() > 1 {
            println!(
                "Usage: help [command]\n\
                 Only one command is allowed\n\
                 Available commands:\n\
                 echo, exit, help, pwd, cd, ls, cat, mkdir, cp, mv, rm"
            );
            return Ok(());
        }

        let cmd = &args[0];
        match cmd.as_str() {
            "echo"  => println!("{}", EchoCommand.help()),
            "exit"  => println!("{}", ExitCommand.help()),
            "help"  => println!("{}", HelpCommand.help()),
            "pwd"   => println!("{}", PwdCommand.help()),
            "cd"    => println!("{}", CdCommand.help()),
            "ls"    => println!("{}", LsCommand.help()),
            "cat"   => println!("{}", CatCommand.help()),
            "mkdir" => println!("{}", MkdirCommand.help()),
            "cp"    => println!("{}", CpCommand.help()),
            "mv"    => println!("{}", MvCommand.help()),
            "rm"    => println!("{}", RmCommand.help()),
            _ => println!("Unknown command: {}", cmd),
        }

        Ok(())
    }

    fn help(&self) -> &str {
        "help - Display help information"
    }
}