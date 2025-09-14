use crate::commands::CommandExecutor;
use crate::error::ShellError;
use std::path::Path;
use std::fs;

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
    fn execute(&self, _args: &[String]) -> Result<(), ShellError> {
        // TODO: Implement pwd command
        // - Get current working directory
        // - Print to stdout
        todo!("Implement pwd command")
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
     let pwd = env::current_dir().unwrap().to_string_lossy().to_string(); // current directory as String


        if args.is_empty() {
            return Err(ShellError::ExecutionError("rm: missing operand".to_string()));
        }

        let mut recursive = false;
        let mut is_option = true;
        let mut targets: Vec<&String> = Vec::new();

        for arg in args {
           
            if arg.as_str() == pwd {
                return Err(ShellError::ExecutionError("rm: refusing to remove current directory".to_string()));
            }
            if is_dot_or_dotdot(arg) {
                return Err(
                    ShellError::ExecutionError("rm: refusing to remove '.' or '..'".to_string())
                );
            }
            if arg == "--" {
                is_option = false;
                continue;
            }
            if arg == "-r" && is_option {
                recursive = true;
            } else if arg.starts_with('-') && is_option {
                return Err(ShellError::InvalidOption(arg.clone()));
            } else {
                targets.push(arg);
            }
        }

        if targets.is_empty() {
            return Err(ShellError::ExecutionError("rm: missing operand".to_string()));
        }
        for target in targets {
            let path = Path::new(target);

            if !path.exists() {
                eprintln!("rm: cannot remove '{}': No such file or directory", target);
                continue;
            }
            let result = if path.is_dir() {
                if recursive {
                    fs::remove_dir_all(path)
                } else {
                    Err(
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("rm: cannot remove '{}': Is a directory (use -r)", target)
                        )
                    )
                }
            } else {
                fs::remove_file(path)
            };

            if let Err(err) = result {
                eprintln!("rm: failed to remove '{}': {}", target, err);
            }
        }
        Ok(())
    }

    fn help(&self) -> &str {
        "rm [-r] [file...] - Remove files or directories"
    }
}
fn is_dot_or_dotdot(path: &str) -> bool {
    path == "." || path == ".." || path == "./" || path == "../"
}
