# LS Command Documentation

## Overview
The `ls` command in this shell implementation provides comprehensive directory listing functionality with support for various flags and features. It's implemented in Rust using only standard library functions, ensuring portability and reliability.

## Command Syntax
```
ls [-a] [-l] [-F] [directory...]
```

### Flags
- `-a`: Show hidden files (files starting with '.')
- `-l`: Long format display with detailed information
- `-F`: Add file type indicators to filenames

### Arguments
- `directory...`: One or more directories to list (defaults to current directory '.')

## Architecture

### Core Components

#### 1. LsFlags Structure
```rust
struct LsFlags {
    show_hidden: bool,      // -a flag
    long_format: bool,      // -l flag  
    file_indicators: bool,  // -F flag
}
```

#### 2. Main Execution Flow
1. **Parse Arguments**: Extract flags and directory paths
2. **Process Directories**: For each specified directory (or current directory)
3. **Read Directory**: Get directory entries using `fs::read_dir()`
4. **Filter Hidden Files**: Skip files starting with '.' unless `-a` flag is set
5. **Sort Entries**: Sort files using locale-aware collation
6. **Display Results**: Choose between simple or long format

## Detailed Functionality

### 1. Argument Parsing (`parse_args`)

**Purpose**: Parse command-line arguments and extract flags and paths.

**Process**:
- Iterates through each argument
- Handles single dash (`-`) as a path
- Parses flag characters: `a`, `l`, `F`
- Returns error for unknown flags
- Collects non-flag arguments as paths

**Example**:
```bash
ls -la /home /tmp
# Flags: show_hidden=true, long_format=true, file_indicators=false
# Paths: ["/home", "/tmp"]
```

### 2. Directory Listing (`list_directory`)

**Purpose**: List contents of a single directory.

**Process**:
1. **Validation**: Check if path exists and is a directory
2. **Read Directory**: Use `fs::read_dir()` to get entries
3. **Error Handling**: Handle permission denied, not found, etc.
4. **Filter Hidden Files**: Skip files starting with '.' unless `-a` flag
5. **Sorting**: Sort files using locale-aware collation
6. **Display**: Choose format based on flags

**Error Handling**:
- `Permission denied`: When directory cannot be read
- `No such file or directory`: When path doesn't exist
- `Not a directory`: When path is not a directory

### 3. Simple Format Display (`print_simple_format`)

**Purpose**: Display files in simple format (default).

**Features**:
- **File Type Indicators** (`-F` flag):
  - `/` for directories
  - `@` for symlinks
  - `|` for FIFOs (named pipes)
  - `=` for sockets
  - `*` for executable files
- **Color Coding**:
  - Bold blue for directories
  - Cyan for symlinks
  - Green for executables
  - Default color for regular files

**Output Format**:
```
file1  file2/  file3@  file4*  file5|
```

### 4. Long Format Display (`print_long_format`)

**Purpose**: Display detailed file information (similar to `ls -l`).

**Information Displayed**:
1. **Permissions**: 10-character permission string
2. **Hard Links**: Number of hard links
3. **Owner**: Username or UID
4. **Group**: Group name or GID
5. **Size**: File size or device numbers
6. **Time**: Modification time
7. **Name**: Filename with indicators and colors
8. **Symlink Target**: For symbolic links

**Output Format**:
```
-rwxr-xr--+ 1 mzakri mzakri 1024 Sep 9 12:00 file.txt
```

### 5. Permission Formatting (`format_permissions`)

**Purpose**: Create Unix-style permission strings.

**Permission String Format**:
```
[type][owner][group][other]
```

#### File Type Characters:
- `-`: Regular file
- `d`: Directory
- `l`: Symbolic link
- `c`: Character device
- `b`: Block device
- `p`: FIFO (named pipe)
- `s`: Socket

#### Permission Bits:
- `r`: Read permission
- `w`: Write permission
- `x`: Execute permission
- `-`: No permission

#### Special Permission Bits:

**Setuid (4000)**:
- `s`: Setuid + executable
- `S`: Setuid + not executable

**Setgid (2000)**:
- `s`: Setgid + executable  
- `S`: Setgid + not executable

**Sticky Bit (1000)**:
- `t`: Sticky + executable
- `T`: Sticky + not executable

**Examples**:
```
-rwxr-xr--   # Regular file, owner rwx, group r-x, other r--
drwxr-xr-x   # Directory, owner rwx, group r-x, other r-x
-rwsr-xr-x   # Setuid executable
-rwxr-sr-x   # Setgid executable
drwxr-xr-t   # Directory with sticky bit
```

### 6. Extended Attributes Detection (`has_extended_attributes`)

**Purpose**: Detect if file has extended attributes or ACLs (shows `+` symbol).

**Detection Methods**:

#### ACL Detection (`getfacl`):
- Runs `getfacl` command on the file
- Checks for extended ACL entries:
  - `user:username:` (not just `user::`)
  - `group:groupname:` (not just `group::`)
  - `mask::` entries
- Standard ACLs only have `user::`, `group::`, `other::`

#### Extended Attributes Detection (`lsattr`):
- Runs `lsattr` command on the file
- Checks for specific extended attribute flags:
  - `i`: Immutable
  - `a`: Append-only
  - `j`: Data journalling
  - `s`: Secure deletion
  - `t`: No tail merging
  - `u`: Undeletable
  - `A`, `S`, `T`, `D`: Other extended attributes
- Ignores standard flags like `e` (extent-based allocation)

**Examples**:
```
-rwxr-xr--   # No extended attributes
-rwxr-xr--+  # Has extended attributes or ACLs
```

