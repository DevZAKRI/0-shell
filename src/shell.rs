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
        println!("{}", build_banner());
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

fn build_banner() -> String {
    let terminal_width = 80;

    let lines = [
        "   ____   _____    _____       _____   _    _ ",
        "  / __ \\ |  __ \\  / ____|     / ____| | |  | |",
        " | |  | || |__) || (___      | (___   | |__| |",
        " | |  | ||  ___/  \\___ \\      \\___ \\  |  __  |",
        " | |__| || |      ____) |     ____) | | |  | |",
        "  \\____/ |_|     |_____/     |_____/  |_|  |_|",
        "                                              ",
        "        ⚡ O P S - S H ⚡    FAST & LIGHT ⚡     ",
        "  >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> ",
    ];

    // Gradient colors for speed effect
    let colors = [
        "\x1b[91m", // red
        "\x1b[93m", // yellow
        "\x1b[96m", // cyan
    ];
    let reset = "\x1b[0m";

    let mut out = String::new();
    for (i, l) in lines.iter().enumerate() {
        let color = colors[i % colors.len()];

        // Center the line
        let padding = if l.len() < terminal_width {
            (terminal_width - l.len()) / 2
        } else {
            0
        };
        let centered_line = format!("{}{}", " ".repeat(padding), l);

        out.push_str(color);
        out.push_str(&centered_line);
        out.push_str(reset);
        out.push('\n');
    }

    if out.ends_with('\n') { out.pop(); }
    out
}
