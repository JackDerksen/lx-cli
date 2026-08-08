use crate::config::Config;
use crate::file_entry::{FileEntry, FileType};
use crate::sort::sort_default;
use colored::Colorize;
use unicode_width::UnicodeWidthStr;

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
                .map(|e| UnicodeWidthStr::width(e.path.to_string_lossy().as_ref()))
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

pub fn calculate_column_widths_with_filename_prefixes(
    entries: &[FileEntry],
    filename_prefixes: &[String],
    fields: &[String],
) -> std::collections::HashMap<String, usize> {
    let mut entries_with_prefixes = entries.to_vec();

    for (entry, filename_prefix) in entries_with_prefixes.iter_mut().zip(filename_prefixes) {
        entry.path = format!("{filename_prefix}{}", entry.path.to_string_lossy()).into();
    }

    calculate_column_widths(&entries_with_prefixes, fields)
}

pub fn print_long_entries_with_widths(
    entries: &[FileEntry],
    config: &Config,
    prefix: &str,
    fields: &[String],
    widths: &std::collections::HashMap<String, usize>,
) {
    print_long_entries_with_optional_filename_prefixes(
        entries, config, prefix, fields, widths, None,
    );
}

pub fn print_long_entries_with_filename_prefixes(
    entries: &[FileEntry],
    filename_prefixes: &[String],
    config: &Config,
    fields: &[String],
    widths: &std::collections::HashMap<String, usize>,
) {
    print_long_entries_with_optional_filename_prefixes(
        entries,
        config,
        "",
        fields,
        widths,
        Some(filename_prefixes),
    );
}

fn print_long_entries_with_optional_filename_prefixes(
    entries: &[FileEntry],
    config: &Config,
    prefix: &str,
    fields: &[String],
    widths: &std::collections::HashMap<String, usize>,
    filename_prefixes: Option<&[String]>,
) {
    // Print each entry
    for (entry_index, entry) in entries.iter().enumerate() {
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
                    if icon.is_empty() {
                        String::new()
                    } else {
                        format!("{}", icon.color(icon_color))
                    }
                }
                "filename" => {
                    let filename_str = entry.path.to_string_lossy().to_string();
                    let width = widths.get("filename").copied().unwrap_or(0);
                    let filename_prefix = filename_prefixes
                        .and_then(|prefixes| prefixes.get(entry_index))
                        .map(String::as_str)
                        .unwrap_or("");

                    let padding = if idx < fields.len() - 1 {
                        width.saturating_sub(
                            UnicodeWidthStr::width(filename_prefix)
                                + UnicodeWidthStr::width(filename_str.as_str()),
                        )
                    } else {
                        0
                    };
                    let padded = format!("{filename_str}{}", " ".repeat(padding));

                    let filename_colored = match entry.get_file_type() {
                        FileType::Directory | FileType::Executable => {
                            padded.color(entry.get_color(&config.colors)).bold()
                        }
                        FileType::RegularFile => padded.color(entry.get_color(&config.colors)),
                    };
                    format!("{filename_prefix}{filename_colored}")
                }
                _ => String::new(),
            };
            output_parts.push(part);
        }

        let visible_parts: Vec<&str> = output_parts
            .iter()
            .map(|part| part.as_str())
            .filter(|part| !part.is_empty())
            .collect();
        println!("{}{}", prefix, visible_parts.join("  "));
    }
}
