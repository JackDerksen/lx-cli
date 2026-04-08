/// Handles sorting of file entries
use crate::file_entry::FileEntry;

/// Default sort: by file type (directory, executable, regular), then alphabetically by name (case-insensitive)
pub fn sort_default(entries: &mut [FileEntry]) {
    entries.sort_by_cached_key(|entry| {
        (
            entry.get_file_type(),
            entry.path.to_string_lossy().to_lowercase(),
        )
    });
}
