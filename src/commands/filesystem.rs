use crate::commands::CommandExecutor;
use crate::error::ShellError;
use std::fs;
use std::path::Path;
use std::io;
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
        if args.len() < 2 {
            return Err(ShellError::ExecutionError("cp: missing operand".to_string()));
        }
        // Check for -r flag
        let recursive = args.contains(&"-r".to_string());

        // Filter arguments to get sources and destination
        let mut filtered: Vec<&String> = args.iter().filter(|a| a != &&"-r".to_string()).collect();

         if filtered.len() < 2 {
            return Err(ShellError::ExecutionError("cp: missing an operand".to_string()));
        }
      
        let target = Path::new(filtered.pop().unwrap()); // last argument = destination
        let sources = filtered; // rest = sources

        for src in sources {
            let src_path = Path::new(src);

            if !src_path.exists() {
                eprintln!("cp: cannot stat '{}': No such file or directory", src);
                continue;
            }

            if src_path.is_dir() {
                if !recursive {
                    eprintln!("cp: omitting directory '{}', use -r to copy", src);
                    continue;
                }
                // Copy directory recursively
                if let Err(err) = copy_dir_all(src_path, &target.join(src_path.file_name().unwrap())) {
                    eprintln!("cp: failed to copy directory '{}': {}", src, err);
                }
            } else {
                // Determine destination path
                let dest_path = if target.is_dir() {
                    target.join(src_path.file_name().unwrap())
                } else {
                    target.to_path_buf()
                };

                if let Err(err) = fs::copy(src_path, &dest_path) {
                    eprintln!("cp: failed to copy '{}': {}", src, err);
                }
            }
        }

        Ok(())
    }

    fn help(&self) -> &str {
        "cp source... destination - Copy files and directories"
    }
}

/// Recursively copy a directory
fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_all(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path)?;
        }
    }
    Ok(())
}

impl CommandExecutor for MvCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        if args.len() < 2 {
            return Err(ShellError::ExecutionError("mv: missing operand".to_string()));
        }
        // Last argument = destination
        let filtered = args;
        let target = Path::new(&filtered[filtered.len() - 1]);
        let sources = &filtered[..filtered.len() - 1];

        for src in sources {
            let src_path = Path::new(src);

            if !src_path.exists() {
                eprintln!("mv: cannot stat '{}': No such file or directory", src);
                continue;
            }

            let dest_path = if target.is_dir() {
                target.join(src_path.file_name().unwrap())
            } else {
                target.to_path_buf()
            };

            if let Err(err) = fs::rename(src_path, &dest_path) {
                eprintln!("mv: failed to move '{}': {}", src, err);
            }
        }

        Ok(())
    }

    fn help(&self) -> &str {
        "mv source... destination - Move (rename) files or directories"
    }
}

impl CommandExecutor for RmCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
         if args.is_empty() {
            return Err(ShellError::ExecutionError("rm: missing operand".to_string()));
        }
        // Check if -r flag is present
        let recursive = args.contains(&"-r".to_string());
        // Collect actual targets (filter out flags)
        let targets: Vec<&String> = args.iter().filter(|a| a != &&"-r".to_string()).collect();
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
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("rm: cannot remove '{}': Is a directory (use -r)", target),
                    ))
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
