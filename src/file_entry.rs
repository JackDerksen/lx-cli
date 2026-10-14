/// Core data structures (FileEntry and FileType) that represent files and its metadata.
use crate::config::IconConfig;
use crate::icon::FileIcon;
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
        self.permission_chars().iter().collect()
    }

    /// The `ls -l` permission string as characters, without allocating.
    pub fn permission_chars(&self) -> [char; 10] {
        let mode = self.mode;
        let flag = |bit: u32, set: char| if mode & bit != 0 { set } else { '-' };
        let execute = |bit: u32, special: u32, both: char, special_only: char| match (
            mode & bit != 0,
            mode & special != 0,
        ) {
            (true, true) => both,
            (false, true) => special_only,
            (true, false) => 'x',
            (false, false) => '-',
        };
        [
            self.file_type_marker(),
            flag(0o400, 'r'),
            flag(0o200, 'w'),
            execute(0o100, u32::from(libc::S_ISUID), 's', 'S'),
            flag(0o040, 'r'),
            flag(0o020, 'w'),
            execute(0o010, u32::from(libc::S_ISGID), 's', 'S'),
            flag(0o004, 'r'),
            flag(0o002, 'w'),
            execute(0o001, u32::from(libc::S_ISVTX), 't', 'T'),
        ]
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

    pub fn format_size(&self) -> FormattedSize {
        let (number, unit) = if self.size < 1024 {
            (self.size.to_string(), SizeUnit::Bytes)
        } else {
            let mut value = self.size as f64 / 1024.0;
            let mut unit = SizeUnit::Kilobytes;
            // Pick the unit after rounding so 1048575 bytes reads 1.0M, not 1024.0K.
            while (value * 10.0).round() / 10.0 >= 1024.0 && unit != SizeUnit::Terabytes {
                value /= 1024.0;
                unit = unit.next();
            }
            (format!("{value:.1}"), unit)
        };
        FormattedSize {
            number,
            unit,
            incomplete: self.size_incomplete,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeUnit {
    Bytes,
    Kilobytes,
    Megabytes,
    Gigabytes,
    Terabytes,
}

impl SizeUnit {
    fn next(self) -> Self {
        match self {
            Self::Bytes => Self::Kilobytes,
            Self::Kilobytes => Self::Megabytes,
            Self::Megabytes => Self::Gigabytes,
            Self::Gigabytes | Self::Terabytes => Self::Terabytes,
        }
    }

    pub fn suffix(self) -> &'static str {
        match self {
            Self::Bytes => "B",
            Self::Kilobytes => "K",
            Self::Megabytes => "M",
            Self::Gigabytes => "G",
            Self::Terabytes => "T",
        }
    }

    /// Theme roles for the number and the unit suffix.
    pub fn roles(self) -> (&'static str, &'static str) {
        match self {
            Self::Bytes => ("size.bytes.number", "size.bytes.unit"),
            Self::Kilobytes => ("size.kilobytes.number", "size.kilobytes.unit"),
            Self::Megabytes => ("size.megabytes.number", "size.megabytes.unit"),
            Self::Gigabytes => ("size.gigabytes.number", "size.gigabytes.unit"),
            Self::Terabytes => ("size.terabytes.number", "size.terabytes.unit"),
        }
    }
}

/// A human-readable size, kept in parts so renderers can style each one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormattedSize {
    pub number: String,
    pub unit: SizeUnit,
    /// The size is a lower bound because some contents could not be read.
    pub incomplete: bool,
}

impl std::fmt::Display for FormattedSize {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}{}", self.number, self.unit.suffix())?;
        if self.incomplete {
            formatter.write_str("?")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileType {
    Directory,
    Executable,
    Symlink,
    RegularFile,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry_with_size(size: u64) -> FileEntry {
        FileEntry {
            path: OsString::from("file"),
            full_path: PathBuf::from("/file"),
            filesystem_identity: (0, 0),
            is_dir: false,
            is_executable: false,
            is_hidden: false,
            mode: FILE_TYPE_REGULAR | 0o644,
            size,
            size_incomplete: false,
            modified: SystemTime::UNIX_EPOCH,
            owner: String::new(),
            group: String::new(),
            user_id: 0,
            group_id: 0,
            nlink: 1,
            git_status: None,
            symlink: None,
        }
    }

    #[test]
    fn sizes_pick_their_unit_after_rounding() {
        const KIB: u64 = 1024;
        for (size, expected) in [
            (0, "0B"),
            (1023, "1023B"),
            (KIB, "1.0K"),
            (KIB * KIB - 1, "1.0M"),
            (KIB * KIB, "1.0M"),
            (KIB * KIB * KIB - 1, "1.0G"),
            (KIB.pow(4) - 1, "1.0T"),
            (KIB.pow(4) * 3 / 2, "1.5T"),
            (KIB.pow(5) * 2, "2048.0T"),
        ] {
            assert_eq!(entry_with_size(size).format_size().to_string(), expected);
        }
        let mut incomplete = entry_with_size(KIB * 3 / 2);
        incomplete.size_incomplete = true;
        let formatted = incomplete.format_size();
        assert_eq!(formatted.number, "1.5");
        assert_eq!(formatted.unit, SizeUnit::Kilobytes);
        assert_eq!(formatted.to_string(), "1.5K?");
    }
}
