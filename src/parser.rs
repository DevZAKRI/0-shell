use crate::error::ShellError;
use std::io;
use std::io::Write;

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
    
        Ok(Some(Command { name, args }))
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::error::ShellError;

//     fn create_parser() -> CommandParser {
//         CommandParser::new()
//     }

//     #[test]
//     fn test_empty_input() {
//         let parser = create_parser();
        
//         // Empty string
//         assert_eq!(parser.parse("").unwrap(), None);
        
//         // Whitespace only
//         assert_eq!(parser.parse("   ").unwrap(), None);
//         assert_eq!(parser.parse("\t\t\t").unwrap(), None);
//         assert_eq!(parser.parse(" \t \t ").unwrap(), None);
        
//         // Newlines only
//         assert_eq!(parser.parse("\n").unwrap(), None);
//         assert_eq!(parser.parse("\r\n").unwrap(), None);
//     }

//     #[test]
//     fn test_simple_commands() {
//         let parser = create_parser();
        
//         // Basic command
//         let result = parser.parse("ls").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "ls".to_string(),
//             args: vec![]
//         }));
        
//         // Command with one argument
//         let result = parser.parse("ls -la").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "ls".to_string(),
//             args: vec!["-la".to_string()]
//         }));
        
//         // Command with multiple arguments
//         let result = parser.parse("cp file1 file2").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "cp".to_string(),
//             args: vec!["file1".to_string(), "file2".to_string()]
//         }));
//     }

//     #[test]
//     fn test_whitespace_handling() {
//         let parser = create_parser();
        
//         // Leading/trailing whitespace
//         let result = parser.parse("  ls  ").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "ls".to_string(),
//             args: vec![]
//         }));
        
//         // Multiple spaces between arguments
//         let result = parser.parse("ls    -la    /home").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "ls".to_string(),
//             args: vec!["-la".to_string(), "/home".to_string()]
//         }));
        
//         // Mixed whitespace
//         let result = parser.parse("ls\t-la /home").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "ls".to_string(),
//             args: vec!["-la".to_string(), "/home".to_string()]
//         }));
//     }

//     #[test]
//     fn test_single_quotes() {
//         let parser = create_parser();
        
//         // Simple quoted string
//         let result = parser.parse("echo 'hello world'").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["hello world".to_string()]
//         }));
        
//         // Quoted string with spaces
//         let result = parser.parse("mkdir 'My Documents'").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "mkdir".to_string(),
//             args: vec!["My Documents".to_string()]
//         }));
        
//         // Multiple quoted arguments
//         let result = parser.parse("cp 'file 1' 'file 2'").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "cp".to_string(),
//             args: vec!["file 1".to_string(), "file 2".to_string()]
//         }));
        
//         // Mixed quoted and unquoted
//         let result = parser.parse("echo 'hello' world").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["hello".to_string(), "world".to_string()]
//         }));
//     }

//     #[test]
//     fn test_double_quotes() {
//         let parser = create_parser();
        
//         // Simple double quoted string
//         let result = parser.parse("echo \"hello world\"").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["hello world".to_string()]
//         }));
        
//         // Double quotes with spaces
//         let result = parser.parse("mkdir \"My Documents\"").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "mkdir".to_string(),
//             args: vec!["My Documents".to_string()]
//         }));
        
//         // Mixed quote types
//         let result = parser.parse("echo 'hello' \"world\"").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["hello".to_string(), "world".to_string()]
//         }));
//     }

//     #[test]
//     fn test_nested_quotes() {
//         let parser = create_parser();
        
//         // Single quotes inside double quotes
//         let result = parser.parse("echo \"It's working\"").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["It's working".to_string()]
//         }));
        
//         // Double quotes inside single quotes
//         let result = parser.parse("echo 'He said \"Hello\"'").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["He said \"Hello\"".to_string()]
//         }));
        
//         // Complex nested quotes
//         let result = parser.parse("echo \"She said 'It's great!'\"").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["She said 'It's great!'".to_string()]
//         }));
//     }

//     #[test]
//     fn test_escape_sequences() {
//         let parser = create_parser();
        
//         // Escaped quotes
//         let result = parser.parse("echo \"He said \\\"Hello\\\"\"").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["He said \"Hello\"".to_string()]
//         }));
        
//         let result = parser.parse("echo 'He said \\'Hello\\''").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["He said 'Hello'".to_string()]
//         }));
        
//         // Escaped backslash
//         let result = parser.parse("echo \"C:\\\\Windows\"").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["C:\\Windows".to_string()]
//         }));
        
//         // Escape sequences
//         let result = parser.parse("echo \"Line1\\nLine2\"").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["Line1\nLine2".to_string()]
//         }));
        
//         let result = parser.parse("echo \"Tab\\tseparated\"").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["Tab\tseparated".to_string()]
//         }));
        
