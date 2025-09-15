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
        let mut token_had_quotes = false;
        let mut chars = trimmed.chars();
    
        while let Some(ch) = chars.next() {
            match ch {
                '"' | '\'' => {
                    if !in_quotes {
                        in_quotes = true;
                        quote_char = ch;
                        token_had_quotes = true;
                    } else if ch == quote_char {
                        in_quotes = false;
                        quote_char = '\0';
                    } else {
                        current_part.push(ch);
                    }
                }
                '\n' => {
                    if in_quotes {
                        current_part.push('\n');
                    } else {
                        if !current_part.is_empty() {
                            let pushed = if !token_had_quotes {
                                expand_tilde_word(&current_part)
                            } else {
                                current_part.clone()
                            };
                            parts.push(pushed);
                            current_part.clear();
                            token_had_quotes = false;
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
                                '\n' => {
                                    // Backslash-newline is removed (line continuation)
                                }
                                _ => {
                                    current_part.push('\\');
                                    current_part.push(next_ch);
                                }
                            }
                        } else {
                            return Err(ShellError::IncompleteInput('\\'));
                        }
                    } else {
                        if let Some(next_ch) = chars.next() {
                            match next_ch {
                                ' ' => {
                                    current_part.push(' ');
                                }
                                '\n' => {
                                    // Backslash-newline is removed (line continuation)
                                }
                                _ => current_part.push(next_ch),
                            }
                        } else {
                            return Err(ShellError::IncompleteInput('\\'));
                        }
                    }
                }
                ' ' | '\t' => {
                    if !in_quotes {
                        if !current_part.is_empty() {
                            let pushed = if !token_had_quotes {
                                expand_tilde_word(&current_part)
                            } else {
                                current_part.clone()
                            };
                            parts.push(pushed);
                            current_part.clear();
                            token_had_quotes = false;
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
            return Err(ShellError::IncompleteInput(quote_char));
        }
    
        if !current_part.is_empty() {
            let pushed = if !token_had_quotes {
                expand_tilde_word(&current_part)
            } else {
                current_part.clone()
            };
            parts.push(pushed);
        } else if in_quotes == false && !parts.is_empty() {
            // Handle empty quoted strings as arguments
            parts.push(String::new());
        }
    
        if parts.is_empty() {
            return Ok(None);
        }
    
        let name = parts[0].clone();
        let args = parts[1..].to_vec();
    
        Ok(Some(Command { name, args }))
    }
}
fn expand_tilde_word(word: &str) -> String {
    if word == "~" {
        return std::env::var("HOME").unwrap_or_else(|_| String::from("~"));
    }
    if let Some(rest) = word.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}/{}", home, rest);
        }
    }
    word.to_string()
}

