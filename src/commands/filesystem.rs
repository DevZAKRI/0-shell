use crate::commands::CommandExecutor;
use crate::error::ShellError;
use std::fs;
use std::io;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::FileTypeExt;
use std::time::SystemTime;
use std::os::unix::fs::MetadataExt;
use std::process::Command;
use std::ffi::CStr;

// Standard ls colors
const RESET: &str = "\x1b[0m";
const BLUE_BOLD: &str = "\x1b[1;34m";  // Directories (bold blue)
const GREEN: &str = "\x1b[0;32m";      // Executables
const CYAN: &str = "\x1b[0;36m";       // Symlinks

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
        let color = self.get_color(&meta);
        let mut display_name = name.to_string();
        if flags.file_indicators {
            if ftype.is_dir() { display_name.push('/'); }
            else if ftype.is_symlink() { display_name.push('@'); }
            else if ftype.is_fifo() { display_name.push('|'); }
            else if ftype.is_socket() { display_name.push('='); }
            else if self.is_executable(&meta) { display_name.push('*'); }
        }
        let link_suffix = if ftype.is_symlink() {
            match fs::read_link(&full) { Ok(t) => format!(" -> {}", t.display()), Err(_) => String::from(" -> (broken)") }
        } else { String::new() };
        println!("{} {:>4} {} {} {} {} {}{}{}{}", perms, nlink, owner, group, size_field, time_str, color, display_name, RESET, link_suffix);
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
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

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
                        display_name.push('@');
                    } else if ftype.is_fifo() {
                        display_name.push('|');
                    } else if ftype.is_socket() {
                        display_name.push('=');
                    } else if self.is_executable(&metadata) {
                        display_name.push('*');
                    }
                }
                
                let color = self.get_color(&metadata);
                println!("{}{}{}", color, display_name, RESET);
            } else {
                println!("{}", display_name);
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
            
            if let Ok(metadata) = entry.metadata() {
                if flags.file_indicators {
                    let ftype = metadata.file_type();
                    if ftype.is_dir() {
                        display_name.push('/');
                    } else if ftype.is_symlink() {
                        display_name.push('@');
                    } else if ftype.is_fifo() {
                        display_name.push('|');
                    } else if ftype.is_socket() {
                        display_name.push('=');
                    } else if self.is_executable(&metadata) {
                        display_name.push('*');
                    }
                }
                
                let color = self.get_color(&metadata);
                print!("{}{}{}  ", color, display_name, RESET);
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
                        display_name.push('@');
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
                        Ok(target) => format!(" -> {}", target.display()),
                        Err(_) => String::from(" -> (broken)"),
                    }
                } else {
                    String::new()
                };

                let color = self.get_color(&metadata);
                println!("{} {:>4} {} {} {} {} {}{}{}{}", 
                    perms, nlink, owner, group, size_field, time_str, color, display_name, RESET, link_suffix);
            } else {
                println!("{}", display_name);
            }
        }
        Ok(())
    }

    fn get_color(&self, metadata: &fs::Metadata) -> &'static str {
        if metadata.is_dir() {
            BLUE_BOLD
        } else if metadata.is_symlink() {
            CYAN
        } else if self.is_executable(metadata) {
            GREEN
        } else {
            RESET
        }
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
        // Try to detect extended attributes using system commands
        // First try getfacl (for ACLs)
        if let Ok(output) = Command::new("getfacl")
            .arg(path)
            .output()
        {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                // Check if there are any user-specific or group-specific ACL entries
                // Standard ACLs only have user::, group::, and other:: entries
                // Extended ACLs have additional user:username: or group:groupname: entries
                for line in output_str.lines() {
                    if line.starts_with("user:") && !line.starts_with("user::") {
                        return true; // Found user-specific ACL
                    }
                    if line.starts_with("group:") && !line.starts_with("group::") {
                        return true; // Found group-specific ACL
                    }
                    if line.starts_with("mask::") {
                        return true; // Found ACL mask
                    }
                }
            }
        }
        
        // Try lsattr to check for extended attributes (Linux-specific)
        if let Ok(output) = Command::new("lsattr")
            .arg(path)
            .output()
        {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = output_str.lines().next() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let flags = parts[0];
                        // Only check for specific extended attribute flags
                        // Standard flags like 'e' (extent-based allocation) are not extended attributes
                        // Extended attributes are typically things like 'i' (immutable), 'a' (append-only), etc.
                        // But we need to be careful - only some flags indicate extended attributes
                        let extended_flags = ['i', 'a', 'j', 's', 't', 'u', 'A', 'S', 'T', 'D'];
                        if flags.chars().any(|c| extended_flags.contains(&c)) {
                            return true;
                        }
                    }
                }
            }
        }
        
        // No extended attributes detected
        false
    }

    fn format_time(&self, time: SystemTime) -> String {
        let now = SystemTime::now();
        let duration = now.duration_since(time).unwrap_or_default();
        
        // If file is older than 6 months, show year instead of time
        let six_months = std::time::Duration::from_secs(6 * 30 * 24 * 60 * 60);
        
        // Convert SystemTime to timestamp for formatting
        let timestamp = time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
        let secs = timestamp.as_secs();
        
        // Simple time formatting using standard library
        if duration > six_months {
            // Show year for old files: "Jan 01  2024"
            self.format_date_with_year(secs)
        } else {
            // Show time for recent files: "Jan 01 12:00"
            self.format_date_with_time(secs)
        }
    }

    fn format_date_with_year(&self, secs: u64) -> String {
        let (month, day, year) = self.seconds_to_date(secs);
        let month_name = self.month_to_name(month);
        format!("{} {:2}  {}", month_name, day, year)
    }

    fn format_date_with_time(&self, secs: u64) -> String {
        // Get timezone offset and apply it
        let timezone_offset = self.get_timezone_offset();
        let local_secs = (secs as i64 + timezone_offset) as u64;
        
        // Use local seconds for date calculation too
        let (month, day, _) = self.seconds_to_date(local_secs);
        let month_name = self.month_to_name(month);
        
        let hours = (local_secs % (24 * 60 * 60)) / (60 * 60);
        let minutes = (local_secs % (60 * 60)) / 60;
        
        format!("{} {:2} {:02}:{:02}", month_name, day, hours, minutes)
    }

    fn seconds_to_date(&self, secs: u64) -> (u32, u32, u32) {
        let mut days = secs / (24 * 60 * 60);
        let mut year = 1970;
        
        // Account for leap years
        while days >= self.days_in_year(year) {
            days -= self.days_in_year(year);
            year += 1;
        }
        
        let mut month = 1;
        let mut day = days as u32;
        
        // Calculate month and day
        while day >= self.days_in_month(year, month) {
            day -= self.days_in_month(year, month);
            month += 1;
        }
        
        (month, day + 1, year) // +1 because day 0 is January 1st
    }

    fn days_in_year(&self, year: u32) -> u64 {
        if self.is_leap_year(year) { 366 } else { 365 }
    }

    fn is_leap_year(&self, year: u32) -> bool {
        (year % 4 == 0) && ((year % 100 != 0) || (year % 400 == 0))
    }

    fn days_in_month(&self, year: u32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => if self.is_leap_year(year) { 29 } else { 28 },
            _ => 30,
        }
    }

    fn month_to_name(&self, month: u32) -> &'static str {
        match month {
            1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
            5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
            9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
            _ => "Jan",
        }
    }

    fn is_executable(&self, metadata: &fs::Metadata) -> bool {
        let mode = metadata.permissions().mode();
        mode & 0o111 != 0
    }

    // Timezone detection methods
    fn get_timezone_offset(&self) -> i64 {
        // Method 1: Try to read from environment
        if let Ok(tz) = std::env::var("TZ") {
            if let Some(offset) = self.parse_tz_env(&tz) {
                return offset;
            }
        }
        
        // Method 2: Try to read from /etc/timezone
        if let Ok(content) = std::fs::read_to_string("/etc/timezone") {
            if let Some(offset) = self.parse_timezone_name(&content.trim()) {
                return offset;
            }
        }
        
        // Default fallback (adjust based on your location)
        3600 // UTC+1
    }

    fn parse_tz_env(&self, tz: &str) -> Option<i64> {
        // Parse TZ format like "UTC+1" or "CET-1"
        if tz.starts_with("UTC") {
            if let Some(offset_str) = tz.strip_prefix("UTC") {
                if offset_str.is_empty() {
                    return Some(0);
                }
                if let Ok(offset) = offset_str.parse::<i64>() {
                    return Some(offset * 3600);
                }
            }
        }
        None
    }

    fn parse_timezone_name(&self, tz_name: &str) -> Option<i64> {
        // Common timezone offsets
        match tz_name {
            "UTC" => Some(0),
            "GMT" => Some(0),
            "CET" => Some(3600), // Central European Time (UTC+1)
            "CEST" => Some(7200), // Central European Summer Time (UTC+2)
            "EET" => Some(7200), // Eastern European Time (UTC+2)
            "EEST" => Some(10800), // Eastern European Summer Time (UTC+3)
            "EST" => Some(-18000), // Eastern Standard Time (UTC-5)
            "EDT" => Some(-14400), // Eastern Daylight Time (UTC-4)
            "PST" => Some(-28800), // Pacific Standard Time (UTC-8)
            "PDT" => Some(-25200), // Pacific Daylight Time (UTC-7)
            _ => None,
        }
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
