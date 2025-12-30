/// Handles sorting of file entries
use crate::file_entry::FileEntry;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Name,
    Size,
    Date,
    Type,
}

impl SortField {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "size" => SortField::Size,
            "date" => SortField::Date,
            "type" => SortField::Type,
            _ => SortField::Name, // Default to name
        }
    }
}

pub fn sort_entries(entries: &mut Vec<FileEntry>, sort_field: SortField) {
    match sort_field {
        SortField::Name => sort_by_name(entries),
        SortField::Size => sort_by_size(entries),
        SortField::Date => sort_by_date(entries),
        SortField::Type => sort_by_type(entries),
    }
}

fn sort_by_name(entries: &mut Vec<FileEntry>) {
    entries.sort_by(|a, b| {
        let a_name = a.path.to_string_lossy();
        let b_name = b.path.to_string_lossy();
        a_name.cmp(&b_name)
    });
}

fn sort_by_size(entries: &mut Vec<FileEntry>) {
    entries.sort_by(|a, b| {
        // Sort by size in descending order (larger first)
        b.size.cmp(&a.size)
    });
}

fn sort_by_date(entries: &mut Vec<FileEntry>) {
    entries.sort_by(|a, b| {
        // Sort by modification date in descending order (newest first)
        match b.modified.cmp(&a.modified) {
            Ordering::Equal => {
                let a_name = a.path.to_string_lossy();
                let b_name = b.path.to_string_lossy();
                a_name.cmp(&b_name)
            }
            other => other,
        }
    });
}

fn sort_by_type(entries: &mut Vec<FileEntry>) {
    entries.sort_by(|a, b| {
        // Sort by file type (directories, executables, regular files)
        match a.get_file_type().cmp(&b.get_file_type()) {
            Ordering::Equal => {
                // Within same type, sort by name
                let a_name = a.path.to_string_lossy();
                let b_name = b.path.to_string_lossy();
                a_name.cmp(&b_name)
            }
            other => other,
        }
    });
}
