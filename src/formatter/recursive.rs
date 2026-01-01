use crate::config::Config;
use crate::file_entry::FileEntry;
use crate::formatter::long::{calculate_column_widths, print_long_entries_with_widths};
use crate::sort::sort_default;
use colored::Colorize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub fn format_recursive(path: &Path, config: &Config, show_hidden: bool, use_long_format: bool) {
    let recursive_long_style = &config.display.tree.recursive_long_format;
    // Print the root directory as the parent
    if let Some(dir_name) = path.file_name() {
        let dir_name_str = dir_name.to_string_lossy();

        // Create a temporary FileEntry just for getting the directory icon
        let temp_entry = FileEntry {
            path: dir_name.to_os_string(),
            is_dir: true,
            is_executable: false,
            mode: 0o755,
            size: 0,
            modified: std::time::SystemTime::UNIX_EPOCH,
            owner: String::new(),
            group: String::new(),
            nlink: 0,
        };

        let icon = temp_entry.get_icon_custom(&config.icons);
        let icon_colored = icon.color(temp_entry.get_icon_color(&config.icons.colors));
        let dir_name_colored = dir_name_str
            .color(temp_entry.get_color(&config.colors))
            .bold();

        // Only add space if icon is not empty
        if icon.is_empty() {
            println!("{}", dir_name_colored);
        } else {
            println!("{} {}", icon_colored, dir_name_colored);
        }
    }

    // Print the tree contents
    print_directory_tree(
        path,
        config,
        show_hidden,
        "",
        &config.display.tree.style,
        use_long_format,
        recursive_long_style.as_str(),
    );
}

fn print_directory_tree(
    path: &Path,
    config: &Config,
    show_hidden: bool,
    prefix: &str,
    tree_style: &str,
    use_long_format: bool,
    recursive_long_style: &str,
) {
    match fs::read_dir(path) {
        Ok(entries_iter) => {
            let mut entries: Vec<_> = entries_iter.filter_map(|e| e.ok()).collect();

            // Sort entries by filename
            entries.sort_by(|a, b| {
                let a_name = a.file_name();
                let b_name = b.file_name();
                a_name.cmp(&b_name)
            });

            // Filter out hidden files if needed
            let entries: Vec<_> = entries
                .into_iter()
                .filter(|e| {
                    if show_hidden {
                        true
                    } else {
                        let file_name = e.file_name();
                        let name_str = file_name.to_string_lossy();
                        !name_str.starts_with('.')
                    }
                })
                .collect();

            if use_long_format {
                if recursive_long_style == "header" {
                    // Header-style output (list format with directory paths as headers)
                    print_long_format_with_headers(
                        &entries,
                        path,
                        config,
                        prefix,
                        show_hidden,
                        tree_style,
                        recursive_long_style,
                    );
                } else {
                    // Nested-style output (inline indentation for subdirectories)
                    // Collect FileEntry objects for long format, separating files and dirs
                    let mut file_entries: Vec<(FileEntry, std::path::PathBuf)> = Vec::new();

                    for entry in entries.iter() {
                        let entry_path = entry.path();
                        let file_name = entry.file_name();
                        if let Ok(metadata) = entry.metadata() {
                            let is_dir = metadata.is_dir();
                            let is_executable =
                                !is_dir && (metadata.permissions().mode() & 0o111) != 0;

                            let file_entry = FileEntry {
                                path: file_name.clone(),
                                is_dir,
                                is_executable,
                                mode: metadata.permissions().mode(),
                                size: metadata.len(),
                                modified: metadata
                                    .modified()
                                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                                owner: String::new(),
                                group: String::new(),
                                nlink: 0,
                            };

                            file_entries.push((file_entry, entry_path));
                        }
                    }

                    // Calculate widths once for all entries at this level
                    let mut entries_only: Vec<FileEntry> =
                        file_entries.iter().map(|(e, _)| e.clone()).collect();
                    // Apply default sorting: by type, then alphabetically (case-insensitive)
                    sort_default(&mut entries_only);
                    let fields = &config.display.long_format_fields;
                    let widths = calculate_column_widths(&entries_only, fields);

                    // Print each entry and recurse into directories immediately after
                    for entry in &entries_only {
                        // Find the corresponding path from the original file_entries
                        let entry_path = file_entries
                            .iter()
                            .find(|(e, _)| e.path.to_string_lossy() == entry.path.to_string_lossy())
                            .map(|(_, p)| p.clone())
                            .unwrap_or_else(|| std::path::PathBuf::new());
                        // Print this entry using pre-calculated widths
                        let single_entry = vec![entry.clone()];
                        print_long_entries_with_widths(
                            &single_entry,
                            config,
                            prefix,
                            fields,
                            &widths,
                        );

                        // If it's a directory, recurse into it immediately
                        if entry.is_dir {
                            let new_prefix = format!("{}    ", prefix);
                            print_directory_tree(
                                &entry_path,
                                config,
                                show_hidden,
                                &new_prefix,
                                tree_style,
                                use_long_format,
                                recursive_long_style,
                            );
                        }
                    }
                }
            } else {
                // Tree-style output (original behavior)
                for (idx, entry) in entries.iter().enumerate() {
                    let is_last = idx == entries.len() - 1;
                    let entry_path = entry.path();
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();

                    // Get metadata
                    if let Ok(metadata) = entry.metadata() {
                        let is_dir = metadata.is_dir();
                        let is_executable = !is_dir && (metadata.permissions().mode() & 0o111) != 0;

                        // Create FileEntry for icon/color handling
                        let file_entry = FileEntry {
                            path: file_name.clone(),
                            is_dir,
                            is_executable,
                            mode: metadata.permissions().mode(),
                            size: metadata.len(),
                            modified: metadata
                                .modified()
                                .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                            owner: String::new(),
                            group: String::new(),
                            nlink: 0,
                        };

                        // Determine tree connectors
                        let (connector, extension_prefix) = if tree_style == "ascii" {
                            if is_last {
                                ("└──", "    ")
                            } else {
                                ("├──", "│   ")
                            }
                        } else {
                            // indent style
                            ("", "  ")
                        };

                        // Get icon and color
                        let icon = file_entry.get_icon_custom(&config.icons);
                        let icon_colored =
                            icon.color(file_entry.get_icon_color(&config.icons.colors));

                        // Get filename color
                        let filename_colored = match file_entry.get_file_type() {
                            crate::file_entry::FileType::Directory
                            | crate::file_entry::FileType::Executable => file_name_str
                                .color(file_entry.get_color(&config.colors))
                                .bold(),
                            crate::file_entry::FileType::RegularFile => {
                                file_name_str.color(file_entry.get_color(&config.colors))
                            }
                        };

                        // Print the entry
                        println!(
                            "{}{}{} {}",
                            prefix, connector, icon_colored, filename_colored
                        );

                        // If it's a directory, recurse
                        if is_dir {
                            let new_prefix = if tree_style == "ascii" {
                                format!("{}{}", prefix, extension_prefix)
                            } else {
                                format!("{}  ", prefix)
                            };

                            print_directory_tree(
                                &entry_path,
                                config,
                                show_hidden,
                                &new_prefix,
                                tree_style,
                                use_long_format,
                                recursive_long_style,
                            );
                        }
                    }
                }
            }
        }
        Err(_) => {}
    }
}

