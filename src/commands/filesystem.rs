use crate::commands::CommandExecutor;
use crate::error::ShellError;
use std::fs;
use std::io;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::FileTypeExt;
use std::time::SystemTime;
use std::os::unix::fs::MetadataExt;
use std::ffi::CStr;
use chrono::{DateTime, Local};

// Standard ls colors
// const RESET: &str = "\x1b[0m";
// const BLUE_BOLD: &str = "\x1b[1;34m";  // Directories (bold blue)
// const GREEN: &str = "\x1b[0;32m";      // Executables
// const CYAN: &str = "\x1b[0;36m";       // Symlinks

pub struct PwdCommand;
pub struct CdCommand;
pub struct LsCommand;
pub struct CatCommand;
pub struct MkdirCommand;
pub struct CpCommand;
pub struct MvCommand;
pub struct RmCommand;

#[derive(Debug)]
struct LsFlags {
    show_hidden: bool,
    long_format: bool,
    file_indicators: bool,
}

impl CommandExecutor for PwdCommand {
    fn execute(&self, _args: &[String]) -> Result<(), ShellError> {
        // TODO: Implement pwd command
        // - Get current working directory
        // - Print to stdout
        todo!("Implement pwd command")
    }

    fn help(&self) -> &str {
        "pwd - Print working directory"
    }
}

impl CommandExecutor for CdCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        // TODO: Implement cd command
        // - Change to specified directory
        // - Handle no arguments (go to home directory)
        // - Handle relative and absolute paths
        todo!("Implement cd command")
    }

    fn help(&self) -> &str {
        "cd [directory] - Change directory"
    }
}

impl CommandExecutor for LsCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        let (flags, paths) = self.parse_args(args)?;
        
        // If no paths specified, use current directory
        let paths = if paths.is_empty() {
            vec![".".to_string()]
        } else {
            paths
        };

        for path in paths {
            if let Err(e) = self.list_directory(&path, &flags) {
                eprintln!("ls: {}: {}", path, e);
            }
        }
        
        Ok(())
    }

    fn help(&self) -> &str {
        "ls [-a] [-l] [-F] [directory...] - List directory contents"
    }
}

impl LsCommand {
    fn print_total(&self, dir: &Path, files: &[fs::DirEntry], flags: &LsFlags) -> Result<(), ShellError> {
        let mut blocks: u64 = 0;
        // Include . and .. when -a
        if flags.show_hidden {
            if let Ok(meta) = fs::symlink_metadata(dir.join(".")) { blocks = blocks.saturating_add(meta.blocks()); }
            if let Ok(meta) = fs::symlink_metadata(dir.join("..")) { blocks = blocks.saturating_add(meta.blocks()); }
        }
        for entry in files {
            if let Ok(meta) = fs::symlink_metadata(entry.path()) {
                blocks = blocks.saturating_add(meta.blocks());
            }
        }
        // st_blocks are 512-byte blocks; GNU ls prints in 1K blocks by default
        let total_k = (blocks + 1) / 2;
        println!("total {}", total_k);
        Ok(())
    }

