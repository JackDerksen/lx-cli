/// Handles directory traversal and gathering file metadata.
use crate::file_entry::FileEntry;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataMode {
    Basic,
    Full,
}

#[derive(Debug, Clone)]
pub struct DiscoveredEntry {
    pub entry: FileEntry,
    pub full_path: PathBuf,
}

#[derive(Default)]
struct UserGroupCache {
    users: HashMap<u32, String>,
    groups: HashMap<u32, String>,
}

pub fn read_target(
    target_path: &Path,
    show_hidden: bool,
    metadata_mode: MetadataMode,
) -> io::Result<Vec<FileEntry>> {
    if target_path.is_dir() {
        return Ok(
            read_directory_entries(target_path, show_hidden, metadata_mode)?
                .into_iter()
                .map(|entry| entry.entry)
                .collect(),
        );
    }

    let mut cache = UserGroupCache::default();
    Ok(vec![read_file_entry(
        target_path,
        metadata_mode,
        &mut cache,
    )?])
}

pub fn read_directory_entries(
    target_path: &Path,
    show_hidden: bool,
    metadata_mode: MetadataMode,
) -> io::Result<Vec<DiscoveredEntry>> {
    let mut entries = Vec::new();
    let mut cache = UserGroupCache::default();

    for entry in fs::read_dir(target_path)? {
        let entry = entry?;
        let file_name = entry.file_name();

        if !show_hidden && is_hidden(&file_name) {
            continue;
        }

        let full_path = entry.path();
        let metadata = entry.metadata()?;
        let entry = build_file_entry(file_name, &metadata, metadata_mode, &mut cache);

        entries.push(DiscoveredEntry { entry, full_path });
    }

    Ok(entries)
}

pub(crate) fn read_entry(target_path: &Path, metadata_mode: MetadataMode) -> io::Result<FileEntry> {
    let mut cache = UserGroupCache::default();
    read_file_entry(target_path, metadata_mode, &mut cache)
}

fn read_file_entry(
    target_path: &Path,
    metadata_mode: MetadataMode,
    cache: &mut UserGroupCache,
) -> io::Result<FileEntry> {
    let metadata = fs::metadata(target_path)?;
    let file_name = target_path
        .file_name()
        .unwrap_or(target_path.as_os_str())
        .to_os_string();

    Ok(build_file_entry(file_name, &metadata, metadata_mode, cache))
}

fn build_file_entry(
    path: std::ffi::OsString,
    metadata: &fs::Metadata,
    metadata_mode: MetadataMode,
    cache: &mut UserGroupCache,
) -> FileEntry {
    let is_dir = metadata.is_dir();
    let mode = metadata.permissions().mode();
    let (owner, group) = match metadata_mode {
        MetadataMode::Basic => (String::new(), String::new()),
        MetadataMode::Full => (
            cache.get_username(metadata.uid()),
            cache.get_groupname(metadata.gid()),
        ),
    };

    FileEntry {
        is_hidden: is_hidden(&path),
        path,
        is_dir,
        is_executable: !is_dir && (mode & 0o111) != 0,
        mode,
        size: metadata.len(),
        modified: metadata
            .modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
        owner,
        group,
        nlink: metadata.nlink(),
    }
}

pub(crate) fn is_hidden(file_name: &std::ffi::OsStr) -> bool {
    let bytes = file_name.as_encoded_bytes();
    bytes.starts_with(b".") && bytes != b"." && bytes != b".."
}

impl UserGroupCache {
    fn get_username(&mut self, uid: u32) -> String {
        if let Some(username) = self.users.get(&uid) {
            return username.clone();
        }

        let username = lookup_username(uid);
        self.users.insert(uid, username.clone());
        username
    }

    fn get_groupname(&mut self, gid: u32) -> String {
        if let Some(groupname) = self.groups.get(&gid) {
            return groupname.clone();
        }

        let groupname = lookup_groupname(gid);
        self.groups.insert(gid, groupname.clone());
        groupname
    }
}

fn lookup_username(uid: u32) -> String {
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

fn lookup_groupname(gid: u32) -> String {
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
