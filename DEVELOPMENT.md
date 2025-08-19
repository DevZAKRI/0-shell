# 0-Shell Development Guide

## Quick Start

### 1. **Setup Your Environment**
```bash
# Make sure you're in the project directory
cd /home/mzakri/0-shell

# Check that everything compiles
cargo check

# Run the shell (will show todo! panic until implemented)
cargo run
```

### 2. **Choose Your Tasks**
- Check `TASKS.md` for your assigned responsibilities
- Pick one function to implement first
- Work on your own feature branch

### 3. **Implementation Steps**

#### Example: Implementing the `echo` command

1. **Open the file**: `src/commands/builtin.rs`
2. **Find the function**: `EchoCommand::execute`
3. **Replace the todo!**: 
```rust
fn execute(&self, args: &[String]) -> Result<(), ShellError> {
    if args.is_empty() {
        println!();
    } else {
        println!("{}", args.join(" "));
    }
    Ok(())
}
```
4. **Test your implementation**:
```bash
cargo run
# Then in the shell: echo "Hello World"
```

## Development Workflow

### **For Each Function You Implement:**

1. **Plan**: Understand what the function should do
2. **Implement**: Replace `todo!()` with actual code
3. **Test**: Test with various inputs and edge cases
4. **Commit**: Save your work frequently
5. **Push**: Share your progress with the team

### **Testing Your Commands**

```bash
# Test compilation
cargo check

# Test specific command
cargo run
# Then test your command in the shell

# Compare with real Unix commands
# Your ls should behave like system ls
```

## Common Patterns

### **Error Handling**
```rust
// Always return Result<(), ShellError>
// Use ? operator for propagation
// Provide meaningful error messages
```

### **File Operations**
```rust
use std::fs;
use std::path::Path;

// Check if path exists
if Path::new(path).exists() {
    // Do something
}

// Handle errors gracefully
fs::read_to_string(path)
    .map_err(|e| ShellError::FileSystemError(format!("Failed to read {}: {}", path, e)))?
```

### **Command Arguments**
```rust
// Check argument count
if args.is_empty() {
    return Err(ShellError::ExecutionError("Missing argument".to_string()));
}

// Parse flags
let mut show_hidden = false;
for arg in args {
    if arg == "-a" {
        show_hidden = true;
    }
}
```

## Debugging Tips

### **When Things Don't Work:**

1. **Check compilation**: `cargo check`
2. **Add debug prints**: `println!("Debug: {:?}", variable);`
3. **Test step by step**: Implement one small piece at a time
4. **Compare with system commands**: Your output should match Unix behavior

### **Common Issues:**

- **Permission errors**: Check file permissions
- **Path issues**: Use absolute paths for testing
- **Empty arguments**: Always handle empty input gracefully
- **Error propagation**: Use `?` operator consistently

## Team Collaboration

### **Before Starting:**
- Read the function you're implementing
- Understand the expected behavior
- Check if it depends on other functions

### **While Working:**
- Test frequently
- Commit small, working pieces
- Ask for help if stuck

### **After Completing:**
- Test edge cases
- Update TASKS.md
- Help teammates with their functions

## Ready to Start?

1. **Pick your first task** from `TASKS.md`
2. **Create your branch**: `git checkout -b feature/your-name/function-name`
3. **Implement one function** completely
4. **Test it thoroughly**
5. **Commit and push**

Remember: Start simple, test often, and don't hesitate to ask for help! 🚀