    fn print_one_long(&self, dir: &Path, name: &str, flags: &LsFlags) -> Result<(), ShellError> {
        let full = dir.join(name);
        let meta = fs::symlink_metadata(&full).map_err(|e| ShellError::FileSystemError(e.to_string()))?;
        let perms = self.format_permissions_with_extended(&meta, &full);
        let nlink = meta.nlink();
        let owner = self.get_owner_name(meta.uid());
        let group = self.get_group_name(meta.gid());
        let ftype = meta.file_type();
        let size_field = if ftype.is_char_device() || ftype.is_block_device() {
            let (maj, min) = self.major_minor(meta.rdev());
            format!("{:>3}, {:>3}", maj, min)
        } else {
            format!("{:>8}", meta.len())
        };
        let time_to_show = meta.modified().unwrap_or_else(|_| SystemTime::now());
        let time_str = self.format_time(time_to_show);
        let mut display_name = name.to_string();
        if flags.file_indicators {
            if ftype.is_dir() { display_name.push('/'); }
            else if ftype.is_symlink() && !flags.long_format { display_name.push('@'); }  // Only show @ if not long format
            else if ftype.is_fifo() { display_name.push('|'); }
            else if ftype.is_socket() { display_name.push('='); }
            else if !ftype.is_symlink() && self.is_executable(&meta) { display_name.push('*'); }  // Don't add * to symlinks in long format
        }
        let link_suffix = if ftype.is_symlink() {
            match fs::read_link(&full) { 
                Ok(t) => {
                    let mut target = t.display().to_string();
                    // Add file type indicator to the target if -F flag is used
                    if flags.file_indicators {
                        if let Ok(target_meta) = fs::metadata(&full) {
                            let target_type = target_meta.file_type();
                            if target_type.is_socket() {
                                target.push('=');
                            } else if target_type.is_dir() {
                                target.push('/');
                            } else if target_type.is_fifo() {
                                target.push('|');
                            } else if self.is_executable(&target_meta) {  // Use self.is_executable() method
                                target.push('*');
                            }
                        }
                    }
                    format!(" -> {}", target)
                }, 
                Err(_) => String::from(" -> (broken)") 
            }
        } else { String::new() };
        println!("{} {:>4} {} {} {} {} {}{}", perms, nlink, owner, group, size_field, time_str, display_name, link_suffix);
        Ok(())
    }
    fn major_minor(&self, rdev: u64) -> (u32, u32) {
        // Linux device encoding per sysmacros.h
        let major = ((rdev >> 8) & 0xfff) as u32;
        let minor = ((rdev & 0xff) | ((rdev >> 12) & 0xfffff00)) as u32;
        (major, minor)
    }
    fn get_owner_name(&self, uid: u32) -> String {
        // Try using getpwuid system call
        unsafe {
            let passwd = libc::getpwuid(uid as libc::uid_t);
            if !passwd.is_null() {
                let name = CStr::from_ptr((*passwd).pw_name).to_string_lossy();
                return name.to_string();
            }
        }
        
        // Fallback: Try reading /etc/passwd directly
        if let Ok(content) = std::fs::read_to_string("/etc/passwd") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 {
                    if let Ok(line_uid) = parts[2].parse::<u32>() {
                        if line_uid == uid {
                            return parts[0].to_string();
                        }
                    }
                }
            }
        }
        
        // Final fallback to UID as string
        uid.to_string()
    }

    fn get_group_name(&self, gid: u32) -> String {
        // Try using getgrgid system call
        unsafe {
            let group = libc::getgrgid(gid as libc::gid_t);
            if !group.is_null() {
                let name = CStr::from_ptr((*group).gr_name).to_string_lossy();
                return name.to_string();
            }
        }
        
        // Fallback: Try reading /etc/group directly
        if let Ok(content) = std::fs::read_to_string("/etc/group") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 {
                    if let Ok(line_gid) = parts[2].parse::<u32>() {
                        if line_gid == gid {
                            return parts[0].to_string();
                        }
                    }
                }
            }
        }
        
        // Final fallback to GID as string
        gid.to_string()
    }

    fn parse_args(&self, args: &[String]) -> Result<(LsFlags, Vec<String>), ShellError> {
        let mut flags = LsFlags {
            show_hidden: false,
            long_format: false,
            file_indicators: false,
        };

        let mut is_option = true;
        let mut paths = Vec::new();

        for arg in args {
            if arg.starts_with('-') && is_option {
                if arg == "-" {
                    // Single dash is treated as a path
                    paths.push(arg.clone());
                } else if arg == "--" {
                    is_option = false;
                    continue; // Skip to next argument
                } else {
                    // Parse flags
                    for c in arg[1..].chars() {
                        match c {
                            'a' => flags.show_hidden = true,
                            'l' => flags.long_format = true,
                            'F' => flags.file_indicators = true,
                            _ => {
                                return Err(ShellError::ExecutionError(
                                    format!("ls: invalid option -- '{}'", c)
                                ));
                            }
                        }
                    }
                }
            } else {
                paths.push(arg.clone());
            }
        }

        Ok((flags, paths))
    }
    

    fn list_directory(&self, path_str: &str, flags: &LsFlags) -> Result<(), ShellError> {
        let path = Path::new(path_str);
        
        // Check if path exists
        if !path.exists() {
            return Err(ShellError::FileSystemError("No such file or directory".to_string()));
        }

        // Handle files vs directories
        if path.is_file() {
            return self.list_file(path, flags);
        } else if path.is_dir() {
            return self.list_directory_contents(path, flags);
        } else {
            // Handle other file types (symlinks, etc.)
            return self.list_file(path, flags);
        }
    }

    fn list_file(&self, path: &Path, flags: &LsFlags) -> Result<(), ShellError> {
        // Use the full path instead of just the filename
        let name = path.to_string_lossy().to_string();

        if flags.long_format {
            self.print_one_long(path.parent().unwrap_or(Path::new(".")), &name, flags)?;
        } else {
            let mut display_name = name.clone();
            
            if let Ok(metadata) = fs::symlink_metadata(path) {
                if flags.file_indicators {
                    let ftype = metadata.file_type();
                    if ftype.is_dir() {
                        display_name.push('/');
                    } else if ftype.is_symlink() {
                        // For symlinks, only show @ indicator in short format, not long format
                        if !flags.long_format {
                            display_name.push('@');
                        }
                        // In long format, indicators will be added to the target in the link_suffix
                    } else if ftype.is_fifo() {
                        display_name.push('|');
                    } else if ftype.is_socket() {
                        display_name.push('=');
                    } else if self.is_executable(&metadata) {
                        display_name.push('*');
                    }
                }
                
                println!("{display_name}");
            } else {
                println!("{display_name}");
            }
        }
        
        Ok(())
    }

    fn list_directory_contents(&self, path: &Path, flags: &LsFlags) -> Result<(), ShellError> {
        let entries = fs::read_dir(path)
            .map_err(|e| {
                match e.kind() {
                    io::ErrorKind::PermissionDenied => {
                        ShellError::FileSystemError("Permission denied".to_string())
                    }
                    io::ErrorKind::NotFound => {
                        ShellError::FileSystemError("No such file or directory".to_string())
                    }
                    _ => ShellError::FileSystemError(format!("Cannot read directory: {}", e))
                }
            })?;

        let mut files = Vec::new();
        for entry in entries {
            match entry {
                Ok(entry) => {
                    let name = entry.file_name().to_string_lossy().to_string();
                    
                    if !flags.show_hidden && name.starts_with('.') {
                        continue;
                    }
                    
                    files.push(entry);
                }
                Err(e) => {
                    eprintln!("ls: cannot access '{}': {}", path.display(), e);
                }
            }
        }

        files.sort_by(|a, b| {
            let name_a = a.file_name().to_string_lossy().to_string().replace(".", "").to_lowercase();
            let name_b = b.file_name().to_string_lossy().to_string().replace(".", "").to_lowercase();
            name_a.cmp(&name_b)
        });

        // Print files
        if flags.long_format {
            // Print total line first
            self.print_total(path, &files, flags)?;
            // Include . and .. before listing entries when showing hidden
            if flags.show_hidden {
                self.print_one_long(path, ".", flags)?;
                self.print_one_long(path, "..", flags)?;
            }
            self.print_long_format(&files, flags)?;
        } else {
            self.print_simple_format(&files, flags)?;
        }

        Ok(())
    }

    fn print_simple_format(&self, files: &[fs::DirEntry], flags: &LsFlags) -> Result<(), ShellError> {
        for entry in files {
            let name = entry.file_name().to_string_lossy().to_string();
            let mut display_name = name.clone();
            
            if let Ok(metadata) = fs::symlink_metadata(entry.path()) {
                if flags.file_indicators {
                    let ftype = metadata.file_type();
                    if ftype.is_dir() {
                        display_name.push('/');
                    } else if ftype.is_symlink() {
                        // For symlinks, only show @ indicator in short format, not long format
                        if !flags.long_format {
                            display_name.push('@');
                        }
                        // In long format, indicators will be added to the target in the link_suffix
                    } else if ftype.is_fifo() {
                        display_name.push('|');
                    } else if ftype.is_socket() {
                        display_name.push('=');
                    } else if self.is_executable(&metadata) {
                        display_name.push('*');
                    }
                }
                
                print!("{}  ", display_name);
            } else {
                print!("{}  ", display_name);
            }
        }
        println!();
        Ok(())
    }

    fn print_long_format(&self, files: &[fs::DirEntry], flags: &LsFlags) -> Result<(), ShellError> {
        for entry in files {
            let name = entry.file_name().to_string_lossy().to_string();
            let mut display_name = name.clone();
            
            if let Ok(metadata) = entry.metadata() {
                // Permissions
                let perms = self.format_permissions_with_extended(&metadata, &entry.path());
                
                // Hard links count
                let nlink = metadata.nlink();
                
                // Get actual owner and group IDs
                let uid = metadata.uid();
                let gid = metadata.gid();
                let owner = self.get_owner_name(uid);
                let group = self.get_group_name(gid);
                
                // Size or device numbers
                let ftype = metadata.file_type();
                let size_field = if ftype.is_char_device() || ftype.is_block_device() {
                    let (maj, min) = self.major_minor(metadata.rdev());
                    format!("{:>3}, {:>3}", maj, min)
                } else {
                    format!("{:>8}", metadata.len())
                };
                
                // Get the most appropriate time to display
                // Standard ls shows modification time
                let time_to_show = metadata.modified()
                    .unwrap_or_else(|_| std::time::SystemTime::now());
                let time_str = self.format_time(time_to_show);
                
                // File type indicator
                if flags.file_indicators {
                    let ftype = metadata.file_type();
                    if ftype.is_dir() {
                        display_name.push('/');
                    } else if ftype.is_symlink() {
                        // For symlinks, don't add any indicator to the symlink name itself
                        // Indicators will be added to the target in the link_suffix
                    } else if ftype.is_fifo() {
                        display_name.push('|');
                    } else if ftype.is_socket() {
                        display_name.push('=');
                    } else if self.is_executable(&metadata) {
                        display_name.push('*');
                    }
                }
                
                // If symlink, append " -> target" like ls -l
                let link_suffix = if ftype.is_symlink() {
                    match std::fs::read_link(entry.path()) {
                        Ok(target) => {
                            let mut target_str = target.display().to_string();
                            // Add file type indicator to the target if -F flag is used
                            if flags.file_indicators {
                                if let Ok(target_meta) = fs::metadata(entry.path()) {
                                    let target_type = target_meta.file_type();
                                    if target_type.is_socket() {
                                        target_str.push('=');
                                    } else if target_type.is_dir() {
                                        target_str.push('/');
                                    } else if target_type.is_fifo() {
                                        target_str.push('|');
                                    } else if self.is_executable(&target_meta) {
                                        target_str.push('*');
                                    }
                                }
                            }
                            format!(" -> {}", target_str)
                        },
                        Err(_) => String::from(" -> (broken)"),
                    }
                } else {
                    String::new()
                };

                println!("{} {:>4} {} {} {} {} {}{}", 
                    perms, nlink, owner, group, size_field, time_str, display_name, link_suffix);
            } else {
                println!("{}", display_name);
            }
        }
        Ok(())
    }

    fn format_permissions(&self, metadata: &fs::Metadata) -> String {
        let mode = metadata.permissions().mode();
        let mut perms = String::new();
        
        let ftype = metadata.file_type();
        if ftype.is_dir() {
            perms.push('d');
        } else if ftype.is_symlink() {
            perms.push('l');
        } else if ftype.is_char_device() {
            perms.push('c');
        } else if ftype.is_block_device() {
            perms.push('b');
        } else if ftype.is_fifo() {
            perms.push('p');
        } else if ftype.is_socket() {
            perms.push('s');
        } else {
            perms.push('-');
        }
        
        // Owner permissions
        perms.push(if mode & 0o400 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o200 != 0 { 'w' } else { '-' });
        
        // Handle setuid bit (4000) - if set and executable, show 's', if set and not executable, show 'S'
        if mode & 0o4000 != 0 {
            perms.push(if mode & 0o100 != 0 { 's' } else { 'S' });
        } else {
            perms.push(if mode & 0o100 != 0 { 'x' } else { '-' });
        }
        
        // Group permissions
        perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
        
        // Handle setgid bit (2000) - if set and executable, show 's', if set and not executable, show 'S'
        if mode & 0o2000 != 0 {
            perms.push(if mode & 0o010 != 0 { 's' } else { 'S' });
        } else {
            perms.push(if mode & 0o010 != 0 { 'x' } else { '-' });
        }
        
        // Other permissions
        perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
        
        // Handle sticky bit (1000) - if set and executable, show 't', if set and not executable, show 'T'
        if mode & 0o1000 != 0 {
            perms.push(if mode & 0o001 != 0 { 't' } else { 'T' });
        } else {
            perms.push(if mode & 0o001 != 0 { 'x' } else { '-' });
        }
        
        perms
    }

    fn format_permissions_with_extended(&self, metadata: &fs::Metadata, path: &Path) -> String {
        let mut perms = self.format_permissions(metadata);
        
        // Check for extended attributes or ACLs
        if self.has_extended_attributes(path) {
            perms.push('+');
        }
        
        perms
    }

    fn has_extended_attributes(&self, path: &Path) -> bool {
        // Use libc system calls to detect extended attributes
        // Convert path to C string
        let path_cstr = match std::ffi::CString::new(path.to_string_lossy().as_bytes()) {
            Ok(s) => s,
            Err(_) => return false,
        };
        
        unsafe {
            let mut buffer = [0u8; 1024];
            let result = libc::listxattr(
                path_cstr.as_ptr(),
                buffer.as_mut_ptr() as *mut i8,
                buffer.len(),
            );
            
            if result > 0 {
                let attr_names = std::str::from_utf8(&buffer[..result as usize]).unwrap_or("");
                for name in attr_names.split('\0') {
                    if !name.is_empty() && self.is_extended_attribute(name) {
                        return true;
                    }
                }
            }
        }
        
        false
    }
    
    fn is_extended_attribute(&self, name: &str) -> bool {
        name.starts_with("system.posix_acl_") ||
        name.starts_with("security.") ||
        name.starts_with("trusted.") ||
        name.starts_with("user.") ||
        name.starts_with("system.") ||
        name.starts_with("xattr.")
    }

    fn format_time(&self, time: SystemTime) -> String {
        let now = SystemTime::now();
        let duration = now.duration_since(time).unwrap_or_default();
        
        let six_months = std::time::Duration::from_secs(6 * 30 * 24 * 60 * 60);
        
        
        let datetime: DateTime<Local> = time.into();
        
        if duration > six_months {
            datetime.format("%b %d  %Y").to_string()
        } else {

            datetime.format("%b %d %H:%M").to_string()
        }
    }

    fn is_executable(&self, metadata: &fs::Metadata) -> bool {
        let mode = metadata.permissions().mode();
        mode & 0o111 != 0
    }
}



