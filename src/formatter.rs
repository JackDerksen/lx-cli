use crate::file_entry::{FileEntry, FileType};
use colored::Colorize;
use unicode_width::UnicodeWidthStr;

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

        // Calculate column widths (icon + space + filename)
        let dir_width = directories
            .iter()
            .map(|e| {
                let filename = e.path.file_name().unwrap().to_string_lossy();
                UnicodeWidthStr::width(e.get_icon().as_str())
                    + 1
                    + UnicodeWidthStr::width(filename.as_ref())
            })
            .max()
            .unwrap_or(0);

        let exec_width = executables
            .iter()
            .map(|e| {
                let filename = e.path.file_name().unwrap().to_string_lossy();
                UnicodeWidthStr::width(e.get_icon().as_str())
                    + 1
                    + UnicodeWidthStr::width(filename.as_ref())
            })
            .max()
            .unwrap_or(0);

        let file_width = regular_files
            .iter()
            .map(|e| {
                let filename = e.path.file_name().unwrap().to_string_lossy();
                UnicodeWidthStr::width(e.get_icon().as_str())
                    + 1
                    + UnicodeWidthStr::width(filename.as_ref())
            })
            .max()
            .unwrap_or(0);

        let column_spacing = 2;

        // Determine how many rows we need
        let max_rows = *[directories.len(), executables.len(), regular_files.len()]
            .iter()
            .max()
            .unwrap_or(&0);

        // Print side-by-side columns
        for i in 0..max_rows {
            let mut line = String::new();

            // Directory column
            if dir_width > 0 {
                if i < directories.len() {
                    let entry = &directories[i];
                    let filename = entry.path.file_name().unwrap().to_string_lossy();
                    let icon = entry.get_icon();
                    let actual_width = UnicodeWidthStr::width(icon.as_str())
                        + 1
                        + UnicodeWidthStr::width(filename.as_ref());

                    line.push_str(&format!(
                        "{} {}",
                        icon,
                        filename.color(entry.get_color()).bold()
                    ));
                    // Add padding after the colored text
                    if actual_width < dir_width {
                        line.push_str(&" ".repeat(dir_width - actual_width));
                    }
                } else {
                    // Empty space for this row in directory column
                    line.push_str(&" ".repeat(dir_width));
                }

                // Add spacing after directory column if there are more columns
                if exec_width > 0 || file_width > 0 {
                    line.push_str(&" ".repeat(column_spacing));
                }
            }

            // Executable column
            if exec_width > 0 {
                if i < executables.len() {
                    let entry = &executables[i];
                    let filename = entry.path.file_name().unwrap().to_string_lossy();
                    let icon = entry.get_icon();
                    let actual_width = UnicodeWidthStr::width(icon.as_str())
                        + 1
                        + UnicodeWidthStr::width(filename.as_ref());

                    line.push_str(&format!(
                        "{} {}",
                        icon,
                        filename.color(entry.get_color()).bold()
                    ));
                    // Add padding after the colored text
                    if actual_width < exec_width {
                        line.push_str(&" ".repeat(exec_width - actual_width));
                    }
                } else {
                    // Empty space for this row in executable column
                    line.push_str(&" ".repeat(exec_width));
                }

                // Add spacing after executable column if there are regular files
                if file_width > 0 {
                    line.push_str(&" ".repeat(column_spacing));
                }
            }

            // Regular files column
            if i < regular_files.len() {
                let entry = &regular_files[i];
                let filename = entry.path.file_name().unwrap().to_string_lossy();
                let icon = entry.get_icon();
                let actual_width = UnicodeWidthStr::width(icon.as_str())
                    + 1
                    + UnicodeWidthStr::width(filename.as_ref());

                line.push_str(&format!("{} {}", icon, filename.color(entry.get_color())));
                // Add padding after the colored text
                if actual_width < file_width {
                    line.push_str(&" ".repeat(file_width - actual_width));
                }
            }

            println!("{}", line.trim_end());
        }
    }

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

        // Calculate padding
        let mut max_path_width = 0;
        for entry in &all_entries {
            let path_name = entry.path.to_string_lossy();
            if path_name.len() > max_path_width {
                max_path_width = path_name.len();
            }
        }
        let padded_width = max_path_width + 2;

        for entry in all_entries {
            let path_name = entry.path.to_string_lossy();
            let permissions = entry.format_permissions();
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
