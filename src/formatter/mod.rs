mod long;
mod one_per_line;
mod recursive;
mod render;
mod short;
mod tree;

pub use long::format_long;
pub use one_per_line::{format_one_per_line, format_plain};
pub use recursive::{TreeOptions, format_recursive};
pub use short::{format_short, format_short_compact};

use crate::config::Config;
use crate::file_entry::FileEntry;
use render::Renderer;
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;

fn format_short_entry(entry: &FileEntry, config: &Config) -> String {
    let renderer = Renderer::new(config);
    let output = renderer.filename(entry, false).text;
    match entry.git_status {
        Some(status) => format!("{} {output}", renderer.git(status, false).text),
        None => output,
    }
}

fn entry_width(entry: &FileEntry, config: &Config) -> usize {
    filename_width(entry, config)
        + entry.git_status.map_or(0, |status| {
            UnicodeWidthStr::width(status.badge(config.icons.enabled)) + 1
        })
}

fn filename_width(entry: &FileEntry, config: &Config) -> usize {
    let icon = entry.get_icon_custom(&config.icons);
    UnicodeWidthStr::width(entry.display_name().as_ref())
        + UnicodeWidthStr::width(icon.as_str())
        + usize::from(!icon.is_empty())
}

fn print_line(out: &mut dyn Write, arguments: std::fmt::Arguments<'_>) -> io::Result<()> {
    out.write_fmt(arguments)?;
    out.write_all(b"\n")
}
