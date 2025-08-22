use std::io::{self, Write};
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
            
            self.display_prompt()?; // "$ ola libghit tb9a tban dima"
            
           
            let input = self.read_input()?;
            
            // Handle EOF (Ctrl+D)
            if input.is_empty() {
                println!();
                break;
            }
            
            // execute command after parsing or err
            if let Err(e) = self.execute_command(&input) {
                eprintln!("Error: {}", e);
            }
        }
        
        Ok(())
    }

    fn display_prompt(&self) -> Result<(), ShellError> {
        print!("$ ");
        io::stdout().flush().map_err(|e| ShellError::IoError(e))?;
        Ok(())
    }

    fn read_input(&self) -> Result<String, ShellError> {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| ShellError::IoError(e))?;
        
        Ok(input.trim().to_string())
    }

    fn execute_command(&mut self, input: &str) -> Result<(), ShellError> {
        let command = self.parser.parse(input)?;
        
        if let Some(cmd) = command {
            self.command_registry.execute(&cmd)?;
        }
        
        Ok(())
    }
}