use crate::file_entry::{FileEntry, FileType};
use colored::Colorize;
use unicode_width::UnicodeWidthStr;

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
    let max_name_width = all_entries
        .iter()
        .map(|e| {
            let filename = e.path.file_name().unwrap().to_string_lossy();
            UnicodeWidthStr::width(e.get_icon().as_str())
                + 1
                + UnicodeWidthStr::width(filename.as_ref())
        })
        .max()
        .unwrap_or(0);

    let max_owner_width = all_entries.iter().map(|e| e.owner.len()).max().unwrap_or(0);

    let max_size_width = all_entries
        .iter()
        .map(|e| e.format_size().len())
        .max()
        .unwrap_or(0);

    // Format: <icon> <name> <owner> <modification datetime> <size> <permissions>
    for entry in all_entries {
        let icon = entry.get_icon();
        let filename = entry.path.file_name().unwrap().to_string_lossy();
        let color = entry.get_color();

        let name_width =
            UnicodeWidthStr::width(icon.as_str()) + 1 + UnicodeWidthStr::width(filename.as_ref());
        let name_padding = if name_width < max_name_width {
            max_name_width - name_width
        } else {
            0
        };

        let owner = &entry.owner;
        let modified = entry.format_modified();
        let size = entry.format_size();
        let permissions = entry.format_permissions();

        println!(
            "{} {}{}  {:<owner_width$}  {}  {:>size_width$}  {}",
            icon,
            filename.color(color).bold(),
            " ".repeat(name_padding),
            owner,
            modified,
            size,
            permissions,
            owner_width = max_owner_width,
            size_width = max_size_width,
        );
    }
}
