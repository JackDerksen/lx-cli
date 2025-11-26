/// Output formatting logic, separated into `format_short()` and `format_long()` methods.
use crate::file_entry::{FileEntry, FileType};
use colored::Colorize;

pub struct Formatter;

impl Formatter {
    pub fn format_short(entries: Vec<FileEntry>) {
        let mut directories: Vec<FileEntry> = Vec::new();
        let mut executables: Vec<FileEntry> = Vec::new();
        let mut regular_files: Vec<FileEntry> = Vec::new();

        for entry in entries {
            match entry.get_file_type() {
                FileType::Directory => directories.push(entry),
                FileType::Executable => executables.push(entry),
                FileType::RegularFile => regular_files.push(entry),
            }
        }

        // Print directories
        if !directories.is_empty() {
            println!("{}", "Directories:".bold().underline());
            for entry in directories {
                let filename = entry.path.file_name().unwrap().to_string_lossy();
                println!(
                    "  {} {}",
                    entry.get_icon(),
                    filename.color(entry.get_color()).bold()
                );
            }
            println!();
        }

        // Print executables
        if !executables.is_empty() {
            println!("{}", "Executables:".bold().underline());
            for entry in executables {
                let filename = entry.path.file_name().unwrap().to_string_lossy();
                println!(
                    "  {} {}",
                    entry.get_icon(),
                    filename.color(entry.get_color()).bold()
                );
            }
            println!();
        }

        // Print regular files
        if !regular_files.is_empty() {
            println!("{}", "Files:".bold().underline());
            for entry in regular_files {
                let filename = entry.path.file_name().unwrap().to_string_lossy();
                println!(
                    "  {} {}",
                    entry.get_icon(),
                    filename.color(entry.get_color())
                );
            }
        }
    }

    pub fn format_long(entries: Vec<FileEntry>) {
        let mut max_path_width = 0;
        for entry in &entries {
            let path_name = entry.path.to_string_lossy();
            if path_name.len() > max_path_width {
                max_path_width = path_name.len();
            }
        }
        let padded_width = max_path_width + 2;

        for entry in entries {
            let path_name = entry.path.to_string_lossy();
            let permissions = "drwxr-xr-x"; // Get from metadata properly
            let icon = entry.get_icon();
            let filename = entry.path.file_name().unwrap().to_string_lossy();
            let color = entry.get_color();

            let path_column = format!("{:<width$}", path_name, width = padded_width);

            println!(
                "{}{}  {} {}",
                path_column,
                permissions,
                icon,
                filename.color(color).bold()
            );
        }
    }
}
