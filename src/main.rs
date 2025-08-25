#![allow(dead_code, unused_variables)]
mod shell;
mod commands;
mod parser;
mod error;

use shell::Shell;

fn main() {
    let mut shell = Shell::new();
    
    if let Err(e) = shell.run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}