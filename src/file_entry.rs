/// Core data structures (FileEntry and FileType) that represent files and its metadata.
use crate::config::IconConfig;
use crate::icon::FileIcon;
use chrono::format::{Item, StrftimeItems};
use std::borrow::Cow;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::SystemTime;

const FILE_TYPE_MASK: u32 = libc::S_IFMT as u32;
const FILE_TYPE_REGULAR: u32 = libc::S_IFREG as u32;
const FILE_TYPE_DIRECTORY: u32 = libc::S_IFDIR as u32;
const FILE_TYPE_SYMLINK: u32 = libc::S_IFLNK as u32;
const FILE_TYPE_PIPE: u32 = libc::S_IFIFO as u32;
const FILE_TYPE_SOCKET: u32 = libc::S_IFSOCK as u32;
const FILE_TYPE_BLOCK_DEVICE: u32 = libc::S_IFBLK as u32;
const FILE_TYPE_CHARACTER_DEVICE: u32 = libc::S_IFCHR as u32;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: OsString,
    /// Absolute listing path, kept separate from the formatted display name.
    pub full_path: PathBuf,
    /// Device and inode used to recognise system folders and configured paths.
    pub filesystem_identity: (u64, u64),
    pub is_dir: bool,
    pub is_executable: bool,
    pub is_hidden: bool,
    pub mode: u32,
    pub size: u64,
    pub size_incomplete: bool,
    pub modified: SystemTime,
    pub owner: String,
    pub group: String,
    pub user_id: u32,
    pub group_id: u32,
    pub nlink: u64,
    pub git_status: Option<crate::git::GitStatus>,
    pub symlink: Option<Symlink>,
}

#[derive(Debug, Clone)]
pub struct Symlink {
    pub target: PathBuf,
    pub is_broken: bool,
}

impl FileEntry {
    fn get_file_icon(&self) -> FileIcon {
        if self.symlink.is_some() {
            FileIcon::Symlink
        } else if self.is_dir {
            FileIcon::Directory
        } else if self.is_executable {
            FileIcon::Executable
        } else {
            FileIcon::RegularFile
        }
    }

    pub fn get_icon_custom(&self, config: &IconConfig) -> String {
        let file_icon = self.get_file_icon();
        if let Some(icon) = file_icon.configured_icon(config) {
            return icon.to_string();
        }
        if let Some(icon) = config.paths.get(&self.filesystem_identity) {
            return icon.clone();
        }
        if let Some(rule) = config.patterns.iter().find(|rule| rule.matches(self)) {
            return rule.icon.clone();
        }
        if config.enabled && self.symlink.is_none() {
            let name = self
                .full_path
                .file_name()
                .unwrap_or(self.full_path.as_os_str());
            let icon = if self.is_dir {
                crate::icon::system_directory_icon(self.filesystem_identity)
                    .or_else(|| crate::icon::directory_name_icon(name))
            } else {
                crate::icon::file_name_icon(name)
            };
            if let Some(icon) = icon {
                return icon.to_string();
            }
        }
        file_icon.as_str_custom(config)
    }

    pub(crate) fn style_role(&self) -> &'static str {
        if let Some(link) = &self.symlink {
            return if link.is_broken {
                "filekinds.broken_symlink"
            } else {
                "filekinds.symlink"
            };
        }

