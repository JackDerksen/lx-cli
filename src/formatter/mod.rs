mod long;
mod one_per_line;
mod recursive;
mod short;
mod tree;

pub use long::format_long;
pub use one_per_line::{format_one_per_line, format_plain};
pub use recursive::{TreeOptions, format_recursive};
pub use short::{format_short, format_short_compact};

use crate::config::Config;
use crate::file_entry::{FileEntry, FileType};
use colored::Colorize;
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;

fn format_filename(entry: &FileEntry, config: &Config, padding: usize) -> String {
    let mut filename = entry.display_name().into_owned();
    filename.extend(std::iter::repeat_n(' ', padding));
    let filename = filename.color(entry.get_color(&config.colors));
    match entry.get_file_type() {
        FileType::Directory | FileType::Executable => filename.bold().to_string(),
        FileType::RegularFile | FileType::Symlink => filename.to_string(),
    }
}

fn format_short_entry(entry: &FileEntry, config: &Config) -> String {
    let filename = format_filename(entry, config, 0);
    let icon = entry.get_icon_custom(&config.icons);
    let output = if icon.is_empty() {
        filename
    } else {
        format!(
            "{} {filename}",
            icon.color(entry.get_icon_color(&config.icons.colors))
        )
    };
    match entry.git_status {
        Some(status) => format!("{} {output}", status.format(1, &config.colors.git)),
        None => output,
    }
}

fn entry_width(entry: &FileEntry, config: &Config) -> usize {
    let icon = entry.get_icon_custom(&config.icons);
    UnicodeWidthStr::width(entry.display_name().as_ref())
        + UnicodeWidthStr::width(icon.as_str())
        + usize::from(!icon.is_empty())
        + if entry.git_status.is_some() { 2 } else { 0 }
}

fn print_line(arguments: std::fmt::Arguments<'_>) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_fmt(arguments)?;
    stdout.write_all(b"\n")
}
