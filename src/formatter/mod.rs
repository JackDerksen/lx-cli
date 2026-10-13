mod long;
mod one_per_line;
mod recursive;
mod short;
mod tree;

pub use long::format_long;
pub use one_per_line::{format_one_per_line, format_plain};
pub use recursive::{TreeOptions, format_recursive};
pub use short::{format_short, format_short_compact};

use crate::config::{Config, FontStyle, apply_font_styles};
use crate::file_entry::FileEntry;
use colored::Colorize;
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;

fn format_filename(
    entry: &FileEntry,
    config: &Config,
    padding: usize,
    styles: Option<&[FontStyle]>,
) -> String {
    let filename = entry.display_name().color(entry.get_color(&config.colors));
    let filename = apply_font_styles(
        filename,
        styles.unwrap_or_else(|| entry.font_styles(&config.styles)),
    )
    .to_string();
    let icon = entry.get_icon_custom(&config.icons);
    let mut output = if icon.is_empty() {
        filename
    } else {
        format!(
            "{} {filename}",
            icon.color(entry.get_icon_color(&config.icons.colors))
        )
    };
    output.extend(std::iter::repeat_n(' ', padding));
    output
}

fn format_short_entry(entry: &FileEntry, config: &Config) -> String {
    let output = format_filename(entry, config, 0, None);
    match entry.git_status {
        Some(status) => format!("{} {output}", status.format(1, &config.colors.git)),
        None => output,
    }
}

fn entry_width(entry: &FileEntry, config: &Config) -> usize {
    filename_width(entry, config) + if entry.git_status.is_some() { 2 } else { 0 }
}

fn filename_width(entry: &FileEntry, config: &Config) -> usize {
    let icon = entry.get_icon_custom(&config.icons);
    UnicodeWidthStr::width(entry.display_name().as_ref())
        + UnicodeWidthStr::width(icon.as_str())
        + usize::from(!icon.is_empty())
}

fn print_line(arguments: std::fmt::Arguments<'_>) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_fmt(arguments)?;
    stdout.write_all(b"\n")
}