fn print_long_format_with_headers(
    entries: &[std::fs::DirEntry],
    path: &std::path::Path,
    config: &crate::config::Config,
    prefix: &str,
    show_hidden: bool,
    tree_style: &str,
    recursive_long_style: &str,
) {
    // Print directory header with path
    if !prefix.is_empty() {
        println!("{}:", path.display());
    }

    // Collect FileEntry objects for long format
    let mut file_entries: Vec<(FileEntry, std::path::PathBuf)> = Vec::new();

    for entry in entries.iter() {
        let entry_path = entry.path();
        let file_name = entry.file_name();
        if let Ok(metadata) = entry.metadata() {
            let is_dir = metadata.is_dir();
            let is_executable = !is_dir && (metadata.permissions().mode() & 0o111) != 0;

            let file_entry = FileEntry {
                path: file_name.clone(),
                is_dir,
                is_executable,
                mode: metadata.permissions().mode(),
                size: metadata.len(),
                modified: metadata
                    .modified()
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                owner: String::new(),
                group: String::new(),
                nlink: 0,
            };

            file_entries.push((file_entry, entry_path));
        }
    }

    // Use the new print_long_entries function for configurable field ordering
    let mut file_entries_only: Vec<FileEntry> =
        file_entries.iter().map(|(e, _)| e.clone()).collect();
    if !file_entries_only.is_empty() {
        // Apply default sorting: by type, then alphabetically (case-insensitive)
        sort_default(&mut file_entries_only);
        let fields = &config.display.long_format_fields;
        let widths = calculate_column_widths(&file_entries_only, fields);
        print_long_entries_with_widths(&file_entries_only, config, "", fields, &widths);
    }

    // Recurse into directories
    for (entry, entry_path) in &file_entries {
        if entry.is_dir {
            let new_prefix = format!("{}    ", prefix);
            print_directory_tree(
                &entry_path,
                config,
                show_hidden,
                &new_prefix,
                tree_style,
                true,
                recursive_long_style,
            );
        }
    }
}
