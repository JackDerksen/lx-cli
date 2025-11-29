/// Handles file type icons and their associated colors.
use crate::config::{ColorConfig, IconColorConfig, IconConfig};
use colored::Color;

#[derive(Debug, Clone, Copy)]
pub enum FileIcon {
    Directory,
    Executable,
    RegularFile,
}

impl FileIcon {
    pub fn as_str(&self) -> &str {
        match self {
            FileIcon::Directory => "",
            FileIcon::Executable => "",
            FileIcon::RegularFile => "",
        }
    }

    pub fn as_str_custom(&self, config: &IconConfig) -> String {
        match self {
            FileIcon::Directory => config.get_directory_icon(),
            FileIcon::Executable => config.get_executable_icon(),
            FileIcon::RegularFile => config.get_regular_icon(),
        }
    }

    pub fn get_color(&self, config: &ColorConfig) -> Color {
        match self {
            FileIcon::Directory => config.get_directory_color(),
            FileIcon::Executable => config.get_executable_color(),
            FileIcon::RegularFile => config.get_regular_color(),
        }
    }

    pub fn get_icon_color(&self, config: &IconColorConfig) -> Color {
        match self {
            FileIcon::Directory => config.get_directory_color(),
            FileIcon::Executable => config.get_executable_color(),
            FileIcon::RegularFile => config.get_regular_color(),
        }
    }
}
