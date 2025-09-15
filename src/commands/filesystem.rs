use crate::commands::CommandExecutor;
use crate::error::ShellError;
use std::fs;
use std::io::{ self, Read, Write };
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::FileTypeExt;
use std::time::SystemTime;
use std::os::unix::fs::MetadataExt;
use std::ffi::CStr;
use std::env;
use chrono::{ DateTime, Local };

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

pub struct CommandOptions {
    is_option: bool,
}

impl CommandExecutor for PwdCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        let mut is_option = true;

        for arg in args {
            if arg == "--" {
                is_option = false;
                continue;
            }
            if arg.starts_with('-') && is_option {
                return Err(ShellError::InvalidOption(arg.clone()));
            }
        }

        println!("{}", env::current_dir()?.display());
        Ok(())
    }

    fn help(&self) -> &str {
        "Usage: pwd - Print working directory"
    }
}

impl CommandExecutor for CdCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        let mut is_option = true;
        let mut target_dir = String::new();

        for arg in args {
            if arg == "--" {
                is_option = false;
                continue;
            }
            if arg.starts_with('-') && is_option && arg != "-" {
                return Err(ShellError::InvalidOption(arg.clone()));
            }
            if target_dir.is_empty() {
                target_dir = arg.clone();
            }
        }

        let current_dir = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string());

        let target_dir = self.resolve_target_directory(&target_dir)?;

        match std::env::set_current_dir(&target_dir) {
            Ok(()) => {
                unsafe {
                    std::env::set_var("OLDPWD", current_dir);
                }
                Ok(())
            }
            Err(e) => {
                match e.kind() {
                    io::ErrorKind::NotFound => {
                        Err(ShellError::FileSystemError(
                            format!("cd: {}: No such file or directory", target_dir)
                        ))
                    }
                    io::ErrorKind::PermissionDenied => {
                        Err(ShellError::FileSystemError(
                            format!("cd: {}: Permission denied", target_dir)
                        ))
                    }
                    io::ErrorKind::NotADirectory => {
                        Err(ShellError::FileSystemError(
                            format!("cd: {}: Not a directory", target_dir)
                        ))
                    }
                    _ => {
                        Err(ShellError::FileSystemError(
                            format!("cd: {}: {}", target_dir, e)
                        ))
                    }
                }
            }
        }
    }

    fn help(&self) -> &str {
        "Usage: cd [directory] - Change directory to the specified directory"
    }
}

impl CdCommand {
    fn resolve_target_directory(&self, target: &str) -> Result<String, ShellError> {
        if target.is_empty() {
            return Ok(self.get_home_directory()?);
        }

        if target.starts_with("~/") {
            let home = self.get_home_directory()?;
            let path = target.strip_prefix("~/").unwrap();
            return Ok(format!("{}/{}", home, path));
        }

        if target == "-" {
            return self.get_oldpwd_directory();
        }

        Ok(target.to_string())
    }

    fn get_home_directory(&self) -> Result<String, ShellError> {
        std::env::var("HOME")
            .map_err(|_| ShellError::FileSystemError(
                "cd: HOME environment variable not set".to_string()
            ))
    }

    fn get_oldpwd_directory(&self) -> Result<String, ShellError> {
        let oldpwd = std::env::var("OLDPWD")
            .map_err(|_| ShellError::FileSystemError(
                "cd: OLDPWD not set".to_string()
            ))?;
        
        println!("{}", oldpwd);
        Ok(oldpwd)
    }
}

impl CommandExecutor for LsCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        for _arg in args {}
        let (flags, paths) = self.parse_args(args)?;
        
        
        let paths = if paths.is_empty() {
            vec![".".to_string()]
        } else {
            paths
        };

        let mut files = Vec::new();
        let mut directories = Vec::new();
        let mut missing = Vec::new();
        
        for path in &paths {
            let path_obj = Path::new(path);
            if path_obj.is_file() {
                files.push(path.clone());
            } else if path_obj.is_dir() {
                directories.push(path.clone());
            } else {
                if !path_obj.exists() {
                    missing.push(path.clone());
                } else {
                    files.push(path.clone());
                }
            }
        }
        
        for m in &missing {
            eprintln!("ls: {}: No such file or directory", m);
        }
        
        if !files.is_empty() {
            if flags.long_format {
                for file in &files {
                    if let Err(e) = self.list_file(Path::new(file), &flags) {
                        eprintln!("ls: {}: {}", file, e);
                    }
                }
            } else {
                if let Err(e) = self.list_files_together(&files, &flags) {
                    eprintln!("ls: error listing files: {}", e);
                }
            }
        }
        
        for (i, dir) in directories.iter().enumerate() {
            if !files.is_empty() || i > 0 {
                println!();
            }
            if !files.is_empty() || directories.len() > 1 {
                println!("{}:", dir);
            }
            if let Err(e) = self.list_directory_contents(Path::new(dir), &flags) {
                eprintln!("ls: {}: {}", dir, e);
            }
        }

        Ok(())
    }

    fn help(&self) -> &str {
        "Usage: ls [-a] [-l] [-F] [directory...] - List directory contents"
    }
}

