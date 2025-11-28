/// Core data structures (FileEntry and FileType) that represent files and its metadata.
use crate::config::ColorConfig;
use crate::icon::FileIcon;
use colored::Color;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_executable: bool,
    pub mode: u32,
    pub size: u64,
    pub modified: SystemTime,
    pub owner: String,
    pub group: String,
    pub nlink: u64,
}

impl FileEntry {
    fn get_file_icon(&self) -> FileIcon {
        if self.is_dir {
            FileIcon::Directory
        } else if self.is_executable {
            FileIcon::Executable
        } else {
            FileIcon::RegularFile
        }
    }

    pub fn get_icon(&self) -> String {
        self.get_file_icon().as_str().to_string()
    }

    pub fn get_color(&self, config: &ColorConfig) -> Color {
        self.get_file_icon().get_color(config)
    }

    pub fn get_file_type(&self) -> FileType {
        if self.is_dir {
            FileType::Directory
        } else if self.is_executable {
            FileType::Executable
        } else {
            FileType::RegularFile
        }
    }

    pub fn format_permissions(&self) -> String {
        let mode = self.mode;

        // File type
        let file_type = if self.is_dir { 'd' } else { '-' };

        // Owner permissions
        let user_r = if mode & 0o400 != 0 { 'r' } else { '-' };
        let user_w = if mode & 0o200 != 0 { 'w' } else { '-' };
        let user_x = if mode & 0o100 != 0 { 'x' } else { '-' };

        // Group permissions
        let group_r = if mode & 0o040 != 0 { 'r' } else { '-' };
        let group_w = if mode & 0o020 != 0 { 'w' } else { '-' };
        let group_x = if mode & 0o010 != 0 { 'x' } else { '-' };

        // Other permissions
        let other_r = if mode & 0o004 != 0 { 'r' } else { '-' };
        let other_w = if mode & 0o002 != 0 { 'w' } else { '-' };
        let other_x = if mode & 0o001 != 0 { 'x' } else { '-' };

        format!(
            "{}{}{}{}{}{}{}{}{}{}",
            file_type, user_r, user_w, user_x, group_r, group_w, group_x, other_r, other_w, other_x
        )
    }

    pub fn format_size(&self) -> String {
        let size = self.size;
        if size < 1024 {
            format!("{}B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1}K", size as f64 / 1024.0)
        } else if size < 1024 * 1024 * 1024 {
            format!("{:.1}M", size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1}G", size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    pub fn format_modified(&self) -> String {
        use std::time::UNIX_EPOCH;

        if let Ok(duration) = self.modified.duration_since(UNIX_EPOCH) {
            let secs = duration.as_secs();
            let datetime = chrono::DateTime::from_timestamp(secs as i64, 0)
                .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        } else {
            "Unknown".to_string()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Directory,
    Executable,
    RegularFile,
}
