use crate::commands::CommandExecutor;
use crate::error::ShellError;
use std::fs;
use std::io::{self, Read, Write};

pub struct PwdCommand;
pub struct CdCommand;
pub struct LsCommand;
pub struct CatCommand;
pub struct MkdirCommand;
pub struct CpCommand;
pub struct MvCommand;
pub struct RmCommand;

pub struct CommandOptions {
    is_option: bool,
}
    

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
        let mut command_options = CommandOptions {
            is_option: true,
        };
        // If no arguments provided, read from stdin
        if args.is_empty() {
            return self.read_from_stdin();
        }
        
        //let mut has_error = false;

        for file_path in args {
            if file_path == "--" {
                command_options.is_option = false;
                if args.len() == 1 {
                    return self.read_from_stdin();
                }
                continue;
            }
            if file_path.starts_with('-') && file_path != "-" && command_options.is_option {
                return Err(ShellError::InvalidOption(file_path.clone()));
            }
            match self.process_file(file_path) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("cat: {}: {}", file_path, e);
                    //has_error = true;
                }
            }
        }

        // if has_error {
        //     Err(ShellError::ExecutionError("Some files could not be processed".to_string()))
        // } else {
        //     Ok(())
        // }
        Ok(())
    }

    fn help(&self) -> &str {
        "cat [file...] - Concatenate and display files (reads from stdin if no files provided)"
    }
}

impl CatCommand {
    /// Read from stdin and write to stdout
    fn read_from_stdin(&self) -> Result<(), ShellError> {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut stdout = io::stdout();
        
        let mut buffer = [0; 8192];
        loop {
            match handle.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    stdout.write_all(&buffer[..n])
                        .map_err(|e| ShellError::IoError(e))?;
                }
                Err(e) => return Err(ShellError::IoError(e)),
            }
        }
        
        stdout.flush().map_err(|e| ShellError::IoError(e))?;
        Ok(())
    }

    fn process_file(&self, file_path: &str) -> Result<(), ShellError> {
        if file_path == "-" {
            return self.read_from_stdin();
        }
        let content = fs::read(file_path)
            .map_err(|e| {
                match e.kind() {
                    io::ErrorKind::NotFound => {
                        ShellError::FileSystemError("No such file or directory".to_string())
                    }
                    io::ErrorKind::PermissionDenied => {
                        ShellError::FileSystemError("Permission denied".to_string())
                    }
                    io::ErrorKind::InvalidInput => {
                        ShellError::FileSystemError("Is a directory".to_string())
                    }
                    _ => ShellError::FileSystemError(format!("Cannot read file: {}", e))
                }
            })?;

        io::stdout().write_all(&content)
            .map_err(|e| ShellError::IoError(e))?;
        io::stdout().flush().map_err(|e| ShellError::IoError(e))?;
        
        Ok(())
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
