use crate::config::Config;
use crate::file_entry::{FileEntry, FileType};
use crate::sort::sort_default;
use colored::Colorize;

pub fn format_one_per_line(mut entries: Vec<FileEntry>, config: &Config) {
    // Apply default sorting: by type, then alphabetically (case-insensitive)
    sort_default(&mut entries);

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
    for entry in directories {
        let filename = entry.path.to_string_lossy();
        let icon = entry.get_icon_custom(&config.icons);
        let filename_colored = filename.color(entry.get_color(&config.colors)).bold();

        if icon.is_empty() {
            println!("{}", filename_colored);
        } else {
            println!(
                "{} {}",
                icon.color(entry.get_icon_color(&config.icons.colors)),
                filename_colored
            );
        }
    }

    // Print executables
    for entry in executables {
        let filename = entry.path.to_string_lossy();
        let icon = entry.get_icon_custom(&config.icons);
        let filename_colored = filename.color(entry.get_color(&config.colors)).bold();

        if icon.is_empty() {
            println!("{}", filename_colored);
        } else {
            println!(
                "{} {}",
                icon.color(entry.get_icon_color(&config.icons.colors)),
                filename_colored
            );
        }
    }

    // Print regular files
    for entry in regular_files {
        let filename = entry.path.to_string_lossy();
        let icon = entry.get_icon_custom(&config.icons);
        let filename_colored = filename.color(entry.get_color(&config.colors));

        if icon.is_empty() {
            println!("{}", filename_colored);
        } else {
            println!(
                "{} {}",
                icon.color(entry.get_icon_color(&config.icons.colors)),
                filename_colored
            );
        }
    }
}