impl CommandExecutor for CatCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        // TODO: Implement cat command
        // - Read and display file contents
        // - Handle multiple files
        // - Handle missing files gracefully
        todo!("Implement cat command")
    }

    fn help(&self) -> &str {
        "cat [file...] - Concatenate and display files"
    }
}

impl CommandExecutor for MkdirCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        // TODO: Implement mkdir command
        // - Create directories
        // - Handle multiple directories
        // - Handle existing directories gracefully
        todo!("Implement mkdir command")
    }

    fn help(&self) -> &str {
        "mkdir [directory...] - Create directories"
    }
}

impl CommandExecutor for CpCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        // TODO: Implement cp command
        // - Copy files and directories
        // - Handle file to file copying
        // - Handle file to directory copying
        todo!("Implement cp command")
    }

    fn help(&self) -> &str {
        "cp source destination - Copy files and directories"
    }
}

impl CommandExecutor for MvCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        // TODO: Implement mv command
        // - Move/rename files and directories
        // - Handle file to file moving
        // - Handle file to directory moving
        todo!("Implement mv command")
    }

    fn help(&self) -> &str {
        "mv source destination - Move (rename) files"
    }
}

impl CommandExecutor for RmCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        // TODO: Implement rm command with -r flag
        // - Remove files
        // - Handle -r flag for recursive directory removal
        // - Handle missing files gracefully
        todo!("Implement rm command")
    }

    fn help(&self) -> &str {
        "rm [-r] [file...] - Remove files or directories"
    }
}
