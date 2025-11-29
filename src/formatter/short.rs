use crate::config::Config;
use crate::file_entry::{FileEntry, FileType};
use colored::Colorize;
use unicode_width::UnicodeWidthStr;

pub fn format_short(entries: Vec<FileEntry>, config: &Config) {
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

    let column_spacing = config.display.column_spacing;
    let max_rows = config.display.max_rows;

    // If max_rows is set (not 0), format each file type with wrapping
    // Otherwise, use the original single-column-per-type format
    if max_rows > 0 {
        format_with_max_rows(
            directories,
            executables,
            regular_files,
            max_rows,
            column_spacing,
            config,
        );
    } else {
        format_single_column_per_type(
            directories,
            executables,
            regular_files,
            column_spacing,
            config,
        );
    }
}

fn format_with_max_rows(
    directories: Vec<FileEntry>,
    executables: Vec<FileEntry>,
    regular_files: Vec<FileEntry>,
    max_rows: usize,
    column_spacing: usize,
    config: &Config,
) {
    // Calculate how many columns needed for each file type
    let dir_num_cols = if directories.is_empty() {
        0
    } else {
        (directories.len() + max_rows - 1) / max_rows
    };

    let exec_num_cols = if executables.is_empty() {
        0
    } else {
        (executables.len() + max_rows - 1) / max_rows
    };

    let file_num_cols = if regular_files.is_empty() {
        0
    } else {
        (regular_files.len() + max_rows - 1) / max_rows
    };

    // Calculate width for each file type
    let dir_width = directories
        .iter()
        .map(|e| {
            let filename = e.path.to_string_lossy();
            UnicodeWidthStr::width(e.get_icon().as_str())
                + 1
                + UnicodeWidthStr::width(filename.as_ref())
        })
        .max()
        .unwrap_or(0);

    let exec_width = executables
        .iter()
        .map(|e| {
            let filename = e.path.to_string_lossy();
            UnicodeWidthStr::width(e.get_icon().as_str())
                + 1
                + UnicodeWidthStr::width(filename.as_ref())
        })
        .max()
        .unwrap_or(0);

    let file_width = regular_files
        .iter()
        .map(|e| {
            let filename = e.path.to_string_lossy();
            UnicodeWidthStr::width(e.get_icon().as_str())
                + 1
                + UnicodeWidthStr::width(filename.as_ref())
        })
        .max()
        .unwrap_or(0);

    // Print rows, with all file types side-by-side
    for row in 0..max_rows {
        let mut line = String::new();
        let mut has_any_content = false;

        // Directories section
        if dir_num_cols > 0 {
            for col in 0..dir_num_cols {
                let idx = col * max_rows + row;
                if idx < directories.len() {
                    if col > 0 {
                        line.push_str(&" ".repeat(column_spacing));
                    }

                    let entry = &directories[idx];
                    let filename = entry.path.to_string_lossy();
                    let icon = entry.get_icon_custom(&config.icons);
                    let actual_width = UnicodeWidthStr::width(icon.as_str())
                        + 1
                        + UnicodeWidthStr::width(filename.as_ref());

                    line.push_str(&format!(
                        "{} {}",
                        icon.color(entry.get_icon_color(&config.icons.colors)),
                        filename.color(entry.get_color(&config.colors)).bold()
                    ));

                    if actual_width < dir_width {
                        line.push_str(&" ".repeat(dir_width - actual_width));
                    }
                    has_any_content = true;
                } else if col == 0 {
                    // Empty row in directories section, but still need spacing for alignment
                    line.push_str(&" ".repeat(dir_width));
                } else {
                    line.push_str(&" ".repeat(column_spacing + dir_width));
                }
            }

            // Add spacing after directories if executables or files exist
            if exec_num_cols > 0 || file_num_cols > 0 {
                line.push_str(&" ".repeat(column_spacing));
            }
        }

        // Executables section
        if exec_num_cols > 0 {
            for col in 0..exec_num_cols {
                let idx = col * max_rows + row;
                if idx < executables.len() {
                    if col > 0 {
                        line.push_str(&" ".repeat(column_spacing));
                    }

                    let entry = &executables[idx];
                    let filename = entry.path.to_string_lossy();
                    let icon = entry.get_icon_custom(&config.icons);
                    let actual_width = UnicodeWidthStr::width(icon.as_str())
                        + 1
                        + UnicodeWidthStr::width(filename.as_ref());

                    line.push_str(&format!(
                        "{} {}",
                        icon.color(entry.get_icon_color(&config.icons.colors)),
                        filename.color(entry.get_color(&config.colors)).bold()
                    ));

                    if actual_width < exec_width {
                        line.push_str(&" ".repeat(exec_width - actual_width));
                    }
                    has_any_content = true;
                } else if col == 0 {
                    line.push_str(&" ".repeat(exec_width));
                } else {
                    line.push_str(&" ".repeat(column_spacing + exec_width));
                }
            }

            // Add spacing after executables if files exist
            if file_num_cols > 0 {
                line.push_str(&" ".repeat(column_spacing));
            }
        }

        // Regular files section
        if file_num_cols > 0 {
            for col in 0..file_num_cols {
                let idx = col * max_rows + row;
                if idx < regular_files.len() {
                    if col > 0 {
                        line.push_str(&" ".repeat(column_spacing));
                    }

                    let entry = &regular_files[idx];
                    let filename = entry.path.to_string_lossy();
                    let icon = entry.get_icon_custom(&config.icons);
                    let actual_width = UnicodeWidthStr::width(icon.as_str())
                        + 1
                        + UnicodeWidthStr::width(filename.as_ref());

                    line.push_str(&format!(
                        "{} {}",
                        icon.color(entry.get_icon_color(&config.icons.colors)),
                        filename.color(entry.get_color(&config.colors))
                    ));

                    if col < file_num_cols - 1 && actual_width < file_width {
                        line.push_str(&" ".repeat(file_width - actual_width));
                    }
                    has_any_content = true;
                }
            }
        }

        if has_any_content {
            println!("{}", line.trim_end());
        }
    }
}

