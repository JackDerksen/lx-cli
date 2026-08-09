use crate::config::Config;
use crate::file_entry::{DateTimePart, DateTimePartAlignment, FileEntry, FileType};
use crate::sort::{SortOptions, sort_entries};
use colored::Colorize;
use unicode_width::UnicodeWidthStr;

pub fn format_long(mut entries: Vec<FileEntry>, config: &Config, sort: SortOptions) {
    sort_entries(&mut entries, sort);

    print_long_entries(&entries, config, "");
}

pub fn print_long_entries(entries: &[FileEntry], config: &Config, prefix: &str) {
    if entries.is_empty() {
        return;
    }

    let fields = &config.display.long_format_fields;
    let widths = calculate_column_widths(entries, fields, config);
    print_long_entries_with_widths(entries, config, prefix, fields, &widths);
}

pub fn calculate_column_widths(
    entries: &[FileEntry],
    fields: &[String],
    config: &Config,
) -> std::collections::HashMap<String, usize> {
    let mut max_widths: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let modified_values = format_modified_entries(entries, &config.display.datetime_format);

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
            "modified" => modified_values
                .iter()
                .map(|value| UnicodeWidthStr::width(value.as_str()))
                .max()
                .unwrap_or(0),
            "icon" => entries
                .iter()
                .map(|e| UnicodeWidthStr::width(e.get_icon_custom(&config.icons).as_str()))
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
        let title_width = if config.display.long_format_titles {
            UnicodeWidthStr::width(column_title(field))
        } else {
            0
        };
        max_widths.insert(field.clone(), width.max(title_width));
    }

    max_widths
}

pub fn calculate_column_widths_with_filename_prefixes(
    entries: &[FileEntry],
    filename_prefixes: &[String],
    fields: &[String],
    config: &Config,
) -> std::collections::HashMap<String, usize> {
    let mut entries_with_prefixes = entries.to_vec();

    for (entry, filename_prefix) in entries_with_prefixes.iter_mut().zip(filename_prefixes) {
        entry.path = format!("{filename_prefix}{}", entry.path.to_string_lossy()).into();
    }

    calculate_column_widths(&entries_with_prefixes, fields, config)
}

pub fn print_long_entries_with_widths(
    entries: &[FileEntry],
    config: &Config,
    prefix: &str,
    fields: &[String],
    widths: &std::collections::HashMap<String, usize>,
) {
    if config.display.long_format_titles {
        print_long_titles(config, prefix, fields, widths);
    }
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
    if config.display.long_format_titles {
        print_long_titles(config, "", fields, widths);
    }
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
    let modified_values = format_modified_entries(entries, &config.display.datetime_format);

    // Print each entry
    for (entry_index, entry) in entries.iter().enumerate() {
        let mut output_parts: Vec<String> = Vec::new();

        for (idx, field) in fields.iter().enumerate() {
            let part = match field.as_str() {
                "permissions" => {
                    let width = widths.get("permissions").copied().unwrap_or(0);
                    let permissions = entry.format_permissions();
                    if idx < fields.len() - 1 {
                        pad_to_display_width(permissions, width)
                    } else {
                        permissions
                    }
                }
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
                "modified" => {
                    let width = widths.get("modified").copied().unwrap_or(0);
                    let modified = &modified_values[entry_index];
                    if idx < fields.len() - 1 {
                        pad_to_display_width(modified.to_string(), width)
                    } else {
                        modified.to_string()
                    }
                }
                "icon" => {
                    let icon = entry.get_icon_custom(&config.icons);
                    let icon_color = entry.get_icon_color(&config.icons.colors);
                    let width = widths.get("icon").copied().unwrap_or(0);
                    let padded = if idx < fields.len() - 1 {
                        pad_to_display_width(icon, width)
                    } else {
                        icon
                    };
                    format!("{}", padded.color(icon_color))
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

fn format_modified_entries(entries: &[FileEntry], datetime_format: &str) -> Vec<String> {
    let parts: Vec<Vec<DateTimePart>> = entries
        .iter()
        .map(|entry| entry.format_modified_parts(datetime_format))
        .collect();
    let mut widths = vec![0; parts.iter().map(Vec::len).max().unwrap_or(0)];

    for entry_parts in &parts {
        for (index, part) in entry_parts.iter().enumerate() {
            if part.alignment == DateTimePartAlignment::None {
                continue;
            }

            let width = UnicodeWidthStr::width(part.value.as_str());
            widths[index] = widths[index].max(width);
        }
    }

    parts
        .into_iter()
        .map(|entry_parts| {
            entry_parts
                .iter()
                .enumerate()
                .map(|(index, part)| format_datetime_part(part, widths[index]))
                .collect()
        })
        .collect()
}

fn format_datetime_part(part: &DateTimePart, width: usize) -> String {
    match part.alignment {
        DateTimePartAlignment::Left => pad_to_display_width(part.value.clone(), width),
        DateTimePartAlignment::Right => format!(
            "{}{}",
            " ".repeat(width.saturating_sub(UnicodeWidthStr::width(part.value.as_str()))),
            part.value
        ),
        DateTimePartAlignment::None => part.value.clone(),
    }
}

fn print_long_titles(
    config: &Config,
    prefix: &str,
    fields: &[String],
    widths: &std::collections::HashMap<String, usize>,
) {
    let titles: Vec<String> = fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let title = column_title(field);
            let width = widths.get(field).copied().unwrap_or(0);
            let padded = if index < fields.len() - 1 {
                pad_to_display_width(title.to_string(), width)
            } else {
                title.to_string()
            };

            format!(
                "{}",
                padded.color(config.display.get_long_format_title_color())
            )
        })
        .collect();

    println!("{}{}", prefix, titles.join("  "));
}

fn column_title(field: &str) -> &str {
    match field {
        "permissions" => "Permissions",
        "nlink" => "Links",
        "owner" => "Owner",
        "group" => "Group",
        "size" => "Size",
        "modified" => "Modified",
        "icon" => "Icon",
        "filename" => "Name",
        _ => field,
    }
}

fn pad_to_display_width(value: String, width: usize) -> String {
    let padding = width.saturating_sub(UnicodeWidthStr::width(value.as_str()));
    format!("{value}{}", " ".repeat(padding))
}
