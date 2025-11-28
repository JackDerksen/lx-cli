/// Handles file type icons and their associated colors.
use crate::config::ColorConfig;
use colored::Color;

#[derive(Debug, Clone, Copy)]
pub enum FileIcon {
    Directory,
    Executable,
    RegularFile,
}

// TODO: Add more file types and icons, as well as the ability to set custom icons in the config
impl FileIcon {
    pub fn as_str(&self) -> &str {
        match self {
            FileIcon::Directory => "",
            FileIcon::Executable => "",
            FileIcon::RegularFile => "",
        }
    }

    pub fn get_color(&self, config: &ColorConfig) -> Color {
        match self {
            FileIcon::Directory => config.get_directory_color(),
            FileIcon::Executable => config.get_executable_color(),
            FileIcon::RegularFile => config.get_regular_color(),
        }
    }
}
