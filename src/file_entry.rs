/// Core data structures (FileEntry and FileType) that represent files and its metadata.
use crate::icon::FileIcon;
use colored::Color;
use std::path::PathBuf;

#[derive(Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_executable: bool,
    pub mode: u32,
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

    pub fn get_color(&self) -> Color {
        self.get_file_icon().get_color()
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Directory,
    Executable,
    RegularFile,
}
