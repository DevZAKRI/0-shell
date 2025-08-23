use crate::commands::CommandExecutor;
use crate::error::ShellError;
use std::env;

pub struct PwdCommand;
pub struct CdCommand;
pub struct LsCommand;
pub struct CatCommand;
pub struct MkdirCommand;
pub struct CpCommand;
pub struct MvCommand;
pub struct RmCommand;

impl CommandExecutor for PwdCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        if args.len() > 0 {
            return Err(ShellError::ExecutionError("pwd: too many arguments".to_string()));
        }
        
        println!("{}", env::current_dir()?.display());
        Ok(())
    }

    fn help(&self) -> &str {
        "pwd - Print working directory"
    }
}

impl CommandExecutor for CdCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        // TODO: Implement cd command
        // - Change to specified directory
        // - Handle no arguments (go to home directory)
        // - Handle relative and absolute paths
        todo!("Implement cd command")
    }

    fn help(&self) -> &str {
        "cd [directory] - Change directory"
    }
}

impl CommandExecutor for LsCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        // TODO: Implement ls command with flags:
        // - -a: show hidden files
        // - -l: long format (permissions, size, dates)
        // - -F: add file type indicators
        todo!("Implement ls command")
    }

    fn help(&self) -> &str {
        "ls [-a] [-l] [-F] - List directory contents"
    }
}

impl CommandExecutor for CatCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        // TODO: Implement cat command
        // - Read and display file contents
        // - Handle multiple files
        // - Handle missing files gracefully
        todo!("Implement cat command")
    }

    fn help(&self) -> &str {
        "cat [file...] - Concatenate and display files"
    }
}

impl CommandExecutor for MkdirCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        // TODO: Implement mkdir command
        // - Create directories
        // - Handle multiple directories
        // - Handle existing directories gracefully
        todo!("Implement mkdir command")
    }

    fn help(&self) -> &str {
        "mkdir [directory...] - Create directories"
    }
}

impl CommandExecutor for CpCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        // TODO: Implement cp command
        // - Copy files and directories
        // - Handle file to file copying
        // - Handle file to directory copying
        todo!("Implement cp command")
    }

    fn help(&self) -> &str {
        "cp source destination - Copy files and directories"
    }
}

impl CommandExecutor for MvCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        // TODO: Implement mv command
        // - Move/rename files and directories
        // - Handle file to file moving
        // - Handle file to directory moving
        todo!("Implement mv command")
    }

    fn help(&self) -> &str {
        "mv source destination - Move (rename) files"
    }
}

impl CommandExecutor for RmCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        // TODO: Implement rm command with -r flag
        // - Remove files
        // - Handle -r flag for recursive directory removal
        // - Handle missing files gracefully
        todo!("Implement rm command")
    }

    fn help(&self) -> &str {
        "rm [-r] [file...] - Remove files or directories"
    }
}
