/// Core data structures (FileEntry and FileType) that represent files and its metadata.
use crate::icon::FileIcon;
use colored::Color;
use std::path::PathBuf;

#[derive(Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_executable: bool,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Directory,
    Executable,
    RegularFile,
}
