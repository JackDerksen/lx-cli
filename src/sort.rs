/// Handles sorting of file entries
use crate::file_entry::FileEntry;
use std::cmp::Ordering;

/// Default sort: by file type (directory, executable, regular), then alphabetically by name (case-insensitive)
pub fn sort_default(entries: &mut Vec<FileEntry>) {
    entries.sort_by(|a, b| {
        // First sort by type to maintain grouping
        match a.get_file_type().cmp(&b.get_file_type()) {
            Ordering::Equal => {
                // Within same type, sort alphabetically by name (case-insensitive)
                let a_name = a.path.to_string_lossy().to_lowercase();
                let b_name = b.path.to_string_lossy().to_lowercase();
                a_name.cmp(&b_name)
            }
            other => other,
        }
    });
}