//         let result = parser.parse("echo \"Carriage\\rreturn\"").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["Carriage\rreturn".to_string()]
//         }));
//     }

//     #[test]
//     fn test_escaped_spaces() {
//         let parser = create_parser();
        
//         // Escaped space outside quotes
//         let result = parser.parse("echo hello\\ world").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["hello world".to_string()]
//         }));
        
//         // Escaped space inside quotes (should be literal)
//         let result = parser.parse("echo \"hello\\ world\"").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["hello\\ world".to_string()]
//         }));
//     }

//     #[test]
//     fn test_special_characters() {
//         let parser = create_parser();
        
//         // Special characters in arguments
//         let result = parser.parse("echo $PATH").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["$PATH".to_string()]
//         }));
        
//         let result = parser.parse("ls *.txt").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "ls".to_string(),
//             args: vec!["*.txt".to_string()]
//         }));
        
//         let result = parser.parse("grep \"pattern\" file.txt").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "grep".to_string(),
//             args: vec!["pattern".to_string(), "file.txt".to_string()]
//         }));
//     }

//     #[test]
//     fn test_edge_cases() {
//         let parser = create_parser();
        
//         // Command with only special characters
//         let result = parser.parse("echo !@#$%^&*()").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["!@#$%^&*()".to_string()]
//         }));
        
//         // Empty quoted string
//         let result = parser.parse("echo \"\"").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["".to_string()]
//         }));
        
//         let result = parser.parse("echo ''").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["".to_string()]
//         }));
        
//         // Command name with special characters
//         let result = parser.parse("my-command arg1").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "my-command".to_string(),
//             args: vec!["arg1".to_string()]
//         }));
        
//         // Very long command
//         let long_arg = "a".repeat(1000);
//         let result = parser.parse(&format!("echo {}", long_arg)).unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec![long_arg]
//         }));
//     }

//     #[test]
//     fn test_unicode_characters() {
//         let parser = create_parser();
        
//         // Unicode in command name
//         let result = parser.parse("echo 你好").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["你好".to_string()]
//         }));
        
//         // Unicode in quoted string
//         let result = parser.parse("echo \"Hello 世界\"").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["Hello 世界".to_string()]
//         }));
        
//         // Emoji
//         let result = parser.parse("echo 🚀").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["🚀".to_string()]
//         }));
//     }

//     #[test]
//     fn test_malformed_quotes() {
//         let parser = create_parser();
        
//         // Unclosed single quote
//         let result = parser.parse("echo 'hello");
//         assert!(result.is_err());
        
//         // Unclosed double quote
//         let result = parser.parse("echo \"hello");
//         assert!(result.is_err());
        
//         // Mismatched quotes
//         let result = parser.parse("echo 'hello\"");
//         assert!(result.is_err());
        
//         let result = parser.parse("echo \"hello'");
//         assert!(result.is_err());
//     }

//     #[test]
//     fn test_escape_edge_cases() {
//         let parser = create_parser();
        
//         // Backslash at end of input
//         let result = parser.parse("echo hello\\");
//         assert!(result.is_err());
        
//         // Backslash followed by invalid escape
//         let result = parser.parse("echo \"hello\\x\"").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["hello\\x".to_string()]
//         }));
        
//         // Multiple backslashes
//         let result = parser.parse("echo \\\\\\\\").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["\\\\".to_string()]
//         }));
//     }

//     #[test]
//     fn test_complex_real_world_examples() {
//         let parser = create_parser();
        
//         // Complex command with multiple quotes and escapes
//         let result = parser.parse("grep -n \"pattern with 'quotes'\" file.txt").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "grep".to_string(),
//             args: vec!["-n".to_string(), "pattern with 'quotes'".to_string(), "file.txt".to_string()]
//         }));
        
//         // Command with path containing spaces
//         let result = parser.parse("cp \"My Documents/file.txt\" /home/user/").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "cp".to_string(),
//             args: vec!["My Documents/file.txt".to_string(), "/home/user/".to_string()]
//         }));
        
//         // Command with multiple escaped characters
//         let result = parser.parse("echo \"Line 1\\nLine 2\\tTabbed\"").unwrap();
//         assert_eq!(result, Some(Command {
//             name: "echo".to_string(),
//             args: vec!["Line 1\nLine 2\tTabbed".to_string()]
//         }));
//     }

//     #[test]
//     fn test_command_structure() {
//         let parser = create_parser();
        
//         // Verify command structure is correct
//         let result = parser.parse("ls -la /home/user").unwrap().unwrap();
//         assert_eq!(result.name, "ls");
//         assert_eq!(result.args, vec!["-la".to_string(), "/home/user".to_string()]);
        
//         // Verify args are properly separated
//         let result = parser.parse("cp file1 file2 file3").unwrap().unwrap();
//         assert_eq!(result.name, "cp");
//         assert_eq!(result.args, vec!["file1".to_string(), "file2".to_string(), "file3".to_string()]);
//     }
// }