### 7. User and Group Resolution

**Purpose**: Convert UID/GID to human-readable names.

#### Owner Resolution (`get_owner_name`):
1. Reads `/etc/passwd` file
2. Parses each line to find matching UID
3. Returns username if found
4. Falls back to UID number if not found

#### Group Resolution (`get_group_name`):
1. Reads `/etc/group` file
2. Parses each line to find matching GID
3. Returns group name if found
4. Falls back to GID number if not found

### 8. Time Formatting (`format_time`)

**Purpose**: Format file modification time for display.

**Logic**:
- **Recent files** (< 6 months): Show time (e.g., "Sep 9 12:00")
- **Old files** (> 6 months): Show year (e.g., "Sep 9  2023")

**Timezone Handling**:
- Reads timezone from `TZ` environment variable
- Falls back to `/etc/timezone` file
- Supports common timezone names (UTC, CET, EST, PST, etc.)
- Defaults to UTC+1 if no timezone found

### 9. Device Number Handling (`major_minor`)

**Purpose**: Extract major and minor device numbers for device files.

**Linux Device Encoding**:
- Major number: bits 8-19 of `rdev`
- Minor number: bits 0-7 and 20-31 of `rdev`

**Display Format**:
```
crw-rw-rw- 1 root root 1, 3 Sep 9 12:00 /dev/null
brw-rw---- 1 root disk 8, 0 Sep 9 12:00 /dev/sda
```

### 10. Color Coding (`get_color`)

**Purpose**: Assign colors to different file types.

**Color Scheme**:
- **Bold Blue** (`\x1b[1;34m`): Directories
- **Cyan** (`\x1b[0;36m`): Symbolic links
- **Green** (`\x1b[0;32m`): Executable files
- **Reset** (`\x1b[0m`): Regular files

### 11. Block Size Calculation (`print_total`)

**Purpose**: Calculate and display total block usage.

**Process**:
1. Sum up `st_blocks` from all files
2. Include `.` and `..` when `-a` flag is set
3. Convert 512-byte blocks to 1K blocks (GNU ls standard)
4. Display as "total X" line

## Error Handling

### Common Error Scenarios:
1. **Permission Denied**: Directory cannot be read
2. **No Such File**: Path doesn't exist
3. **Not a Directory**: Path is not a directory
4. **Invalid Flag**: Unknown command-line option

### Error Display:
- Errors are printed to stderr
- Format: `ls: <path>: <error message>`
- Processing continues with other paths

## Performance Considerations

### Optimizations:
1. **Single Directory Read**: Uses `fs::read_dir()` once per directory
2. **Lazy Metadata**: Only reads metadata when needed
3. **Efficient Sorting**: Uses Rust's built-in sorting algorithms
4. **Minimal System Calls**: Batches operations where possible

### Memory Usage:
- Loads entire directory into memory for sorting
- Metadata is read on-demand
- No caching of user/group names

## Compatibility

### Standards Compliance:
- **POSIX**: Follows POSIX `ls` command specification
- **GNU ls**: Compatible with GNU coreutils `ls` behavior
- **File Permissions**: Matches Unix permission display exactly
- **Time Format**: Uses standard time formatting

### Platform Support:
- **Unix-like Systems**: Linux, macOS, BSD
- **File Systems**: ext4, xfs, zfs, btrfs, etc.
- **Architectures**: x86_64, ARM, etc.

## Examples

### Basic Usage:
```bash
ls                    # List current directory
ls /home              # List specific directory
ls -a                 # Show hidden files
ls -l                 # Long format
ls -la                # Long format with hidden files
ls -F                  # Add file type indicators
ls -laF               # All flags combined
```

### Output Examples:

**Simple Format**:
```
file1  file2/  .hidden  script*
```

**Long Format**:
```
total 8
drwxr-xr-x  2 mzakri mzakri 4096 Sep 9 12:00 .
drwxr-xr-x  3 mzakri mzakri 4096 Sep 9 11:00 ..
-rw-r--r--  1 mzakri mzakri 1024 Sep 9 12:00 file1
-rwxr-xr-x  1 mzakri mzakri 2048 Sep 9 12:00 script*
lrwxrwxrwx  1 mzakri mzakri    5 Sep 9 12:00 link@ -> file1
```

**With Extended Attributes**:
```
-rwxr-xr--+ 1 mzakri mzakri 1024 Sep 9 12:00 file_with_acl
```

**Special Permissions**:
```
-rwsr-xr-x  1 root   mzakri 2048 Sep 9 12:00 setuid_script
-rwxr-sr-x  1 mzakri root   2048 Sep 9 12:00 setgid_script
drwxr-xr-t  2 mzakri mzakri 4096 Sep 9 12:00 sticky_dir
```

## Implementation Notes

### Rust-Specific Features:
- **Error Handling**: Uses `Result<T, E>` for robust error handling
- **Memory Safety**: No unsafe code (except for system calls in extended attributes)
- **Performance**: Leverages Rust's zero-cost abstractions
- **Concurrency**: Single-threaded but efficient

### Dependencies:
- **Standard Library Only**: No external crates required
- **System Commands**: Uses `getfacl` and `lsattr` for extended attributes
- **File System**: Relies on OS file system APIs

### Testing:
- **Unit Tests**: Individual function testing
- **Integration Tests**: Full command testing
- **Cross-Platform**: Tested on multiple Unix systems

This implementation provides a complete, standards-compliant `ls` command with advanced features like extended attributes detection, proper permission handling, and comprehensive error management.
