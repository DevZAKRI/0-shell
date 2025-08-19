# 0-Shell

A minimalist Unix-like shell implemented in Rust, designed to run core Unix commands using system calls—without relying on external binaries or built-in shells.

## Features

- **Built-in Commands**: All commands implemented from scratch using Rust's standard library
- **File System Operations**: Complete file and directory management
- **Error Handling**: Robust error handling with user-friendly messages
- **Unix Compliance**: Follows standard Unix shell conventions

## Supported Commands

| Command | Description | Flags |
|---------|-------------|-------|
| `echo` | Display text | None |
| `cd` | Change directory | None |
| `ls` | List directory contents | `-a`, `-l`, `-F` |
| `pwd` | Print working directory | None |
| `cat` | Display file contents | None |
| `cp` | Copy files | None |
| `rm` | Remove files | `-r` (recursive) |
| `mv` | Move/rename files | None |
| `mkdir` | Create directories | None |
| `exit` | Exit the shell | None |

## Project Structure

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
├── ROADMAP.md           # Development roadmap
└── README.md
```

## Requirements

- Rust 1.70+ (edition 2021)
- Linux/Unix system
- No external dependencies beyond Rust standard library

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd 0-shell
```

2. Build the project:
```bash
cargo build --release
```

3. Run the shell:
```bash
./target/release/0-shell
```

## Usage

The shell provides a simple command-line interface:

```bash
$ pwd
/home/user/projects/0-shell

$ ls -la
total 8
drwxr-xr-x  2 user user 4096 Jan 01 12:00 .
drwxr-xr-x  3 user user 4096 Jan 01 12:00 ..
-rw-r--r--  1 user user    0 Jan 01 12:00 file.txt

$ echo "Hello, World!"
Hello, World!

$ cat file.txt
Hello, World!

$ exit
```

## Development

### Building
```bash
cargo build        # Debug build
cargo build --release  # Release build
```

### Testing
```bash
cargo test        # Run tests
cargo check       # Check compilation without building
```

### Running
```bash
cargo run         # Run in debug mode
cargo run --release  # Run in release mode
```

## Architecture

The shell is built with a modular architecture:

- **Shell**: Main shell loop and user interaction
- **Parser**: Command parsing and argument extraction
- **Commands**: Registry and execution of built-in commands
- **Error Handling**: Comprehensive error types and handling

## Learning Objectives

This project demonstrates:
- File and directory operations using Rust's standard library
- User input/output management in a shell environment
- Robust error handling and user experience
- Unix system programming concepts
- Rust's safety and abstraction features

## Constraints

- No external binaries or system calls that spawn them
- Only basic command syntax (no pipes, redirection, globbing)
- Shell behavior aligns with Unix conventions
- Code follows good coding practices

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is open source and available under the [MIT License](LICENSE).

## Roadmap

See [ROADMAP.md](ROADMAP.md) for detailed development phases and future enhancements.