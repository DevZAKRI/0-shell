use std::io::{ self, Write };
use crate::commands::CommandRegistry;
use crate::parser::CommandParser;
use crate::error::ShellError;

pub struct Shell {
    command_registry: CommandRegistry,
    parser: CommandParser,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            command_registry: CommandRegistry::new(),
            parser: CommandParser::new(),
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

    fn display_prompt(&self) -> Result<(), ShellError> {
        print!("$ ");
        io
            ::stdout()
            .flush()
            .map_err(|e| ShellError::IoError(e))?;
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
        Ok(Some(input.trim().to_string()))
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
            complete_input.push(' '); // Add space between lines

            // Try to parse the input to check if it's complete
            match self.parser.parse(&complete_input) {
                Ok(_) => {
                    // Input is complete, break out of the loop
                    break;
                }
                Err(ShellError::IncompleteInput(_)) => {
                    // Input is incomplete, continue reading
                    self.display_continuation_prompt()?;
                    continue;
                }
                Err(e) => {
                    // Some other error, return it
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