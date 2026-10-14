use crate::config::{Config, LongField};
use crate::file_entry::FileEntry;
use crate::formatter::render::{Renderer, push_spaces};
use crate::formatter::{filename_width, print_line};
use crate::sort::{SortOptions, sort_entries};
use chrono::format::Item;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;

pub fn format_long(
    out: &mut dyn Write,
    mut entries: Vec<FileEntry>,
    config: &Config,
    sort: SortOptions,
) -> io::Result<()> {
    sort_entries(&mut entries, sort);
    if entries.is_empty() {
        return Ok(());
    }
    print_long_entries(out, &entries, None, config)
}

/// Prints titles and rows; tree branches, when given, are drawn inside the Name column.
pub fn print_long_entries(
    out: &mut dyn Write,
    entries: &[FileEntry],
    filename_prefixes: Option<&[String]>,
    config: &Config,
) -> io::Result<()> {
    let layout = LongLayout::new(entries, filename_prefixes, config);
    if config.display.long_format_titles {
        print_long_titles(out, config, &layout)?;
    }
    print_long_rows(out, entries, config, &layout, filename_prefixes)
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
    out: &mut dyn Write,
    entries: &[FileEntry],
    config: &Config,
    layout: &LongLayout<'_>,
    filename_prefixes: Option<&[String]>,
) -> io::Result<()> {
    let fields = layout.fields;
    let renderer = Renderer::new(config);
    let tree_style = config.theme.style("tree");
    let separator = " ".repeat(config.display.column_spacing);
    // One buffer reused for every row.
    let mut line = String::new();
    for (entry_index, entry) in entries.iter().enumerate() {
        line.clear();
        let mut first = true;
        for (index, &field) in fields.iter().enumerate() {
            let mut cell = renderer.field(entry, field, &layout.modified[entry_index]);
            let filename_prefix = match field {
                LongField::Filename => filename_prefixes
                    .and_then(|prefixes| prefixes.get(entry_index))
                    .map_or("", String::as_str),
                _ => "",
            };
            if cell.text.is_empty() && filename_prefix.is_empty() {
                continue;
            }
            if !first {
                line.push_str(&separator);
            }
            first = false;
            if !filename_prefix.is_empty() {
                let mut text = String::with_capacity(filename_prefix.len() + cell.text.len() + 16);
                tree_style.paint_into(&mut text, filename_prefix);
                text.push_str(&cell.text);
                cell.text = text;
                cell.width += UnicodeWidthStr::width(filename_prefix);
            }
            let right = matches!(field, LongField::Size | LongField::Nlink);
            let width = if right || index < fields.len() - 1 {
                layout.width(field)
            } else {
                cell.width
            };
            cell.write_padded(&mut line, width, right);
        }
        line.push('\n');
        out.write_all(line.as_bytes())?;
    }
    Ok(())
}

/// Formats every entry's date, padding each numeric or text component to a shared width.
fn format_modified_entries(entries: &[FileEntry], datetime_items: &[Item<'_>]) -> Vec<String> {
    #[derive(Clone, Copy, PartialEq)]
    enum Alignment {
        Left,
        Right,
        None,
    }
    let alignments: Vec<Alignment> = datetime_items
        .iter()
        .map(|item| match item {
            Item::Numeric(_, _) => Alignment::Right,
            Item::Fixed(_) => Alignment::Left,
            _ => Alignment::None,
        })
        .collect();

    // All components go into one buffer; `ends` marks where each finishes.
    let mut text = String::new();
    let mut ends = Vec::with_capacity(entries.len() * datetime_items.len());
    for entry in entries {
        let datetime: chrono::DateTime<chrono::Local> = entry.modified.into();
        for item in datetime_items {
            let _ = write!(
                text,
                "{}",
                datetime.format_with_items(std::iter::once(item))
            );
            ends.push(text.len());
        }
    }
    let component = |index: usize| {
        let start = index.checked_sub(1).map_or(0, |previous| ends[previous]);
        &text[start..ends[index]]
    };

    let mut widths = vec![0; datetime_items.len()];
    for index in 0..ends.len() {
        let column = index % datetime_items.len();
        if alignments[column] != Alignment::None {
            widths[column] = widths[column].max(UnicodeWidthStr::width(component(index)));
        }
    }

    (0..entries.len())
        .map(|entry_index| {
            let mut formatted = String::new();
            for column in 0..datetime_items.len() {
                let value = component(entry_index * datetime_items.len() + column);
                let padding = widths[column].saturating_sub(UnicodeWidthStr::width(value));
                match alignments[column] {
                    Alignment::Left => {
                        formatted.push_str(value);
                        push_spaces(&mut formatted, padding);
                    }
                    Alignment::Right => {
                        push_spaces(&mut formatted, padding);
                        formatted.push_str(value);
                    }
                    Alignment::None => formatted.push_str(value),
                }
            }
            formatted
        })
        .collect()
}

fn print_long_titles(
    out: &mut dyn Write,
    config: &Config,
    layout: &LongLayout<'_>,
) -> io::Result<()> {
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

    print_line(
        out,
        format_args!(
            "{}",
            titles.join(&" ".repeat(config.display.column_spacing))
        ),
    )
}