impl LsCommand {
    fn print_total(
        &self,
        dir: &Path,
        files: &[fs::DirEntry],
        flags: &LsFlags
    ) -> Result<(), ShellError> {
        let mut blocks: u64 = 0;
        if flags.show_hidden {
            if let Ok(meta) = fs::symlink_metadata(dir.join(".")) {
                blocks = blocks.saturating_add(meta.blocks());
            }
            if let Ok(meta) = fs::symlink_metadata(dir.join("..")) {
                blocks = blocks.saturating_add(meta.blocks());
            }
        }
        for entry in files {
            if let Ok(meta) = fs::symlink_metadata(entry.path()) {
                blocks = blocks.saturating_add(meta.blocks());
            }
        }
        let total_k = (blocks + 1) / 2;
        println!("total {}", total_k);
        Ok(())
    }

    fn print_one_long(&self, dir: &Path, name: &str, flags: &LsFlags) -> Result<(), ShellError> {
        let full = dir.join(name);
        let meta = fs
            ::symlink_metadata(&full)
            .map_err(|e| ShellError::FileSystemError(e.to_string()))?;
        let perms = self.format_permissions_with_extended(&meta, &full);
        let nlink = meta.nlink();
        let owner = self.get_owner_name(meta.uid());
        let group = self.get_group_name(meta.gid());
        let ftype = meta.file_type();
        let size_field = if ftype.is_char_device() || ftype.is_block_device() {
            let (maj, min) = self.major_minor(meta.rdev());
            format!("{:>3}, {:>5}", maj, min)
        } else {
            format!("{:>8}", meta.len())
        };
        let time_to_show = meta.modified().unwrap_or_else(|_| SystemTime::now());
        let time_str = self.format_time(time_to_show);
        let mut display_name = name.to_string();
        if flags.file_indicators {
            if ftype.is_dir() {
                display_name.push('/');
            } else if ftype.is_symlink() && !flags.long_format {
                display_name.push('@');
            } else if
                ftype.is_fifo()
            {
                display_name.push('|');
            } else if ftype.is_socket() {
                display_name.push('=');
            } else if !ftype.is_symlink() && self.is_executable(&meta) {
                display_name.push('*');
            }
        }
        let link_suffix = if ftype.is_symlink() {
            match fs::read_link(&full) {
                Ok(t) => {
                    let mut target = t.display().to_string();
                    if flags.file_indicators {
                        if let Ok(target_meta) = fs::metadata(&full) {
                            let target_type = target_meta.file_type();
                            if target_type.is_socket() {
                                target.push('=');
                            } else if target_type.is_dir() {
                                target.push('/');
                            } else if target_type.is_fifo() {
                                target.push('|');
                            } else if self.is_executable(&target_meta) {
                                target.push('*');
                            }
                        }
                    }
                    format!(" -> {}", target)
                }
                Err(_) => String::from(" -> (broken)"),
            }
        } else {
            String::new()
        };

        let perms_width = perms.len().max(10);
        let nlink_width = nlink.to_string().len().max(1);
        let owner_width = owner.len().max(1);
        let group_width = group.len().max(1);
        let size_width = size_field.len().max(1);

        println!(
            "{:<perms_width$} {:>nlink_width$} {:<owner_width$} {:<group_width$} {:>size_width$} {} {}{}",
            perms,
            nlink,
            owner,
            group,
            size_field,
            time_str,
            display_name,
            link_suffix,
            perms_width = perms_width,
            nlink_width = nlink_width,
            owner_width = owner_width,
            group_width = group_width,
            size_width = size_width
        );
        Ok(())
    }
    fn major_minor(&self, rdev: u64) -> (u32, u32) {
        let major = ((rdev >> 8) & 0xfff) as u32;
        let minor = ((rdev & 0xff) | ((rdev >> 12) & 0xfffff00)) as u32;
        (major, minor)
    }
    fn get_owner_name(&self, uid: u32) -> String {
        unsafe {
            let passwd = libc::getpwuid(uid as libc::uid_t);
            if !passwd.is_null() {
                let name = CStr::from_ptr((*passwd).pw_name).to_string_lossy();
                return name.to_string();
            }
        }

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

        uid.to_string()
    }

