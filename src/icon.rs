/// Handles file type icons and their associated colors.
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

    pub fn get_color(&self) -> Color {
        match self {
            FileIcon::Directory => Color::Blue,
            FileIcon::Executable => Color::Green,
            FileIcon::RegularFile => Color::White,
        }
    }
}
