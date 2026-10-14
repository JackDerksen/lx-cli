use crate::config::Config;
use crate::file_entry::{DateTimePart, DateTimePartAlignment, FileEntry};
use crate::formatter::render::Renderer;
use crate::formatter::{filename_width, print_line};
use crate::sort::{SortOptions, sort_entries};
use std::io;
use unicode_width::UnicodeWidthStr;

pub fn format_long(
    mut entries: Vec<FileEntry>,
    config: &Config,
    sort: SortOptions,
) -> io::Result<()> {
    sort_entries(&mut entries, sort);

    print_long_entries(&entries, config, "")
}

pub fn print_long_entries(entries: &[FileEntry], config: &Config, prefix: &str) -> io::Result<()> {
    if entries.is_empty() {
        return Ok(());
    }

    let fields = config.display.long_format_fields();
    let widths = calculate_column_widths(entries, &fields, config);
    print_long_entries_with_widths(entries, config, prefix, &fields, &widths)
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
            "git" => entries
                .iter()
                .filter_map(|entry| entry.git_status)
                .map(|status| UnicodeWidthStr::width(status.badge(config.icons.enabled)))
                .max()
                .unwrap_or(0),
            "nlink" => entries
                .iter()
                .map(|e| e.nlink.to_string().len())
                .max()
                .unwrap_or(0),
            "owner" => entries
                .iter()
                .map(|entry| UnicodeWidthStr::width(entry.owner.as_str()))
                .max()
                .unwrap_or(0),
            "group" => entries
                .iter()
                .map(|entry| UnicodeWidthStr::width(entry.group.as_str()))
                .max()
                .unwrap_or(0),
            "size" => entries
                .iter()
                .map(|e| e.format_size().to_string().len())
                .max()
                .unwrap_or(0),
            "modified" => modified_values
                .iter()
                .map(|value| UnicodeWidthStr::width(value.as_str()))
                .max()
                .unwrap_or(0),
            "filename" => entries
                .iter()
                .map(|entry| filename_width(entry, config))
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
) -> io::Result<()> {
    if config.display.long_format_titles {
        print_long_titles(config, prefix, fields, widths)?;
    }
    print_long_entries_with_optional_filename_prefixes(
        entries, config, prefix, fields, widths, None,
    )
}

pub fn print_long_entries_with_filename_prefixes(
    entries: &[FileEntry],
    filename_prefixes: &[String],
    config: &Config,
    fields: &[String],
    widths: &std::collections::HashMap<String, usize>,
) -> io::Result<()> {
    if config.display.long_format_titles {
        print_long_titles(config, "", fields, widths)?;
    }
    print_long_entries_with_optional_filename_prefixes(
        entries,
        config,
        "",
        fields,
        widths,
        Some(filename_prefixes),
    )
}

fn print_long_entries_with_optional_filename_prefixes(
    entries: &[FileEntry],
    config: &Config,
    prefix: &str,
    fields: &[String],
    widths: &std::collections::HashMap<String, usize>,
    filename_prefixes: Option<&[String]>,
) -> io::Result<()> {
    let modified_values = format_modified_entries(entries, &config.display.datetime_format);

    let renderer = Renderer::new(config);
    for (entry_index, entry) in entries.iter().enumerate() {
        let mut output_parts = Vec::new();
        for (index, field) in fields.iter().enumerate() {
            let mut cell = renderer.field(entry, field, &modified_values[entry_index]);
            if field == "filename" {
                let filename_prefix = filename_prefixes
                    .and_then(|prefixes| prefixes.get(entry_index))
                    .map(String::as_str)
                    .unwrap_or("");
                cell.width += UnicodeWidthStr::width(filename_prefix);
                cell.text = format!("{}{}", renderer.paint("tree", filename_prefix), cell.text);
            }
            if cell.text.is_empty() {
                continue;
            }
            let right = matches!(field.as_str(), "size" | "nlink");
            let width = if right || index < fields.len() - 1 {
                widths.get(field).copied().unwrap_or(0)
            } else {
                cell.width
            };
            output_parts.push(cell.padded(width, right));
        }
        print_line(format_args!(
            "{}{}",
            renderer.paint("tree", prefix),
            output_parts.join(&" ".repeat(config.display.column_spacing))
        ))?;
    }
    Ok(())
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
) -> io::Result<()> {
    let titles: Vec<String> = fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let title = column_title(field);
            let width = widths.get(field).copied().unwrap_or(0);
            let padding = if index < fields.len() - 1 {
                width.saturating_sub(UnicodeWidthStr::width(title))
            } else {
                0
            };

            format!(
                "{}{}",
                config.theme.style("header").paint(title),
                " ".repeat(padding)
            )
        })
        .collect();

    print_line(format_args!(
        "{}{}",
        config.theme.style("tree").paint(prefix),
        titles.join(&" ".repeat(config.display.column_spacing))
    ))
}

fn column_title(field: &str) -> &str {
    match field {
        "git" => "Git",
        "permissions" => "Permissions",
        "nlink" => "Links",
        "owner" => "Owner",
        "group" => "Group",
        "size" => "Size",
        "modified" => "Modified",
        "filename" => "Name",
        _ => field,
    }
}

fn pad_to_display_width(value: String, width: usize) -> String {
    let padding = width.saturating_sub(UnicodeWidthStr::width(value.as_str()));
    format!("{value}{}", " ".repeat(padding))
}
