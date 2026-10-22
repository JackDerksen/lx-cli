use crate::config::Config;
use crate::file_entry::FileEntry;
use crate::formatter::{format_short_entry, print_line};
use crate::sort::{SortOptions, sort_entries};
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub fn format_one_per_line(
    out: &mut dyn Write,
    mut entries: Vec<FileEntry>,
    config: &Config,
    sort: SortOptions,
) -> io::Result<()> {
    sort_entries(&mut entries, sort);
    for entry in &entries {
        print_line(out, format_args!("{}", format_short_entry(entry, config)))?;
    }
    Ok(())
}

/// Writes raw filename bytes without styling, separated by newline or NUL.
pub fn format_plain(
    out: &mut dyn Write,
    mut entries: Vec<FileEntry>,
    target: &Path,
    sort: SortOptions,
    zero: bool,
) -> io::Result<()> {
    sort_entries(&mut entries, sort);
    let is_directory = target.is_dir() && !target.is_symlink();
    let separator = if zero { b"\0" } else { b"\n" };
    for entry in entries {
        let name = if is_directory {
            entry.path.as_os_str()
        } else {
            target.as_os_str()
        };
        out.write_all(name.as_bytes())?;
        out.write_all(separator)?;
    }
    Ok(())
}
