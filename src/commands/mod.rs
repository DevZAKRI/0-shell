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
        // TODO: Initialize command registry
        // TODO: Register all built-in commands
        todo!("Implement command registry initialization")
    }

    fn register_builtin_commands(&mut self) {
        // TODO: Register all 10 required commands:
        // - echo, cd, ls, pwd, cat, cp, rm, mv, mkdir, exit
        todo!("Implement command registration")
    }

    pub fn execute(&self, command: &Command) -> Result<(), ShellError> {
        // TODO: Execute command using registry
        // - Look up command in registry
        // - Execute if found, return error if not found
        todo!("Implement command execution")
    }
}