        match self.mode & FILE_TYPE_MASK {
            FILE_TYPE_DIRECTORY => "filekinds.directory",
            FILE_TYPE_REGULAR if self.is_executable => "filekinds.executable",
            FILE_TYPE_REGULAR => "filekinds.regular",
            FILE_TYPE_SYMLINK => "filekinds.symlink",
            FILE_TYPE_PIPE => "filekinds.pipe",
            FILE_TYPE_SOCKET => "filekinds.socket",
            FILE_TYPE_BLOCK_DEVICE => "filekinds.block_device",
            FILE_TYPE_CHARACTER_DEVICE => "filekinds.character_device",
            _ => "filekinds.special",
        }
    }

    pub fn get_file_type(&self) -> FileType {
        if self.symlink.is_some() {
            FileType::Symlink
        } else if self.is_dir {
            FileType::Directory
        } else if self.is_executable {
            FileType::Executable
        } else {
            FileType::RegularFile
        }
    }

    pub fn display_name(&self) -> Cow<'_, str> {
        let name = self.path.to_string_lossy();
        match &self.symlink {
            Some(link) => format!(
                "{name} -> {}{}",
                link.target.display(),
                if link.is_broken { " [broken]" } else { "" }
            )
            .into(),
            None => name,
        }
    }

    pub fn format_permissions(&self) -> String {
        let mode = self.mode;

        let file_type = self.file_type_marker();

        // Owner permissions
        let user_r = if mode & 0o400 != 0 { 'r' } else { '-' };
        let user_w = if mode & 0o200 != 0 { 'w' } else { '-' };
        let user_x = match (mode & 0o100 != 0, mode & u32::from(libc::S_ISUID) != 0) {
            (true, true) => 's',
            (false, true) => 'S',
            (true, false) => 'x',
            (false, false) => '-',
        };

        // Group permissions
        let group_r = if mode & 0o040 != 0 { 'r' } else { '-' };
        let group_w = if mode & 0o020 != 0 { 'w' } else { '-' };
        let group_x = match (mode & 0o010 != 0, mode & u32::from(libc::S_ISGID) != 0) {
            (true, true) => 's',
            (false, true) => 'S',
            (true, false) => 'x',
            (false, false) => '-',
        };

        // Other permissions
        let other_r = if mode & 0o004 != 0 { 'r' } else { '-' };
        let other_w = if mode & 0o002 != 0 { 'w' } else { '-' };
        let other_x = match (mode & 0o001 != 0, mode & u32::from(libc::S_ISVTX) != 0) {
            (true, true) => 't',
            (false, true) => 'T',
            (true, false) => 'x',
            (false, false) => '-',
        };

        format!(
            "{}{}{}{}{}{}{}{}{}{}",
            file_type, user_r, user_w, user_x, group_r, group_w, group_x, other_r, other_w, other_x
        )
    }

    fn file_type_marker(&self) -> char {
        if self.symlink.is_some() {
            return 'l';
        }

        match self.mode & FILE_TYPE_MASK {
            FILE_TYPE_REGULAR => '-',
            FILE_TYPE_DIRECTORY => 'd',
            FILE_TYPE_SYMLINK => 'l',
            FILE_TYPE_PIPE => 'p',
            FILE_TYPE_SOCKET => 's',
            FILE_TYPE_BLOCK_DEVICE => 'b',
            FILE_TYPE_CHARACTER_DEVICE => 'c',
            _ => '?',
        }
    }

    pub fn format_size(&self) -> String {
        let size = self.size;
        let mut formatted = if size < 1024 {
            format!("{}B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1}K", size as f64 / 1024.0)
        } else if size < 1024 * 1024 * 1024 {
            format!("{:.1}M", size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1}G", size as f64 / (1024.0 * 1024.0 * 1024.0))
        };
        if self.size_incomplete {
            formatted.push('?');
        }
        formatted
    }

    pub fn format_modified_parts(&self, datetime_format: &str) -> Vec<DateTimePart> {
        let datetime: chrono::DateTime<chrono::Local> = self.modified.into();

        StrftimeItems::new(datetime_format)
            .map(|item| {
                let alignment = match item {
                    Item::Numeric(_, _) => DateTimePartAlignment::Right,
                    Item::Fixed(_) => DateTimePartAlignment::Left,
                    _ => DateTimePartAlignment::None,
                };
                let value = datetime
                    .format_with_items(std::iter::once(item))
                    .to_string();

                DateTimePart { value, alignment }
            })
            .collect()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DateTimePartAlignment {
    Left,
    Right,
    None,
}

pub struct DateTimePart {
    pub value: String,
    pub alignment: DateTimePartAlignment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileType {
    Directory,
    Executable,
    Symlink,
    RegularFile,
}
