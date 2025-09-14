mod builtin;
mod filesystem;

use crate::error::ShellError;
use crate::parser::Command;
use std::collections::HashMap;

pub trait CommandExecutor {
    fn execute(&self, args: &[String]) -> Result<(), ShellError>;
    fn help(&self) -> &str;
}

pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn CommandExecutor>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };
        
        registry.register_builtin_commands();
        
        registry
    }

    fn register_builtin_commands(&mut self) {
        self.commands.insert("echo".to_string(), Box::new(builtin::EchoCommand));
        self.commands.insert("exit".to_string(), Box::new(builtin::ExitCommand));
        
        self.commands.insert("pwd".to_string(), Box::new(filesystem::PwdCommand));
        self.commands.insert("cd".to_string(), Box::new(filesystem::CdCommand));
        self.commands.insert("ls".to_string(), Box::new(filesystem::LsCommand));
        self.commands.insert("cat".to_string(), Box::new(filesystem::CatCommand));
        self.commands.insert("mkdir".to_string(), Box::new(filesystem::MkdirCommand));
        self.commands.insert("cp".to_string(), Box::new(filesystem::CpCommand));
        self.commands.insert("mv".to_string(), Box::new(filesystem::MvCommand));
        self.commands.insert("rm".to_string(), Box::new(filesystem::RmCommand));
        self.commands.insert("help".to_string(), Box::new(builtin::HelpCommand));
    }

    pub fn execute(&self, command: &Command) -> Result<(), ShellError> {
        if let Some(executor) = self.commands.get(&command.name) {
            executor.execute(&command.args)
        } else {
            Err(ShellError::CommandNotFound(command.name.clone()))
        }
    }
}
