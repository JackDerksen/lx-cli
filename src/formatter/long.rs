use crate::config::Config;
use crate::file_entry::{FileEntry, FileType};
use crate::sort::sort_default;
use colored::Colorize;

pub fn format_long(mut entries: Vec<FileEntry>, config: &Config) {
    // Apply default sorting: by type, then alphabetically (case-insensitive)
    sort_default(&mut entries);

    print_long_entries(&entries, config, "");
}

pub fn print_long_entries(entries: &[FileEntry], config: &Config, prefix: &str) {
    if entries.is_empty() {
        return;
    }

    let fields = &config.display.long_format_fields;
    let widths = calculate_column_widths(entries, fields);
    print_long_entries_with_widths(entries, config, prefix, fields, &widths);
}

pub fn calculate_column_widths(
    entries: &[FileEntry],
    fields: &[String],
) -> std::collections::HashMap<String, usize> {
    let mut max_widths: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for field in fields {
        let width = match field.as_str() {
            "nlink" => entries
                .iter()
                .map(|e| e.nlink.to_string().len())
                .max()
                .unwrap_or(0),
            "owner" => entries.iter().map(|e| e.owner.len()).max().unwrap_or(0),
            "group" => entries.iter().map(|e| e.group.len()).max().unwrap_or(0),
            "size" => entries
                .iter()
                .map(|e| e.format_size().len())
                .max()
                .unwrap_or(0),
            "filename" => entries
                .iter()
                .map(|e| e.path.to_string_lossy().len())
                .max()
                .unwrap_or(0),
            "permissions" => entries
                .iter()
                .map(|e| e.format_permissions().len())
                .max()
                .unwrap_or(0),
            _ => 0,
        };
        max_widths.insert(field.clone(), width);
    }

    max_widths
}

pub fn print_long_entries_with_widths(
    entries: &[FileEntry],
    config: &Config,
    prefix: &str,
    fields: &[String],
    widths: &std::collections::HashMap<String, usize>,
) {
    // Print each entry
    for entry in entries {
        let mut output_parts: Vec<String> = Vec::new();

        for (idx, field) in fields.iter().enumerate() {
            let part = match field.as_str() {
                "permissions" => entry.format_permissions(),
                "nlink" => {
                    let width = widths.get("nlink").copied().unwrap_or(0);
                    format!("{:>width$}", entry.nlink.to_string(), width = width)
                }
                "owner" => {
                    let width = widths.get("owner").copied().unwrap_or(0);
                    format!("{:<width$}", entry.owner, width = width)
                }
                "group" => {
                    let width = widths.get("group").copied().unwrap_or(0);
                    format!("{:<width$}", entry.group, width = width)
                }
                "size" => {
                    let width = widths.get("size").copied().unwrap_or(0);
                    format!("{:>width$}", entry.format_size(), width = width)
                }
                "modified" => entry.format_modified(),
                "icon" => {
                    let icon = entry.get_icon_custom(&config.icons);
                    let icon_color = entry.get_icon_color(&config.icons.colors);
                    format!("{}", icon.color(icon_color))
                }
                "filename" => {
                    let filename_str = entry.path.to_string_lossy().to_string();
                    let width = widths.get("filename").copied().unwrap_or(0);

                    // Pad filename before applying color
                    let padded = if idx < fields.len() - 1 {
                        format!("{:<width$}", filename_str, width = width)
                    } else {
                        filename_str
                    };

                    let filename_colored = match entry.get_file_type() {
                        FileType::Directory | FileType::Executable => {
                            padded.color(entry.get_color(&config.colors)).bold()
                        }
                        FileType::RegularFile => padded.color(entry.get_color(&config.colors)),
                    };
                    format!("{}", filename_colored)
                }
                _ => String::new(),
            };
            output_parts.push(part);
        }

        println!("{}{}", prefix, output_parts.join("  "));
    }
}