fn format_single_column_per_type(
    directories: Vec<FileEntry>,
    executables: Vec<FileEntry>,
    regular_files: Vec<FileEntry>,
    column_spacing: usize,
    config: &Config,
) {
    // Calculate column widths (icon + space + filename)
    let dir_width = directories
        .iter()
        .map(|e| {
            let filename = e.path.to_string_lossy();
            let icon = e.get_icon_custom(&config.icons);
            UnicodeWidthStr::width(icon.as_str()) + 1 + UnicodeWidthStr::width(filename.as_ref())
        })
        .max()
        .unwrap_or(0);

    let exec_width = executables
        .iter()
        .map(|e| {
            let filename = e.path.to_string_lossy();
            let icon = e.get_icon_custom(&config.icons);
            UnicodeWidthStr::width(icon.as_str()) + 1 + UnicodeWidthStr::width(filename.as_ref())
        })
        .max()
        .unwrap_or(0);

    let file_width = regular_files
        .iter()
        .map(|e| {
            let filename = e.path.to_string_lossy();
            let icon = e.get_icon_custom(&config.icons);
            UnicodeWidthStr::width(icon.as_str()) + 1 + UnicodeWidthStr::width(filename.as_ref())
        })
        .max()
        .unwrap_or(0);

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
                let filename = entry.path.to_string_lossy();
                let icon = entry.get_icon_custom(&config.icons);
                let actual_width = UnicodeWidthStr::width(icon.as_str())
                    + 1
                    + UnicodeWidthStr::width(filename.as_ref());

                line.push_str(&format!(
                    "{} {}",
                    icon.color(entry.get_icon_color(&config.icons.colors)),
                    filename.color(entry.get_color(&config.colors)).bold()
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
                let filename = entry.path.to_string_lossy();
                let icon = entry.get_icon_custom(&config.icons);
                let actual_width = UnicodeWidthStr::width(icon.as_str())
                    + 1
                    + UnicodeWidthStr::width(filename.as_ref());

                line.push_str(&format!(
                    "{} {}",
                    icon.color(entry.get_icon_color(&config.icons.colors)),
                    filename.color(entry.get_color(&config.colors)).bold()
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
            let filename = entry.path.to_string_lossy();
            let icon = entry.get_icon_custom(&config.icons);
            let actual_width = UnicodeWidthStr::width(icon.as_str())
                + 1
                + UnicodeWidthStr::width(filename.as_ref());

            line.push_str(&format!(
                "{} {}",
                icon.color(entry.get_icon_color(&config.icons.colors)),
                filename.color(entry.get_color(&config.colors))
            ));
            // Add padding after the colored text
            if actual_width < file_width {
                line.push_str(&" ".repeat(file_width - actual_width));
            }
        }

        println!("{}", line.trim_end());
    }
}
