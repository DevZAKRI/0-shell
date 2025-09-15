use std::io::{ self, Write };
use crate::commands::CommandRegistry;
use crate::parser::CommandParser;
use crate::error::ShellError;

pub struct Shell {
    command_registry: CommandRegistry,
    parser: CommandParser,
    last_dir: std::path::PathBuf,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            command_registry: CommandRegistry::new(),
            parser: CommandParser::new(),
            last_dir: std::path::PathBuf::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), ShellError> {
        loop {
            self.display_prompt()?;

            let input = self.read_complete_input()?;

            match input {
                None => {
                    println!();
                    break;
                }
                Some(input) if input.is_empty() => {
                    continue;
                }
                Some(input) => {
                    if let Err(e) = self.execute_command(&input) {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        }
        Ok(())
    }

    fn display_prompt(&mut self) -> Result<(), ShellError> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());

        let current_dir = match std::env::current_dir() {
            Ok(dir) => {
                self.last_dir = dir.clone(); 
                dir
            }
            Err(_) => {
                self.last_dir.clone()
            }
        };

        let mut display_dir = current_dir.to_string_lossy().to_string();
        if display_dir.starts_with(&home) {
            display_dir = display_dir.replacen(&home, "~", 1);
        }

        print!("{} $ ", display_dir);
        std::io::stdout().flush().map_err(ShellError::IoError)?;
        Ok(())
    }
    
    

    fn display_continuation_prompt(&self) -> Result<(), ShellError> {
        print!("> ");
        io
            ::stdout()
            .flush()
            .map_err(|e| ShellError::IoError(e))?;
        Ok(())
    }

    fn read_input(&self) -> Result<Option<String>, ShellError> {
        let mut input = String::new();
        let bytes_read = io
            ::stdin()
            .read_line(&mut input)
            .map_err(|e| ShellError::IoError(e))?;

        if bytes_read == 0 {
            return Ok(None);
        }
        Ok(Some(input))
    }

    fn read_complete_input(&self) -> Result<Option<String>, ShellError> {
        let mut complete_input = String::new();

        loop {
            let input = self.read_input()?;
            let line = match input {
                Some(i) => i,
                None => {
                    return Ok(None);
                }
            };

            if line.is_empty() && complete_input.is_empty() {
                return Ok(Some(String::new()));
            }

            complete_input.push_str(&line);
            
            match self.parser.parse(&complete_input) {
                Ok(_) => {
                    break;
                }
                Err(ShellError::IncompleteInput(_)) => {
                    self.display_continuation_prompt()?;
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(Some(complete_input.trim().to_string()))
    }

    fn execute_command(&mut self, input: &str) -> Result<(), ShellError> {
        let command = self.parser.parse(input)?;

        if let Some(cmd) = command {
            self.command_registry.execute(&cmd)?;
        }

        Ok(())
    }
}
