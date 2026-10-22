/// Handles directory traversal and gathering file metadata.
use crate::dir_scan::{Stat, scan};
use crate::file_entry::{FileEntry, Symlink};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
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

    scan(
        &target_path,
        |name| show_hidden || !is_hidden(name),
        |entry| {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    report_error(format_args!(
                        "cannot read directory '{}': {error}",
                        target_path.display()
                    ));
                    return;
                }
            };
            let full_path = target_path.join(&entry.name);
            match entry.stat {
                Ok(stat) => entries.push(build_file_entry(
                    entry.name,
                    full_path,
                    &stat,
                    metadata_mode,
                    &mut cache,
                )),
                Err(error) => report_error(format_args!(
                    "cannot access '{}': {error}",
                    full_path.display()
                )),
            }
        },
    )?;

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
    let stat = Stat::from(&fs::symlink_metadata(&full_path)?);
    let file_name = target_path
        .file_name()
        .unwrap_or(target_path.as_os_str())
        .to_os_string();

    Ok(build_file_entry(
        file_name,
        full_path,
        &stat,
        metadata_mode,
        cache,
    ))
}

fn build_file_entry(
    path: std::ffi::OsString,
    full_path: PathBuf,
    stat: &Stat,
    metadata_mode: MetadataMode,
    cache: &mut UserGroupCache,
) -> FileEntry {
    let symlink = stat.is_symlink().then(|| {
        let target = fs::read_link(&full_path).unwrap_or_else(|error| {
            report_error(format_args!(
                "cannot read symbolic link '{}': {error}",
                full_path.display()
            ));
            PathBuf::new()
        });
        // Permission errors cannot tell us whether a target is broken.
        let is_broken = fs::metadata(&full_path).is_err_and(|error| {
            matches!(
                error.raw_os_error(),
                Some(libc::ENOENT | libc::ENOTDIR | libc::ELOOP)
            )
        });
        Symlink { target, is_broken }
    });
    let (owner, group) = match metadata_mode {
        MetadataMode::Basic => (String::new(), String::new()),
        MetadataMode::Full => (cache.get_username(stat.uid), cache.get_groupname(stat.gid)),
    };

    FileEntry {
        is_hidden: is_hidden(&path),
        path,
        full_path,
        filesystem_identity: (stat.dev, stat.ino),
        is_dir: stat.is_dir(),
        is_executable: stat.is_file() && (stat.mode & 0o111) != 0,
        mode: stat.mode,
        size: stat.size,
        size_incomplete: false,
        modified: stat.modified,
        owner,
        group,
        user_id: stat.uid,
        group_id: stat.gid,
        nlink: stat.nlink,
        git_status: None,
        symlink,
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
