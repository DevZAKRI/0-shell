use crate::error::ShellError;
use std::io;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
}

pub struct CommandParser;

impl CommandParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, input: &str) -> Result<Option<Command>, ShellError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(None);
        }

        let mut parts = Vec::new();
        let mut current_part = String::new();
        let mut in_quotes = false;
        let mut quote_char = '\0';
        let mut chars = trimmed.chars();

        while let Some(ch) = chars.next() {
            match ch {
                '"' | '\'' => {
                    if !in_quotes {
                        in_quotes = true;
                        quote_char = ch;
                    } else if ch == quote_char {
                        in_quotes = false;
                        quote_char = '\0';
                    } else {
                        current_part.push(ch);
                    }
                }
                '\\' => {
                    if in_quotes {
                        if let Some(next_ch) = chars.next() {
                            match next_ch {
                                'n' => current_part.push('\n'),
                                't' => current_part.push('\t'),
                                'r' => current_part.push('\r'),
                                '\\' | '"' | '\'' => current_part.push(next_ch),
                                _ => {
                                    current_part.push('\\');
                                    current_part.push(next_ch);
                                }
                            }
                        } else {
                            current_part.push('\\');
                        }
                    } else {
                        current_part.push(ch);
                    }
                }
                ' ' | '\t' => {
                    if !in_quotes {
                        if !current_part.is_empty() {
                            parts.push(current_part.clone());
                            current_part.clear();
                        }
                    } else {
                        current_part.push(ch);
                    }
                }
                _ => {
                    current_part.push(ch);
                }
            }
        }

        if in_quotes {
            print!("> ");
            io::stdout().flush().map_err(|e| ShellError::IoError(e))?;

            let mut input_continuation = String::new();

            io::stdin().read_line(&mut input_continuation).map_err(|e| ShellError::IoError(e))?;
            input_continuation.trim().to_string();
            let continued_input = format!("{}\n{}\n", trimmed, input_continuation);
            return self.parse(&continued_input);
        }

        if !current_part.is_empty() {
            parts.push(current_part);
        }

        if parts.is_empty() {
            return Ok(None);
        }

        let name = parts[0].clone();
        let args = parts[1..].to_vec();

        Ok(Some(Command { name, args }))
    }
}
