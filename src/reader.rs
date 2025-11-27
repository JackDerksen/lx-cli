/// Handles directory traversal and gathering file metadata.
use crate::file_entry::FileEntry;
use std::fs;
use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

pub fn read_directory(target_path: &Path, show_hidden: bool) -> io::Result<Vec<FileEntry>> {
    let mut entries: Vec<FileEntry> = Vec::new();

    for entry in fs::read_dir(target_path)? {
        let entry = entry?;
        let path = entry.path();

        // Skip hidden files unless show_hidden is true
        if !show_hidden {
            if let Some(filename) = path.file_name() {
                if let Some(name) = filename.to_str() {
                    if name.starts_with('.') {
                        continue;
                    }
                }
            }
        }

        let metadata = entry.metadata()?;
        let is_dir = metadata.is_dir();
        let is_executable = !is_dir && (metadata.permissions().mode() & 0o111) != 0;

        // Get owner name
        let uid = metadata.uid();
        let owner = get_username(uid);

        // Get modification time
        let modified = metadata
            .modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

        // Get size
        let size = metadata.len();

        entries.push(FileEntry {
            path,
            is_dir,
            is_executable,
            mode: metadata.permissions().mode(),
            size,
            modified,
            owner,
        });
    }

    Ok(entries)
}

fn get_username(uid: u32) -> String {
    // Try to get username from system, fallback to uid
    #[cfg(unix)]
    {
        use std::ffi::CStr;
        unsafe {
            let passwd = libc::getpwuid(uid);
            if !passwd.is_null() {
                let name = CStr::from_ptr((*passwd).pw_name);
                if let Ok(name_str) = name.to_str() {
                    return name_str.to_string();
                }
            }
        }
    }
    uid.to_string()
}
