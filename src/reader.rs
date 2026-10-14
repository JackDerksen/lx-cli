/// Handles directory traversal and gathering file metadata.
use crate::file_entry::{FileEntry, Symlink};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

static REPORTED_ERRORS: AtomicBool = AtomicBool::new(false);

/// Reports a problem with one entry on stderr and keeps listing; the exit status
/// later reflects it, as with `ls`.
pub(crate) fn report_error(message: std::fmt::Arguments<'_>) {
    eprintln!("lx: {message}");
    REPORTED_ERRORS.store(true, Ordering::Relaxed);
}

/// Whether any entry could not be read during this run.
pub fn reported_errors() -> bool {
    REPORTED_ERRORS.load(Ordering::Relaxed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataMode {
    Basic,
    Full,
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
    if fs::symlink_metadata(target_path)?.is_dir() {
        return read_directory_entries(target_path, show_hidden, metadata_mode);
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
) -> io::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    let mut cache = UserGroupCache::default();
    let target_path = std::path::absolute(target_path)?;

    for entry in fs::read_dir(&target_path)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                report_error(format_args!(
                    "cannot read directory '{}': {error}",
                    target_path.display()
                ));
                continue;
            }
        };
        let file_name = entry.file_name();

        if !show_hidden && is_hidden(&file_name) {
            continue;
        }

        let full_path = entry.path();
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(error) => {
                report_error(format_args!(
                    "cannot access '{}': {error}",
                    full_path.display()
                ));
                continue;
            }
        };
        let entry = build_file_entry(file_name, &full_path, &metadata, metadata_mode, &mut cache)?;

        entries.push(entry);
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
    let full_path = std::path::absolute(target_path)?;
    let metadata = fs::symlink_metadata(&full_path)?;
    let file_name = target_path
        .file_name()
        .unwrap_or(target_path.as_os_str())
        .to_os_string();

    build_file_entry(file_name, &full_path, &metadata, metadata_mode, cache)
}

fn build_file_entry(
    path: std::ffi::OsString,
    full_path: &Path,
    metadata: &fs::Metadata,
    metadata_mode: MetadataMode,
    cache: &mut UserGroupCache,
) -> io::Result<FileEntry> {
    let symlink = if metadata.is_symlink() {
        let target = fs::read_link(full_path).unwrap_or_else(|error| {
            report_error(format_args!(
                "cannot read symbolic link '{}': {error}",
                full_path.display()
            ));
            std::path::PathBuf::new()
        });
        // Permission errors cannot tell us whether a target is broken.
        let is_broken = fs::metadata(full_path).is_err_and(|error| {
            matches!(
                error.raw_os_error(),
                Some(libc::ENOENT | libc::ENOTDIR | libc::ELOOP)
            )
        });
        Some(Symlink { target, is_broken })
    } else {
        None
    };
    let is_dir = metadata.is_dir();
    let mode = metadata.mode();
    let user_id = metadata.uid();
    let group_id = metadata.gid();
    let (owner, group) = match metadata_mode {
        MetadataMode::Basic => (String::new(), String::new()),
        MetadataMode::Full => (cache.get_username(user_id), cache.get_groupname(group_id)),
    };

    Ok(FileEntry {
        is_hidden: is_hidden(&path),
        path,
        full_path: full_path.to_path_buf(),
        filesystem_identity: (metadata.dev(), metadata.ino()),
        is_dir,
        is_executable: metadata.is_file() && (mode & 0o111) != 0,
        mode,
        size: metadata.len(),
        size_incomplete: false,
        modified: metadata
            .modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
        owner,
        group,
        user_id,
        group_id,
        nlink: metadata.nlink(),
        git_status: None,
        symlink,
    })
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
