use crate::config::Config;
use crate::file_entry::FileEntry;
use crate::formatter::long::{calculate_column_widths, print_long_entries_with_widths};
use crate::formatter::{format_long, format_one_per_line};
use crate::reader::{MetadataMode, read_directory_entries, read_target};
use crate::sort::sort_default;
use colored::Colorize;
use std::path::Path;

pub fn format_recursive(path: &Path, config: &Config, show_hidden: bool, use_long_format: bool) {
    if !path.is_dir() {
        let metadata_mode = if use_long_format {
            MetadataMode::Full
        } else {
            MetadataMode::Basic
        };

        if let Ok(entries) = read_target(path, show_hidden, metadata_mode) {
            if use_long_format {
                format_long(entries, config);
            } else {
                format_one_per_line(entries, config);
            }
        }

        return;
    }

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
    let metadata_mode = if use_long_format {
        MetadataMode::Full
    } else {
        MetadataMode::Basic
    };

    if let Ok(mut entries) = read_directory_entries(path, show_hidden, metadata_mode) {
        if use_long_format {
            sort_discovered_entries(&mut entries);

            if recursive_long_style == "header" {
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
                let entries_only: Vec<FileEntry> =
                    entries.iter().map(|entry| entry.entry.clone()).collect();
                let fields = &config.display.long_format_fields;
                let widths = calculate_column_widths(&entries_only, fields);

                for discovered_entry in &entries {
                    let single_entry = [discovered_entry.entry.clone()];
                    print_long_entries_with_widths(&single_entry, config, prefix, fields, &widths);

                    if discovered_entry.entry.is_dir {
                        let new_prefix = format!("{}    ", prefix);
                        print_directory_tree(
                            &discovered_entry.full_path,
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
            entries.sort_by_cached_key(|entry| entry.entry.path.to_string_lossy().to_lowercase());

            for (idx, discovered_entry) in entries.iter().enumerate() {
                let is_last = idx == entries.len() - 1;
                let file_entry = &discovered_entry.entry;
                let file_name_str = file_entry.path.to_string_lossy();

                // Determine tree connectors
                let (connector, extension_prefix) = if tree_style == "ascii" {
                    if is_last {
                        ("└──", "   ")
                    } else {
                        ("├──", "│  ")
                    }
                } else {
                    // indent style
                    ("", "  ")
                };

                // Get icon and color
                let icon = file_entry.get_icon_custom(&config.icons);
                let icon_colored = icon.color(file_entry.get_icon_color(&config.icons.colors));

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
                if icon.is_empty() {
                    println!("{}{}{}", prefix, connector, filename_colored);
                } else {
                    println!(
                        "{}{}{} {}",
                        prefix, connector, icon_colored, filename_colored
                    );
                }

                // If it's a directory, recurse
                if file_entry.is_dir {
                    let new_prefix = if tree_style == "ascii" {
                        format!("{}{}", prefix, extension_prefix)
                    } else {
                        format!("{}  ", prefix)
                    };

                    print_directory_tree(
                        &discovered_entry.full_path,
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

fn sort_discovered_entries(entries: &mut [crate::reader::DiscoveredEntry]) {
    entries.sort_by_cached_key(|entry| {
        (
            entry.entry.get_file_type(),
            entry.entry.path.to_string_lossy().to_lowercase(),
        )
    });
}

fn print_long_format_with_headers(
    entries: &[crate::reader::DiscoveredEntry],
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

    // Use the new print_long_entries function for configurable field ordering
    let mut file_entries_only: Vec<FileEntry> =
        entries.iter().map(|entry| entry.entry.clone()).collect();
    if !file_entries_only.is_empty() {
        // Apply default sorting: by type, then alphabetically (case-insensitive)
        sort_default(&mut file_entries_only);
        let fields = &config.display.long_format_fields;
        let widths = calculate_column_widths(&file_entries_only, fields);
        print_long_entries_with_widths(&file_entries_only, config, "", fields, &widths);
    }

    // Recurse into directories
    for entry in entries {
        if entry.entry.is_dir {
            let new_prefix = format!("{}    ", prefix);
            print_directory_tree(
                &entry.full_path,
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
