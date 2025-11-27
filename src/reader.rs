/// Handles directory traversal and gathering file metadata.
use crate::file_entry::FileEntry;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub fn read_directory(target_path: &Path) -> io::Result<Vec<FileEntry>> {
    let mut entries: Vec<FileEntry> = Vec::new();

    for entry in fs::read_dir(target_path)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        let is_dir = metadata.is_dir();
        let is_executable = !is_dir && (metadata.permissions().mode() & 0o111) != 0;

        entries.push(FileEntry {
            path,
            is_dir,
            is_executable,
            mode: metadata.permissions().mode(),
        });
    }

    Ok(entries)
}
