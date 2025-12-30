use crate::config::Config;
use crate::file_entry::{FileEntry, FileType};
use colored::Colorize;

pub fn format_one_per_line(entries: Vec<FileEntry>, config: &Config) {
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

    // Sort each file type alphabetically by filename
    directories.sort_by(|a, b| {
        let a_name = a.path.to_string_lossy();
        let b_name = b.path.to_string_lossy();
        a_name.cmp(&b_name)
    });
    executables.sort_by(|a, b| {
        let a_name = a.path.to_string_lossy();
        let b_name = b.path.to_string_lossy();
        a_name.cmp(&b_name)
    });
    regular_files.sort_by(|a, b| {
        let a_name = a.path.to_string_lossy();
        let b_name = b.path.to_string_lossy();
        a_name.cmp(&b_name)
    });

    // Print directories
    for entry in directories {
        let filename = entry.path.to_string_lossy();
        let icon = entry.get_icon_custom(&config.icons);
        println!(
            "{} {}",
            icon.color(entry.get_icon_color(&config.icons.colors)),
            filename.color(entry.get_color(&config.colors)).bold()
        );
    }

    // Print executables
    for entry in executables {
        let filename = entry.path.to_string_lossy();
        let icon = entry.get_icon_custom(&config.icons);
        println!(
            "{} {}",
            icon.color(entry.get_icon_color(&config.icons.colors)),
            filename.color(entry.get_color(&config.colors)).bold()
        );
    }

    // Print regular files
    for entry in regular_files {
        let filename = entry.path.to_string_lossy();
        let icon = entry.get_icon_custom(&config.icons);
        println!(
            "{} {}",
            icon.color(entry.get_icon_color(&config.icons.colors)),
            filename.color(entry.get_color(&config.colors))
        );
    }
}
