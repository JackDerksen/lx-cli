use crate::config::{Config, LongField};
use crate::file_entry::{DateTimePart, DateTimePartAlignment, FileEntry};
use crate::formatter::render::Renderer;
use crate::formatter::{filename_width, print_line};
use crate::sort::{SortOptions, sort_entries};
use chrono::format::Item;
use std::collections::HashMap;
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

    let layout = LongLayout::new(entries, None, config);
    if config.display.long_format_titles {
        print_long_titles(config, prefix, &layout)?;
    }
    print_long_rows(entries, config, prefix, &layout, None)
}

/// Long-format rows with tree branches drawn inside the Name column.
pub fn print_long_entries_with_filename_prefixes(
    entries: &[FileEntry],
    filename_prefixes: &[String],
    config: &Config,
) -> io::Result<()> {
    let layout = LongLayout::new(entries, Some(filename_prefixes), config);
    if config.display.long_format_titles {
        print_long_titles(config, "", &layout)?;
    }
    print_long_rows(entries, config, "", &layout, Some(filename_prefixes))
}

/// Columns, their widths, and each entry's formatted date, computed once per listing.
struct LongLayout<'config> {
    fields: &'config [LongField],
    widths: HashMap<LongField, usize>,
    modified: Vec<String>,
}

impl<'config> LongLayout<'config> {
    fn new(
        entries: &[FileEntry],
        filename_prefixes: Option<&[String]>,
        config: &'config Config,
    ) -> Self {
        let fields = config.display.long_format_fields.as_slice();
        let modified = if fields.contains(&LongField::Modified) {
            format_modified_entries(entries, &config.display.datetime_items())
        } else {
            vec![String::new(); entries.len()]
        };
        let mut widths = HashMap::new();
        for &field in fields {
            let width = match field {
                LongField::Git => entries
                    .iter()
                    .filter_map(|entry| entry.git_status)
                    .map(|status| UnicodeWidthStr::width(status.badge(config.icons.enabled)))
                    .max()
                    .unwrap_or(0),
                LongField::Nlink => entries
                    .iter()
                    .map(|e| e.nlink.to_string().len())
                    .max()
                    .unwrap_or(0),
                LongField::Owner => entries
                    .iter()
                    .map(|entry| UnicodeWidthStr::width(entry.owner.as_str()))
                    .max()
                    .unwrap_or(0),
                LongField::Group => entries
                    .iter()
                    .map(|entry| UnicodeWidthStr::width(entry.group.as_str()))
                    .max()
                    .unwrap_or(0),
                LongField::Size => entries
                    .iter()
                    .map(|e| e.format_size().to_string().len())
                    .max()
                    .unwrap_or(0),
                LongField::Modified => modified
                    .iter()
                    .map(|value| UnicodeWidthStr::width(value.as_str()))
                    .max()
                    .unwrap_or(0),
                LongField::Filename => entries
                    .iter()
                    .enumerate()
                    .map(|(index, entry)| {
                        filename_width(entry, config)
                            + filename_prefixes
                                .and_then(|prefixes| prefixes.get(index))
                                .map_or(0, |prefix| UnicodeWidthStr::width(prefix.as_str()))
                    })
                    .max()
                    .unwrap_or(0),
                LongField::Permissions => entries
                    .iter()
                    .map(|e| e.format_permissions().len())
                    .max()
                    .unwrap_or(0),
            };
            let title_width = if config.display.long_format_titles {
                UnicodeWidthStr::width(field.title())
            } else {
                0
            };
            widths.insert(field, width.max(title_width));
        }
        Self {
            fields,
            widths,
            modified,
        }
    }

    fn width(&self, field: LongField) -> usize {
        self.widths.get(&field).copied().unwrap_or(0)
    }
}

fn print_long_rows(
    entries: &[FileEntry],
    config: &Config,
    prefix: &str,
    layout: &LongLayout<'_>,
    filename_prefixes: Option<&[String]>,
) -> io::Result<()> {
    let fields = layout.fields;
    let renderer = Renderer::new(config);
    for (entry_index, entry) in entries.iter().enumerate() {
        let mut output_parts = Vec::new();
        for (index, &field) in fields.iter().enumerate() {
            let mut cell = renderer.field(entry, field, &layout.modified[entry_index]);
            if field == LongField::Filename {
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
            let right = matches!(field, LongField::Size | LongField::Nlink);
            let width = if right || index < fields.len() - 1 {
                layout.width(field)
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

fn format_modified_entries(entries: &[FileEntry], datetime_items: &[Item<'_>]) -> Vec<String> {
    let parts: Vec<Vec<DateTimePart>> = entries
        .iter()
        .map(|entry| entry.format_modified_parts(datetime_items))
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

fn print_long_titles(config: &Config, prefix: &str, layout: &LongLayout<'_>) -> io::Result<()> {
    let fields = layout.fields;
    let titles: Vec<String> = fields
        .iter()
        .enumerate()
        .map(|(index, &field)| {
            let title = field.title();
            let padding = if index < fields.len() - 1 {
                layout
                    .width(field)
                    .saturating_sub(UnicodeWidthStr::width(title))
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

fn pad_to_display_width(value: String, width: usize) -> String {
    let padding = width.saturating_sub(UnicodeWidthStr::width(value.as_str()));
    format!("{value}{}", " ".repeat(padding))
}