    fn get_group_name(&self, gid: u32) -> String {
        unsafe {
            let group = libc::getgrgid(gid as libc::gid_t);
            if !group.is_null() {
                let name = CStr::from_ptr((*group).gr_name).to_string_lossy();
                return name.to_string();
            }
        }

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
                    paths.push(arg.clone());
                } else if arg == "--" {
                    is_option = false;
                    continue;
                } else {
                    for c in arg[1..].chars() {
                        match c {
                            'a' => {
                                flags.show_hidden = true;
                            }
                            'l' => {
                                flags.long_format = true;
                            }
                            'F' => {
                                flags.file_indicators = true;
                            }
                            _ => {
                                return Err(
                                    ShellError::ExecutionError(
                                        format!("ls: invalid option -- '{}'", c)
                                    )
                                );
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

        if !path.exists() {
            return Err(ShellError::FileSystemError("No such file or directory".to_string()));
        }

        if path.is_file() {
            return self.list_file(path, flags);
        } else if path.is_dir() {
            return self.list_directory_contents(path, flags);
        } else {
            return self.list_file(path, flags);
        }
    }

    fn list_file(&self, path: &Path, flags: &LsFlags) -> Result<(), ShellError> {
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
                        if !flags.long_format {
                            display_name.push('@');
                        }
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

    fn list_files_together(&self, files: &[String], flags: &LsFlags) -> Result<(), ShellError> {
        for file in files {
            let path = Path::new(file);
            let name = path.to_string_lossy().to_string();
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
            }
            
            print!("{}  ", display_name);
        }
        println!();
        Ok(())
    }

    fn list_directory_contents(&self, path: &Path, flags: &LsFlags) -> Result<(), ShellError> {
        let entries = fs::read_dir(path).map_err(|e| {
            match e.kind() {
                io::ErrorKind::PermissionDenied => {
                    ShellError::FileSystemError("Permission denied".to_string())
                }
                io::ErrorKind::NotFound => {
                    ShellError::FileSystemError("No such file or directory".to_string())
                }
                _ => ShellError::FileSystemError(format!("Cannot read directory: {}", e)),
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
            let name_a = a
                .file_name()
                .to_string_lossy()
                .to_string()
                .to_lowercase()
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .collect::<String>();
            let name_b = b
                .file_name()
                .to_string_lossy()
                .to_string()
                .to_lowercase()
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .collect::<String>();
            name_a.cmp(&name_b)
        });

        if flags.long_format {
            self.print_total(path, &files, flags)?;
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

    fn print_simple_format(
        &self,
        files: &[fs::DirEntry],
        flags: &LsFlags
    ) -> Result<(), ShellError> {
        for entry in files {
            let name = entry.file_name().to_string_lossy().to_string();
            let mut display_name = name.clone();

            if let Ok(metadata) = fs::symlink_metadata(entry.path()) {
                if flags.file_indicators {
                    let ftype = metadata.file_type();
                    if ftype.is_dir() {
                        display_name.push('/');
                    } else if ftype.is_symlink() {
                        if !flags.long_format {
                            display_name.push('@');
                        }
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
        if files.len() > 0 {
            println!();
        }
        Ok(())
    }

    fn print_long_format(&self, files: &[fs::DirEntry], flags: &LsFlags) -> Result<(), ShellError> {
        if files.is_empty() {
            return Ok(());
        }

        let mut file_data = Vec::new();
        let mut max_nlink_width = 0;
        let mut max_owner_width = 0;
        let mut max_group_width = 0;
        let mut max_size_width = 0;
        let mut max_perms_width = 0;

        for entry in files {
            let name = entry.file_name().to_string_lossy().to_string();
            let mut display_name = name.clone();

            if let Ok(metadata) = entry.metadata() {
                let perms = self.format_permissions_with_extended(&metadata, &entry.path());

                let nlink = metadata.nlink();

                let uid = metadata.uid();
                let gid = metadata.gid();
                let owner = self.get_owner_name(uid);
                let group = self.get_group_name(gid);

                let ftype = metadata.file_type();
                let size_field = if ftype.is_char_device() || ftype.is_block_device() {
                    let (maj, min) = self.major_minor(metadata.rdev());
                    format!("{:>3}, {:>5}", maj, min)
                } else {
                    format!("{:>8}", metadata.len())
                };

                let time_to_show = metadata
                    .modified()
                    .unwrap_or_else(|_| std::time::SystemTime::now());
                let time_str = self.format_time(time_to_show);

                if flags.file_indicators {
                    let ftype = metadata.file_type();
                    if ftype.is_dir() {
                        display_name.push('/');
                    } else if ftype.is_symlink() && !flags.long_format {
                        display_name.push('@');
                    } else if ftype.is_fifo() {
                        display_name.push('|');
                    } else if ftype.is_socket() {
                        display_name.push('=');
                    } else if !ftype.is_symlink() && self.is_executable(&metadata) {
                        display_name.push('*');
                    }
                }

                let link_suffix = if ftype.is_symlink() {
                    match std::fs::read_link(entry.path()) {
                        Ok(target) => {
                            let mut target_str = target.display().to_string();
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
                        }
                        Err(_) => String::from(" -> (broken)"),
                    }
                } else {
                    String::new()
                };

                max_perms_width = max_perms_width.max(perms.len());
                max_nlink_width = max_nlink_width.max(nlink.to_string().len());
                max_owner_width = max_owner_width.max(owner.len());
                max_group_width = max_group_width.max(group.len());
                max_size_width = max_size_width.max(size_field.len());

                file_data.push((
                    perms,
                    nlink,
                    owner,
                    group,
                    size_field,
                    time_str,
                    display_name,
                    link_suffix,
                ));
            } else {
                file_data.push((
                    "".to_string(),
                    0,
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    display_name,
                    "".to_string(),
                ));
            }
        }

        for (
            perms,
            nlink,
            owner,
            group,
            size_field,
            time_str,
            display_name,
            link_suffix,
        ) in file_data {
            println!(
                "{:<perms_width$} {:>nlink_width$} {:<owner_width$} {:<group_width$} {:>size_width$} {} {}{}",
                perms,
                nlink,
                owner,
                group,
                size_field,
                time_str,
                display_name,
                link_suffix,
                perms_width = max_perms_width,
                nlink_width = max_nlink_width,
                owner_width = max_owner_width,
                group_width = max_group_width,
                size_width = max_size_width
            );
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

        perms.push(if (mode & 0o400) != 0 { 'r' } else { '-' });
        perms.push(if (mode & 0o200) != 0 { 'w' } else { '-' });

        if (mode & 0o4000) != 0 {
            perms.push(if (mode & 0o100) != 0 { 's' } else { 'S' });
        } else {
            perms.push(if (mode & 0o100) != 0 { 'x' } else { '-' });
        }

        perms.push(if (mode & 0o040) != 0 { 'r' } else { '-' });
        perms.push(if (mode & 0o020) != 0 { 'w' } else { '-' });

        if (mode & 0o2000) != 0 {
            perms.push(if (mode & 0o010) != 0 { 's' } else { 'S' });
        } else {
            perms.push(if (mode & 0o010) != 0 { 'x' } else { '-' });
        }

        perms.push(if (mode & 0o004) != 0 { 'r' } else { '-' });
        perms.push(if (mode & 0o002) != 0 { 'w' } else { '-' });

        if (mode & 0o1000) != 0 {
            perms.push(if (mode & 0o001) != 0 { 't' } else { 'T' });
        } else {
            perms.push(if (mode & 0o001) != 0 { 'x' } else { '-' });
        }

        perms
    }

    fn format_permissions_with_extended(&self, metadata: &fs::Metadata, path: &Path) -> String {
        let mut perms = self.format_permissions(metadata);

        if self.has_extended_attributes(path) {
            perms.push('+');
        }

        perms
    }

    fn has_extended_attributes(&self, path: &Path) -> bool {
        let path_cstr = match std::ffi::CString::new(path.to_string_lossy().as_bytes()) {
            Ok(s) => s,
            Err(_) => {
                return false;
            }
        };

        unsafe {
            let mut buffer = [0u8; 1024];
            let result = libc::listxattr(
                path_cstr.as_ptr(),
                buffer.as_mut_ptr() as *mut i8,
                buffer.len()
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
        (mode & 0o111) != 0
    }
}

impl CommandExecutor for CatCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        let mut command_options = CommandOptions {
            is_option: true,
        };
        if args.is_empty() {
            return self.read_from_stdin();
        }


        for file_path in args {
            if file_path == "--" {
                command_options.is_option = false;
                if args.len() == 1 {
                    return self.read_from_stdin();
                }
                continue;
            }
            if file_path.starts_with('-') && file_path != "-" && command_options.is_option {
                return Err(ShellError::InvalidOption(file_path.clone()));
            }
            match self.process_file(file_path) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("cat: {}: {}", file_path, e);
                }
            }
        }

        Ok(())
    }

    fn help(&self) -> &str {
        "Usage: cat [file...] - Concatenate and display files (reads from stdin if no files provided)"
    }
}

impl CatCommand {
    fn read_from_stdin(&self) -> Result<(), ShellError> {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut stdout = io::stdout();

        let mut buffer = [0; 8192];
        loop {
            match handle.read(&mut buffer) {
                Ok(0) => {
                    break;
                }
                Ok(n) => {
                    stdout.write_all(&buffer[..n]).map_err(|e| ShellError::IoError(e))?;
                }
                Err(e) => {
                    return Err(ShellError::IoError(e));
                }
            }
        }

        stdout.flush().map_err(|e| ShellError::IoError(e))?;
        Ok(())
    }

    fn process_file(&self, file_path: &str) -> Result<(), ShellError> {
        if file_path == "-" {
            return self.read_from_stdin();
        }
        let content = fs::read(file_path).map_err(|e| {
            match e.kind() {
                io::ErrorKind::NotFound => {
                    ShellError::FileSystemError("No such file or directory".to_string())
                }
                io::ErrorKind::PermissionDenied => {
                    ShellError::FileSystemError("Permission denied".to_string())
                }
                io::ErrorKind::InvalidInput => {
                    ShellError::FileSystemError("Is a directory".to_string())
                }
                _ => ShellError::FileSystemError(format!("Cannot read file: {}", e)),
            }
        })?;

        io
            ::stdout()
            .write_all(&content)
            .map_err(|e| ShellError::IoError(e))?;
        io
            ::stdout()
            .flush()
            .map_err(|e| ShellError::IoError(e))?;

        Ok(())
    }
}

impl CommandExecutor for MkdirCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        if args.is_empty() {
            return Err(ShellError::ExecutionError("mkdir: missing operand".to_string()));
        }

        let mut create_parents = false;
        let mut directories = Vec::new();
        let mut is_option = true;

        for arg in args {
            if arg == "--" {
                is_option = false;
                continue;
            }
            if arg == "-p" && is_option {
                create_parents = true;
            } else if arg.starts_with('-') && is_option {
                return Err(
                    ShellError::ExecutionError(format!("mkdir: invalid option -- '{}'", &arg[1..]))
                );
            } else {
                directories.push(arg);
            }
        }

        if directories.is_empty() {
            return Err(ShellError::ExecutionError("mkdir: missing operand".to_string()));
        }

        for dir_path in directories {
            if create_parents {
                fs
                    ::create_dir_all(dir_path)
                    .map_err(|e|
                        ShellError::FileSystemError(
                            format!("Failed to create directory '{}': {}", dir_path, e)
                        )
                    )?;
            } else {
                fs
                    ::create_dir(dir_path)
                    .map_err(|e|
                        ShellError::FileSystemError(
                            format!("Failed to create directory '{}': {}", dir_path, e)
                        )
                    )?;
            }
        }

        Ok(())
    }

    fn help(&self) -> &str {
        "Usage: mkdir [-p] [directory...] - Create directories"
    }
}

impl CommandExecutor for CpCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        if args.len() < 2 {
            return Err(ShellError::ExecutionError("cp: missing operand".to_string()));
        }

        let mut recursive = false;
        let mut is_option = true;
        let mut filtered: Vec<&String> = Vec::new();

        for arg in args {
            if arg == "--" {
                is_option = false;
                continue;
            }
            if arg == "-r" && is_option {
                recursive = true;
            } else if arg.starts_with('-') && is_option {
                return Err(ShellError::InvalidOption(arg.clone()));
            } else {
                filtered.push(arg);
            }
        }

        if filtered.len() < 2 {
            return Err(ShellError::ExecutionError("cp: missing an operand".to_string()));
        }

        let target = Path::new(filtered.pop().unwrap());
        let sources = filtered;

        let mut has_errors = false;
        
        for src in sources {
            let src_path = Path::new(src);

            if !src_path.exists() {
                eprintln!("cp: cannot stat '{}': No such file or directory", src);
                has_errors = true;
                continue;
            }

            if !src_path.is_file() && !src_path.is_dir() {
                eprintln!("cp: cannot copy '{}': Not a regular file or directory", src);
                has_errors = true;
                continue;
            }

            if src_path.is_dir() {
                if !recursive {
                    eprintln!("cp: omitting directory '{}', use -r to copy", src);
                    has_errors = true;
                    continue;
                }
                
                let dest_path = if target.is_dir() {
                    target.join(src_path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("")))
                } else {
                    target.to_path_buf()
                };
                
                if dest_path.exists() && dest_path.is_file() {
                    eprintln!("cp: cannot overwrite non-directory '{}' with directory '{}'", 
                             dest_path.display(), src);
                    has_errors = true;
                    continue;
                }
                
                if let Err(err) = copy_dir_all(src_path, &dest_path) {
                    match err.kind() {
                        io::ErrorKind::PermissionDenied => {
                            eprintln!("cp: cannot copy '{}': Permission denied", src);
                        }
                        io::ErrorKind::AlreadyExists => {
                            eprintln!("cp: cannot copy '{}': Destination already exists", src);
                        }
                        _ => {
                            eprintln!("cp: failed to copy directory '{}': {}", src, err);
                        }
                    }
                    has_errors = true;
                }
            } else {
                let dest_path = if target.is_dir() {
                    target.join(src_path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("")))
                } else {
                    target.to_path_buf()
                };

                if dest_path.exists() && dest_path.is_dir() {
                    eprintln!("cp: cannot overwrite directory '{}' with non-directory '{}'", 
                             dest_path.display(), src);
                    has_errors = true;
                    continue;
                }

                if let Err(err) = fs::copy(src_path, &dest_path) {
                    match err.kind() {
                        io::ErrorKind::PermissionDenied => {
                            eprintln!("cp: cannot copy '{}': Permission denied", src);
                        }
                        io::ErrorKind::NotFound => {
                            eprintln!("cp: cannot copy '{}': Destination directory does not exist", src);
                        }
                        io::ErrorKind::InvalidInput => {
                            eprintln!("cp: cannot copy '{}': Invalid input", src);
                        }
                        _ => {
                            eprintln!("cp: failed to copy '{}': {}", src, err);
                        }
                    }
                    has_errors = true;
                }
            }
        }
        
        if has_errors {
            return Err(ShellError::ExecutionError("Some files could not be copied".to_string()));
        }

        Ok(())
    }

    fn help(&self) -> &str {
        "Usage: cp source... destination - Copy files and directories"
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_all(&path, &dst_path)?;
        } else if path.is_file() {
            fs::copy(&path, &dst_path)?;
        }
    }
    Ok(())
}

impl CommandExecutor for MvCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        if args.len() < 2 {
            return Err(ShellError::ExecutionError("mv: missing operand".to_string()));
        }

        let mut is_option = true;
        let mut filtered: Vec<&String> = Vec::new();

        for arg in args {
            if arg == "--" {
                is_option = false;
                continue;
            }
            if arg.starts_with('-') && is_option {
                return Err(ShellError::InvalidOption(arg.clone()));
            } else {
                filtered.push(arg);
            }
        }

        if filtered.len() < 2 {
            return Err(ShellError::ExecutionError("mv: missing operand".to_string()));
        }

        let target = Path::new(&filtered[filtered.len() - 1]);
        let sources = &filtered[..filtered.len() - 1];

        let mut has_errors = false;
        
        for src in sources {
            let src_path = Path::new(src);

            if !src_path.exists() {
                eprintln!("mv: cannot stat '{}': No such file or directory", src);
                has_errors = true;
                continue;
            }

            if !src_path.is_file() && !src_path.is_dir() {
                eprintln!("mv: cannot move '{}': Not a regular file or directory", src);
                has_errors = true;
                continue;
            }

            let dest_path = if target.is_dir() {
                target.join(src_path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("")))
            } else {
                target.to_path_buf()
            };

            if dest_path.exists() {
                if dest_path.is_dir() && src_path.is_file() {
                    eprintln!("mv: cannot overwrite directory '{}' with non-directory '{}'", 
                             dest_path.display(), src);
                    has_errors = true;
                    continue;
                } else if dest_path.is_file() && src_path.is_dir() {
                    eprintln!("mv: cannot overwrite non-directory '{}' with directory '{}'", 
                             dest_path.display(), src);
                    has_errors = true;
                    continue;
                }
            }

            if let Err(err) = fs::rename(src_path, &dest_path) {
                match err.kind() {
                    io::ErrorKind::PermissionDenied => {
                        eprintln!("mv: cannot move '{}': Permission denied", src);
                    }
                    io::ErrorKind::NotFound => {
                        eprintln!("mv: cannot move '{}': Destination directory does not exist", src);
                    }
                    io::ErrorKind::InvalidInput => {
                        eprintln!("mv: cannot move '{}': Invalid input", src);
                    }
                    io::ErrorKind::CrossesDevices => {
                        eprintln!("mv: cannot move '{}': Cross-device link not permitted", src);
                    }
                    _ => {
                        eprintln!("mv: failed to move '{}': {}", src, err);
                    }
                }
                has_errors = true;
            }
        }
        
