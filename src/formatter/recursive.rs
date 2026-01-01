use crate::config::Config;
use crate::file_entry::FileEntry;
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
                    // Collect FileEntry objects for long format
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

                    // Use the format_long function logic
                    if !file_entries.is_empty() {
                        let max_nlink_width = file_entries
                            .iter()
                            .map(|(e, _)| e.nlink.to_string().len())
                            .max()
                            .unwrap_or(0);

                        let max_owner_width = file_entries
                            .iter()
                            .map(|(e, _)| e.owner.len())
                            .max()
                            .unwrap_or(0);

                        let max_group_width = file_entries
                            .iter()
                            .map(|(e, _)| e.group.len())
                            .max()
                            .unwrap_or(0);

                        let max_size_width = file_entries
                            .iter()
                            .map(|(e, _)| e.format_size().len())
                            .max()
                            .unwrap_or(0);

                        for (entry, entry_path) in &file_entries {
                            let permissions = entry.format_permissions();
                            let nlink = entry.nlink.to_string();
                            let owner = &entry.owner;
                            let group = &entry.group;
                            let size = entry.format_size();
                            let modified = entry.format_modified();
                            let icon = entry.get_icon_custom(&config.icons);
                            let filename = entry.path.to_string_lossy();
                            let icon_color = entry.get_icon_color(&config.icons.colors);

                            let filename_colored = match entry.get_file_type() {
                                crate::file_entry::FileType::Directory
                                | crate::file_entry::FileType::Executable => {
                                    filename.color(entry.get_color(&config.colors)).bold()
                                }
                                crate::file_entry::FileType::RegularFile => {
                                    filename.color(entry.get_color(&config.colors))
                                }
                            };

                            println!(
                                "{}{}  {:>nlink_width$}  {:<owner_width$}  {:<group_width$}  {:>size_width$}  {}  {} {}",
                                prefix,
                                permissions,
                                nlink,
                                owner,
                                group,
                                size,
                                modified,
                                icon.color(icon_color),
                                filename_colored,
                                nlink_width = max_nlink_width,
                                owner_width = max_owner_width,
                                group_width = max_group_width,
                                size_width = max_size_width,
                            );

                            // If it's a directory, recurse into it with indentation
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

    // Use the format_long function logic
    if !file_entries.is_empty() {
        let max_nlink_width = file_entries
            .iter()
            .map(|(e, _)| e.nlink.to_string().len())
            .max()
            .unwrap_or(0);

        let max_owner_width = file_entries
            .iter()
            .map(|(e, _)| e.owner.len())
            .max()
            .unwrap_or(0);

        let max_group_width = file_entries
            .iter()
            .map(|(e, _)| e.group.len())
            .max()
            .unwrap_or(0);

        let max_size_width = file_entries
            .iter()
            .map(|(e, _)| e.format_size().len())
            .max()
            .unwrap_or(0);

        for (entry, _entry_path) in &file_entries {
            let permissions = entry.format_permissions();
            let nlink = entry.nlink.to_string();
            let owner = &entry.owner;
            let group = &entry.group;
            let size = entry.format_size();
            let modified = entry.format_modified();
            let icon = entry.get_icon_custom(&config.icons);
            let filename = entry.path.to_string_lossy();
            let icon_color = entry.get_icon_color(&config.icons.colors);

            let filename_colored = match entry.get_file_type() {
                crate::file_entry::FileType::Directory
                | crate::file_entry::FileType::Executable => {
                    filename.color(entry.get_color(&config.colors)).bold()
                }
                crate::file_entry::FileType::RegularFile => {
                    filename.color(entry.get_color(&config.colors))
                }
            };

            println!(
                "{}  {:>nlink_width$}  {:<owner_width$}  {:<group_width$}  {:>size_width$}  {}  {} {}",
                permissions,
                nlink,
                owner,
                group,
                size,
                modified,
                icon.color(icon_color),
                filename_colored,
                nlink_width = max_nlink_width,
                owner_width = max_owner_width,
                group_width = max_group_width,
                size_width = max_size_width,
            );
        }
    }

    // Recurse into directories
    for entry in entries.iter() {
        let entry_path = entry.path();
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_dir() {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();
                if !show_hidden || !file_name_str.starts_with('.') {
                    println!();
                    print_directory_tree(
                        &entry_path,
                        config,
                        show_hidden,
                        &format!("{}  ", prefix),
                        tree_style,
                        true,
                        recursive_long_style,
                    );
                }
            }
        }
    }
}
