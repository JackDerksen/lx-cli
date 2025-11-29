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
                // Use as_encoded_bytes() to handle non-UTF-8 filenames
                // Files starting with '.' are hidden on Unix systems
                if filename.as_encoded_bytes().starts_with(b".") {
                    continue;
                }
            }
        }

        let metadata = entry.metadata()?;
        let is_dir = metadata.is_dir();
        let is_executable = !is_dir && (metadata.permissions().mode() & 0o111) != 0;

        // Get owner name
        let uid = metadata.uid();
        let owner = get_username(uid);

        // Get group name
        let gid = metadata.gid();
        let group = get_groupname(gid);

        // Get number of hard links
        let nlink = metadata.nlink();

        // Get modification time
        let modified = metadata
            .modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

        // Get size
        let size = metadata.len();

        entries.push(FileEntry {
            path: path.file_name().unwrap().to_os_string(),
            is_dir,
            is_executable,
            mode: metadata.permissions().mode(),
            size,
            modified,
            owner,
            group,
            nlink,
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

fn get_groupname(gid: u32) -> String {
    // Try to get group name from system, fallback to gid
    #[cfg(unix)]
    {
        use std::ffi::CStr;
        unsafe {
            let group = libc::getgrgid(gid);
            if !group.is_null() {
                let name = CStr::from_ptr((*group).gr_name);
                if let Ok(name_str) = name.to_str() {
                    return name_str.to_string();
                }
            }
        }
    }
    gid.to_string()
}