        if has_errors {
            return Err(ShellError::ExecutionError("Some files could not be moved".to_string()));
        }

        Ok(())
    }

    fn help(&self) -> &str {
        "Usage: mv source... destination - Move (rename) files or directories"
    }
}
impl CommandExecutor for RmCommand {
    fn execute(&self, args: &[String]) -> Result<(), ShellError> {
        let pwd = env::current_dir().unwrap().to_string_lossy().to_string();

        if args.is_empty() {
            return Err(ShellError::ExecutionError("rm: missing operand".to_string()));
        }

        let mut recursive = false;
        let mut is_option = true;
        let mut targets: Vec<&String> = Vec::new();

        for arg in args {
            if arg.as_str() == pwd {
                return Err(
                    ShellError::ExecutionError(
                        "rm: refusing to remove current directory".to_string()
                    )
                );
            }
            if is_dot_or_dotdot(arg) {
                return Err(
                    ShellError::ExecutionError("rm: refusing to remove '.' or '..'".to_string())
                );
            }
            if arg == "--" {
                is_option = false;
                continue;
            }
            if arg == "-r" && is_option {
                recursive = true;
            } else if arg.starts_with('-') && is_option {
                return Err(ShellError::InvalidOption(arg.clone()));
            } else {
                targets.push(arg);
            }
        }

        if targets.is_empty() {
            return Err(ShellError::ExecutionError("rm: missing operand".to_string()));
        }
        let mut has_errors = false;
        
        for target in targets {
            let path = Path::new(target);

            if !path.exists() {
                eprintln!("rm: cannot remove '{}': No such file or directory", target);
                has_errors = true;
                continue;
            }

            if !path.is_file() && !path.is_dir() {
                eprintln!("rm: cannot remove '{}': Not a regular file or directory", target);
                has_errors = true;
                continue;
            }

            let result = if path.is_dir() {
                if recursive {
                    fs::remove_dir_all(path)
                } else {
                    Err(
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("rm: cannot remove '{}': Is a directory (use -r)", target)
                        )
                    )
                }
            } else {
                fs::remove_file(path)
            };

            if let Err(err) = result {
                match err.kind() {
                    io::ErrorKind::PermissionDenied => {
                        eprintln!("rm: cannot remove '{}': Permission denied", target);
                    }
                    io::ErrorKind::Other => {
                        eprintln!("rm: {}", err);
                    }
                    io::ErrorKind::InvalidInput => {
                        eprintln!("rm: cannot remove '{}': Invalid input", target);
                    }
                    _ => {
                        eprintln!("rm: failed to remove '{}': {}", target, err);
                    }
                }
                has_errors = true;
            }
        }
        
        if has_errors {
            return Err(ShellError::ExecutionError("Some files could not be removed".to_string()));
        }
        Ok(())
    }

    fn help(&self) -> &str {
        "Usage: rm [-r] [file...] - Remove files or directories"
    }
}
fn is_dot_or_dotdot(path: &str) -> bool {
    path == "." || path == ".." || path == "./" || path == "../"
}
