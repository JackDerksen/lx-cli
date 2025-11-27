use crate::file_entry::{FileEntry, FileType};
use colored::Colorize;

pub fn format_long(entries: Vec<FileEntry>) {
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

    // Combine all entries in type order
    let mut all_entries = Vec::new();
    all_entries.extend(directories);
    all_entries.extend(executables);
    all_entries.extend(regular_files);

    // Calculate column widths for alignment
    let max_nlink_width = all_entries
        .iter()
        .map(|e| e.nlink.to_string().len())
        .max()
        .unwrap_or(0);

    let max_owner_width = all_entries.iter().map(|e| e.owner.len()).max().unwrap_or(0);

    let max_group_width = all_entries.iter().map(|e| e.group.len()).max().unwrap_or(0);

    let max_size_width = all_entries
        .iter()
        .map(|e| e.format_size().len())
        .max()
        .unwrap_or(0);

    // Format: <permissions> <nlink> <owner> <group> <size> <date> <icon> <name>
    for entry in all_entries {
        let permissions = entry.format_permissions();
        let nlink = entry.nlink.to_string();
        let owner = &entry.owner;
        let group = &entry.group;
        let size = entry.format_size();
        let modified = entry.format_modified();
        let icon = entry.get_icon();
        let filename = entry.path.file_name().unwrap().to_string_lossy();
        let color = entry.get_color();

        println!(
            "{}  {:>nlink_width$}  {:<owner_width$}  {:<group_width$}  {:>size_width$}  {}  {} {}",
            permissions,
            nlink,
            owner,
            group,
            size,
            modified,
            icon,
            filename.color(color).bold(),
            nlink_width = max_nlink_width,
            owner_width = max_owner_width,
            group_width = max_group_width,
            size_width = max_size_width,
        );
    }
}
