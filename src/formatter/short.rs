use crate::config::Config;
use crate::file_entry::{FileEntry, FileType};
use crate::formatter::{entry_width, format_short_entry, print_line};
use crate::sort::{SortOptions, sort_entries};
use std::io::{self, Write};

pub fn format_short(
    out: &mut dyn Write,
    entries: Vec<FileEntry>,
    config: &Config,
    sort: SortOptions,
) -> io::Result<()> {
    format_short_with_max_rows(out, entries, config, config.display.max_rows, sort)
}

pub fn format_short_compact(
    out: &mut dyn Write,
    entries: Vec<FileEntry>,
    config: &Config,
    sort: SortOptions,
) -> io::Result<()> {
    format_short_with_max_rows(out, entries, config, config.display.compact_max_rows, sort)
}

fn format_short_with_max_rows(
    out: &mut dyn Write,
    mut entries: Vec<FileEntry>,
    config: &Config,
    max_rows: usize,
    sort: SortOptions,
) -> io::Result<()> {
    sort_entries(&mut entries, sort);

    if sort.is_custom() {
        let rows = if max_rows == 0 {
            entries.len().div_ceil(3).max(1)
        } else {
            max_rows
        };
        return format_sorted_entries_with_max_rows(
            out,
            entries,
            rows,
            config.display.column_spacing,
            config,
        );
    }

    let mut directories: Vec<FileEntry> = Vec::new();
    let mut executables: Vec<FileEntry> = Vec::new();
    let mut regular_files: Vec<FileEntry> = Vec::new();

    for entry in entries {
        match entry.get_file_type() {
            FileType::Directory => directories.push(entry),
            FileType::Executable => executables.push(entry),
            FileType::RegularFile | FileType::Symlink => regular_files.push(entry),
        }
    }

    let rows = if max_rows == 0 {
        directories
            .len()
            .max(executables.len())
            .max(regular_files.len())
    } else {
        max_rows
    };
    if rows == 0 {
        return Ok(());
    }
    format_with_max_rows(
        out,
        directories,
        executables,
        regular_files,
        rows,
        config.display.column_spacing,
        config,
    )
}

fn format_sorted_entries_with_max_rows(
    out: &mut dyn Write,
    entries: Vec<FileEntry>,
    max_rows: usize,
    column_spacing: usize,
    config: &Config,
) -> io::Result<()> {
    let widths = compact_column_widths(&entries, max_rows, config);

    for row in 0..max_rows {
        let mut line = String::new();
        let mut has_content = false;

        for (column, width) in widths.iter().enumerate() {
            if column > 0 {
                line.push_str(&" ".repeat(column_spacing));
            }

            if let Some(entry) = entries.get(column * max_rows + row) {
                line.push_str(&format_short_entry(entry, config));
                let actual_width = entry_width(entry, config);
                line.push_str(&" ".repeat(width - actual_width));
                has_content = true;
            } else {
                line.push_str(&" ".repeat(*width));
            }
        }

        if has_content {
            print_line(out, format_args!("{}", line.trim_end()))?;
        }
    }
    Ok(())
}

fn format_with_max_rows(
    out: &mut dyn Write,
    directories: Vec<FileEntry>,
    executables: Vec<FileEntry>,
    regular_files: Vec<FileEntry>,
    max_rows: usize,
    column_spacing: usize,
    config: &Config,
) -> io::Result<()> {
    let directory_widths = compact_column_widths(&directories, max_rows, config);
    let executable_widths = compact_column_widths(&executables, max_rows, config);
    let file_widths = compact_column_widths(&regular_files, max_rows, config);
    let sections = [
        (&directories, directory_widths),
        (&executables, executable_widths),
        (&regular_files, file_widths),
    ];

    // Print rows, with all file types side-by-side
    for row in 0..max_rows {
        let mut line = String::new();
        let mut has_any_content = false;
        let mut has_previous_section = false;

        for (entries, widths) in &sections {
            if widths.is_empty() {
                continue;
            }

            if has_previous_section {
                line.push_str(&" ".repeat(column_spacing));
            }
            has_previous_section = true;
            has_any_content |= append_compact_section(
                &mut line,
                entries,
                widths,
                row,
                max_rows,
                column_spacing,
                config,
            );
        }

        if has_any_content {
            print_line(out, format_args!("{}", line.trim_end()))?;
        }
    }
    Ok(())
}

fn compact_column_widths(entries: &[FileEntry], max_rows: usize, config: &Config) -> Vec<usize> {
    entries
        .chunks(max_rows)
        .map(|column| {
            column
                .iter()
                .map(|entry| entry_width(entry, config))
                .max()
                .unwrap_or(0)
        })
        .collect()
}

fn append_compact_section(
    line: &mut String,
    entries: &[FileEntry],
    widths: &[usize],
    row: usize,
    max_rows: usize,
    column_spacing: usize,
    config: &Config,
) -> bool {
    let mut has_content = false;

    for (column, width) in widths.iter().enumerate() {
        if column > 0 {
            line.push_str(&" ".repeat(column_spacing));
        }

        if let Some(entry) = entries.get(column * max_rows + row) {
            line.push_str(&format_short_entry(entry, config));
            has_content = true;
        }

        let actual_width = entries
            .get(column * max_rows + row)
            .map(|entry| entry_width(entry, config))
            .unwrap_or(0);
        line.push_str(&" ".repeat(width - actual_width));
    }

    has_content
}
