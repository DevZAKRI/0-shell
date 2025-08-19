# 0-Shell Project Roadmap

## Project Overview
Build a minimalist Unix-like shell in Rust that runs core Unix commands using system calls—without relying on external binaries or built-in shells.

## Learning Objectives
- Work with file and directory operations
- Manage user input and output within a shell loop
- Implement robust error handling
- Gain experience in Unix process and system call APIs

## Core Requirements
Your shell must implement these commands from scratch:
- `echo`, `cd`, `ls` (with -l, -a, -F flags), `pwd`, `cat`, `cp`, `rm` (with -r), `mv`, `mkdir`, `exit`

## Phase 1: Foundation & Basic Shell Loop (Week 1)
**Goal**: Create the basic shell structure and command loop

**Tasks**:
1. **Project Setup**
   - Initialize Rust project with `cargo new 0-shell`
   - Set up project structure and dependencies
   - Create basic error handling types

2. **Core Shell Loop**
   - Implement main shell loop with prompt (`$ `)
   - Handle user input (stdin/stdout)
   - Graceful exit on Ctrl+D (EOF)
   - Basic command parsing (split on whitespace)

3. **Command Framework**
   - Create command trait/interface
   - Implement command registry system
   - Basic error handling for unknown commands

**Deliverable**: Shell that shows prompt, accepts input, and exits cleanly

---

## Phase 2: File System Commands (Week 2)
**Goal**: Implement core file and directory operations

**Tasks**:
1. **Navigation Commands**
   - `pwd` - Get current working directory
   - `cd` - Change directory with path validation

2. **File Listing**
   - `ls` - Basic directory listing
   - `ls -a` - Show hidden files
   - `ls -l` - Long format with permissions, size, dates
   - `ls -F` - Add file type indicators

3. **File Operations**
   - `cat` - Display file contents
   - `mkdir` - Create directories

**Deliverable**: Working file system navigation and basic file operations

---

## Phase 3: File Management Commands (Week 3)
**Goal**: Complete file manipulation capabilities

**Tasks**:
1. **File Manipulation**
   - `cp` - Copy files (single file to file, file to directory)
   - `mv` - Move/rename files and directories
   - `rm` - Remove files
   - `rm -r` - Recursive directory removal

2. **Utility Commands**
   - `echo` - Display text with argument handling
   - `exit` - Clean shell termination

**Deliverable**: Full set of file management commands working

---

## Phase 4: Error Handling & Polish (Week 4)
**Goal**: Robust error handling and Unix compliance

**Tasks**:
1. **Error Handling**
   - Comprehensive error messages
   - Graceful handling of file permission errors
   - Invalid path handling
   - Command not found messages

2. **Unix Compliance**
   - Proper exit codes
   - Standard error output (stderr)
   - File permission handling
   - Path validation

3. **Testing & Validation**
   - Test all commands with various inputs
   - Edge case handling
   - Performance optimization

**Deliverable**: Production-ready shell with robust error handling

---

## Phase 5: Bonus Features (Optional - Week 5+)
**Goal**: Enhanced user experience and advanced features

**Tasks**:
1. **Signal Handling**
   - Ctrl+C (SIGINT) handling
   - Graceful process termination

2. **Shell Enhancements**
   - Command history (up/down arrows)
   - Auto-completion for files/directories
   - Dynamic prompt showing current directory
   - Colorized output

3. **Advanced Features**
   - Command chaining with `;`
   - Basic piping (`|`)
   - I/O redirection (`>`, `<`)
   - Environment variable support
   - Custom help command

---

## Technical Implementation Strategy

### Core Architecture
```
0-shell/
├── src/
│   ├── main.rs          # Entry point and shell loop
│   ├── shell.rs         # Main shell logic
│   ├── commands/        # Command implementations
│   │   ├── mod.rs       # Command registry
│   │   ├── builtin.rs   # Built-in commands
│   │   └── filesystem.rs # File operations
│   ├── parser.rs        # Command parsing
│   └── error.rs         # Error handling
├── Cargo.toml
└── README.md
```

### Key Dependencies
- `nix` - Unix system calls and constants
- `libc` - Low-level system interfaces
- `termion` - Terminal handling (for bonus features)

### Development Approach
1. **Start Simple**: Basic shell loop first, then add commands one by one
2. **Test Incrementally**: Each command should work before moving to the next
3. **Unix First**: Always test against standard Unix behavior
4. **Error First**: Implement error handling alongside functionality

### Success Criteria
- Shell runs without external dependencies
- All 10 required commands work correctly
- Handles edge cases gracefully
- Follows Unix conventions
- Clean, maintainable Rust code

### Testing Strategy
- Test each command individually
- Test edge cases (empty files, permissions, invalid paths)
- Compare output with standard Unix commands
- Test error conditions and error messages

### Constraints
- No external binaries or system calls that spawn them
- Only basic command syntax required (no pipes, redirection, globbing)
- Shell behavior must align with Unix conventions
- Code must follow good coding practices
