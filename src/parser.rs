use crate::error::ShellError;

#[derive(Debug, Clone, PartialEq)]

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
                '\n' => {
                    if in_quotes {
                        // Inside quotes, newline is treated as literal newline
                        current_part.push('\n');
                    } else {
                        // Outside quotes, newline ends the current part
                        if !current_part.is_empty() {
                            parts.push(current_part.clone());
                            current_part.clear();
                        }
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
                        if let Some(next_ch) = chars.next() {
                            match next_ch {
                                ' ' => {
                                    current_part.push(' ');
                                }
                                _ => current_part.push(ch),
                            }
                        } else {
                            // Backslash at end of input is an error
                            return Err(ShellError::ParseError("Unexpected end of input after backslash".to_string()));
                        }
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
    
        // Check for unclosed quotes and return a special error
        if in_quotes {
            return Err(ShellError::IncompleteInput(quote_char));
        }
    
        if !current_part.is_empty() {
            parts.push(current_part);
        } else if in_quotes == false && !parts.is_empty() {
            // Handle empty quoted strings as arguments
            parts.push(String::new());
        }
    
        if parts.is_empty() {
            return Ok(None);
        }
    
        let name = parts[0].clone();
        let args = parts[1..].to_vec();
    
    
        let name = parts[0].clone();
        let args = parts[1..].to_vec();
    
        Ok(Some(Command { name, args }))
    }
}
