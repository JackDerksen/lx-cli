use crate::config::Config;
use crate::file_entry::{FileEntry, FileType};
use crate::formatter::long::{
    calculate_column_widths, calculate_column_widths_with_filename_prefixes,
    print_long_entries_with_filename_prefixes, print_long_entries_with_widths,
};
use crate::formatter::tree::{TreeEntry, TreeRenderer};
use crate::formatter::{format_long, format_one_per_line};
use crate::reader::{
    DiscoveredEntry, MetadataMode, read_directory_entries, read_entry, read_target,
};
use colored::Colorize;
use std::io;
use std::path::Path;

pub fn format_recursive(
    path: &Path,
    config: &Config,
    show_hidden: bool,
    use_long_format: bool,
) -> io::Result<()> {
    if !path.is_dir() {
        let metadata_mode = if use_long_format {
            MetadataMode::Full
        } else {
            MetadataMode::Basic
        };
        let entries = read_target(path, show_hidden, metadata_mode)?;

        if use_long_format {
            format_long(entries, config);
        } else {
            format_one_per_line(entries, config);
        }
        return Ok(());
    }

    let metadata_mode = if use_long_format {
        MetadataMode::Full
    } else {
        MetadataMode::Basic
    };
    let root = read_entry(path, metadata_mode)?;

    if use_long_format && config.display.tree.recursive_long_format == "header" {
        print_tree_root(&root, config);
        return print_long_format_with_headers(path, config, show_hidden, true);
    }

    let uses_icons = !use_long_format
        && [
            config.icons.directory.as_str(),
            config.icons.executable.as_str(),
            config.icons.regular.as_str(),
        ]
        .iter()
        .any(|icon| !icon.is_empty());
    let renderer = TreeRenderer::new(&config.display.tree.style, uses_icons);
    let sort_entries = if use_long_format {
        sort_by_type_then_name
    } else {
        sort_by_name
    };
    let tree_entries = renderer.collect(path, show_hidden, metadata_mode, sort_entries)?;

    if use_long_format {
        print_nested_long_tree(&root, &tree_entries, config);
    } else {
        print_tree_root(&root, config);
        print_short_tree(&tree_entries, config);
    }

    Ok(())
}

fn print_nested_long_tree(root: &FileEntry, tree_entries: &[TreeEntry], config: &Config) {
    let mut display_entries = Vec::with_capacity(tree_entries.len() + 1);
    let mut filename_prefixes = vec![String::new()];
    display_entries.push(display_entry(root));
    display_entries.extend(
        tree_entries
            .iter()
            .map(|tree_entry| display_entry(&tree_entry.entry)),
    );
    filename_prefixes.extend(
        tree_entries
            .iter()
            .map(|tree_entry| format!("{} ", tree_entry.branch)),
    );

    let fields = &config.display.long_format_fields;
    let widths = calculate_column_widths_with_filename_prefixes(
        &display_entries,
        &filename_prefixes,
        fields,
        config,
    );
    print_long_entries_with_filename_prefixes(
        &display_entries,
        &filename_prefixes,
        config,
        fields,
        &widths,
    );
}

fn print_tree_root(root: &FileEntry, config: &Config) {
    let root = display_entry(root);
    let filename = root.path.to_string_lossy();
    let icon = root.get_icon_custom(&config.icons);
    let filename_colored = filename.color(root.get_color(&config.colors)).bold();

    if icon.is_empty() {
        println!("{filename_colored}");
    } else {
        println!(
            "{} {filename_colored}",
            icon.color(root.get_icon_color(&config.icons.colors))
        );
    }
}

fn print_short_tree(tree_entries: &[TreeEntry], config: &Config) {
    for tree_entry in tree_entries {
        let entry = &tree_entry.entry;
        let filename = display_entry(entry).path.to_string_lossy().to_string();
        let icon = entry.get_icon_custom(&config.icons);
        let filename_colored = match entry.get_file_type() {
            FileType::Directory | FileType::Executable => {
                filename.color(entry.get_color(&config.colors)).bold()
            }
            FileType::RegularFile => filename.color(entry.get_color(&config.colors)),
        };

        if icon.is_empty() {
            println!("{} {}", tree_entry.branch, filename_colored);
        } else {
            println!(
                "{} {} {}",
                tree_entry.branch,
                icon.color(entry.get_icon_color(&config.icons.colors)),
                filename_colored
            );
        }
    }
}

fn print_long_format_with_headers(
    path: &Path,
    config: &Config,
    show_hidden: bool,
    is_root: bool,
) -> io::Result<()> {
    let mut entries = read_directory_entries(path, show_hidden, MetadataMode::Full)?;
    sort_by_type_then_name(&mut entries);

    if !is_root {
        println!("{}:", path.display());
    }

    let display_entries: Vec<FileEntry> = entries
        .iter()
        .map(|discovered_entry| display_entry(&discovered_entry.entry))
        .collect();
    if !display_entries.is_empty() {
        let fields = &config.display.long_format_fields;
        let widths = calculate_column_widths(&display_entries, fields, config);
        print_long_entries_with_widths(&display_entries, config, "", fields, &widths);
    }

    for discovered_entry in entries {
        if discovered_entry.entry.is_dir {
            print_long_format_with_headers(
                &discovered_entry.full_path,
                config,
                show_hidden,
                false,
            )?;
        }
    }

    Ok(())
}

fn sort_by_name(entries: &mut [DiscoveredEntry]) {
    entries.sort_by_cached_key(|entry| entry.entry.path.to_string_lossy().to_lowercase());
}

fn sort_by_type_then_name(entries: &mut [DiscoveredEntry]) {
    entries.sort_by_cached_key(|entry| {
        (
            entry.entry.get_file_type(),
            entry.entry.path.to_string_lossy().to_lowercase(),
        )
    });
}

fn display_entry(entry: &FileEntry) -> FileEntry {
    let mut display_entry = entry.clone();
    if display_entry.is_dir {
        display_entry.path.push("/");
    }
    display_entry
}
