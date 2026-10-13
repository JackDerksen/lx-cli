/// Handles file type icons and their associated colours.
use crate::config::{ColorConfig, IconColorConfig, IconConfig, IconPattern, IconPatternKind};
use crate::file_entry::FileEntry;
use crate::filter::matches_pattern;
use colored::Color;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::OnceLock;

impl IconPattern {
    pub(crate) fn matches(&self, entry: &FileEntry) -> bool {
        if (self.pattern.ends_with('/') && !entry.is_dir)
            || matches!(self.kind, IconPatternKind::Directory) && !entry.is_dir
            || matches!(self.kind, IconPatternKind::File) && entry.is_dir
        {
            return false;
        }
        let pattern = self
            .pattern
            .strip_suffix('/')
            .filter(|pattern| !pattern.is_empty())
            .unwrap_or(&self.pattern);
        let target = if pattern.contains('/') {
            entry.full_path.as_os_str()
        } else {
            entry
                .full_path
                .file_name()
                .unwrap_or(entry.full_path.as_os_str())
        };
        matches_pattern(pattern, &target.to_string_lossy())
    }
}

pub(crate) fn file_name_icon(name: &OsStr) -> Option<&'static str> {
    let name = name.to_str()?.to_ascii_lowercase();
    let stem = name.split(['.', '-', '_']).next()?;
    let named_icon = match stem {
        "license" | "licence" | "copying" | "copyright" => Some("\u{e60a}"),
        "readme" | "changelog" | "contributing" | "authors" => Some("\u{f02d}"),
        "makefile" | "gnumakefile" | "bsdmakefile" => Some("\u{e673}"),
        "dockerfile" | "containerfile" => Some("\u{e650}"),
        _ => None,
    };
    if named_icon.is_some() {
        return named_icon;
    }
    match name.as_str() {
        "cargo.toml" => return Some("\u{e68b}"),
        "package.json" => return Some("\u{e616}"),
        "pyproject.toml" | "pipfile" => return Some("\u{e606}"),
        "go.mod" => return Some("\u{e627}"),
        "gemfile" => return Some("\u{e605}"),
        "composer.json" => return Some("\u{e608}"),
        "package-lock.json" | "pnpm-lock.yaml" | "go.sum" | "bun.lockb" => return Some("\u{e672}"),
        "compose.yml"
        | "compose.yaml"
        | "docker-compose.yml"
        | "docker-compose.yaml"
        | ".dockerignore" => return Some("\u{e650}"),
        ".gitignore" | ".gitattributes" | ".gitmodules" | ".gitkeep" => return Some("\u{e702}"),
        ".editorconfig" | ".npmrc" | ".prettierrc" | ".eslintrc" => return Some("\u{e615}"),
        ".bashrc" | ".zshrc" | ".profile" | ".bash_profile" | ".zprofile" | ".envrc" => {
            return Some("\u{e691}");
        }
        _ => {}
    }
    if name == ".env" || name.starts_with(".env.") {
        return Some("\u{e615}");
    }
    Some(match name.rsplit_once('.')?.1 {
        "rs" => "\u{e68b}",
        "py" | "pyi" | "pyw" => "\u{e606}",
        "js" | "mjs" | "cjs" => "\u{e60c}",
        "ts" | "mts" | "cts" => "\u{e628}",
        "jsx" | "tsx" => "\u{e625}",
        "go" => "\u{e627}",
        "c" | "h" => "\u{e61e}",
        "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => "\u{e646}",
        "java" => "\u{e66d}",
        "rb" => "\u{e605}",
        "php" => "\u{e608}",
        "lua" => "\u{e620}",
        "swift" => "\u{e699}",
        "kt" | "kts" => "\u{e634}",
        "sh" | "bash" | "zsh" | "fish" => "\u{e691}",
        "ps1" => "\u{e683}",
        "html" | "htm" => "\u{e60e}",
        "css" => "\u{e614}",
        "scss" | "sass" => "\u{e603}",
        "json" | "jsonc" | "json5" => "\u{e60b}",
        "toml" | "yaml" | "yml" | "ini" | "conf" | "cfg" => "\u{e615}",
        "xml" => "\u{f121}",
        "lock" => "\u{e672}",
        "md" | "markdown" | "mdx" => "\u{e609}",
        "txt" | "log" | "rst" => "\u{e64e}",
        "pdf" => "\u{e67d}",
        "doc" | "docx" | "odt" | "rtf" => "\u{f1c2}",
        "csv" | "tsv" | "xls" | "xlsx" | "ods" => "\u{e64a}",
        "ppt" | "pptx" | "odp" => "\u{f1c4}",
        "epub" | "mobi" => "\u{f02d}",
        "svg" => "\u{e698}",
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "avif" | "heic" | "bmp" | "ico" | "tif"
        | "tiff" => "\u{e60d}",
        "mp3" | "wav" | "flac" | "ogg" | "opus" | "m4a" | "aiff" => "\u{f1c7}",
        "mp4" | "mkv" | "mov" | "avi" | "webm" | "m4v" | "mpg" | "mpeg" => "\u{f1c8}",
        "zip" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "7z" | "rar" | "zst" => "\u{f1c6}",
        "sql" | "db" | "sqlite" | "sqlite3" => "\u{e64d}",
        "ttf" | "otf" | "woff" | "woff2" => "\u{e659}",
        _ => return None,
    })
}

pub(crate) fn directory_name_icon(name: &OsStr) -> Option<&'static str> {
    Some(match name.to_str()? {
        ".git" => "\u{e702}",
        ".github" | ".gitlab" | ".vscode" | ".idea" => "\u{f107f}",
        "src" | "source" | "sources" | "lib" => "\u{f121}",
        "test" | "tests" | "__tests__" | "spec" | "specs" => "\u{f197e}",
        "target" | "build" | "dist" => "\u{f19fc}",
        "doc" | "docs" | "documentation" | "notes" => "\u{f0c82}",
        "assets" | "images" => "\u{f024f}",
        "node_modules" | "vendor" => "\u{f0253}",
        "scripts" | "bin" => "\u{f120}",
        _ => return None,
    })
}

pub(crate) fn system_directory_icon(filesystem_identity: (u64, u64)) -> Option<&'static str> {
    static ICONS: OnceLock<HashMap<(u64, u64), &'static str>> = OnceLock::new();
    let icons = ICONS.get_or_init(|| {
        let mut directories = vec![
            (dirs::home_dir(), "\u{f10b5}"),
            (dirs::desktop_dir(), "\u{f108}"),
            (dirs::document_dir(), "\u{f0c82}"),
            (dirs::download_dir(), "\u{f024d}"),
            (dirs::audio_dir(), "\u{f1359}"),
            (dirs::picture_dir(), "\u{f024f}"),
            (dirs::video_dir(), "\u{f19fa}"),
            (dirs::public_dir(), "\u{f024c}"),
            (dirs::template_dir(), "\u{f069d}"),
            (dirs::config_dir(), "\u{f107f}"),
            (dirs::cache_dir(), "\u{f0aba}"),
        ];
        if let Some(home) = dirs::home_dir() {
            for (path, icon) in [
                (".config", "\u{f107f}"),
                (".cache", "\u{f0aba}"),
                (".local", "\u{f0253}"),
                (".Trash", "\u{f1f8}"),
                (".ssh", "\u{f0250}"),
            ] {
                directories.push((Some(home.join(path)), icon));
            }
        }
        #[cfg(target_os = "macos")]
        {
            for path in ["/Applications", "/System/Applications"] {
                directories.push((Some(PathBuf::from(path)), "\u{f1b3}"));
            }
            directories.push((Some(PathBuf::from("/Library")), "\u{f02d}"));
            if let Some(home) = dirs::home_dir() {
                directories.push((Some(home.join("Applications")), "\u{f1b3}"));
                directories.push((Some(home.join("Library")), "\u{f02d}"));
            }
        }
        #[cfg(target_os = "linux")]
        {
            directories.extend([
                (dirs::data_dir(), "\u{f0253}"),
                (dirs::state_dir(), "\u{f0aba}"),
                (dirs::executable_dir(), "\u{f120}"),
                (dirs::data_dir().map(|path| path.join("Trash")), "\u{f1f8}"),
            ]);
            for (path, icon) in [
                ("/etc", "\u{f107f}"),
                ("/usr", "\u{f1b3}"),
                ("/var", "\u{f0aba}"),
                ("/tmp", "\u{f0aba}"),
                ("/home", "\u{f024c}"),
            ] {
                directories.push((Some(PathBuf::from(path)), icon));
            }
        }
        let mut icons = HashMap::new();
        for (path, icon) in directories {
            if let Some(metadata) = path.and_then(|path| std::fs::metadata(path).ok()) {
                if metadata.is_dir() {
                    // Filesystem identity survives relative paths and tree display-name changes.
                    icons
                        .entry((metadata.dev(), metadata.ino()))
                        .or_insert(icon);
                }
            }
        }
        icons
    });
    icons.get(&filesystem_identity).copied()
}

#[derive(Debug, Clone, Copy)]
pub enum FileIcon {
    Directory,
    Executable,
    Symlink,
    RegularFile,
}

impl FileIcon {
    pub fn as_str_custom(&self, config: &IconConfig) -> String {
        self.configured_icon(config)
            .unwrap_or(if config.enabled {
                match self {
                    FileIcon::Directory => "\u{f024b}",
                    FileIcon::Executable => "",
                    FileIcon::Symlink => "",
                    FileIcon::RegularFile => "",
                }
            } else {
                ""
            })
            .to_string()
    }

    pub(crate) fn configured_icon<'a>(&self, config: &'a IconConfig) -> Option<&'a str> {
        match self {
            FileIcon::Directory => config.directory.as_deref(),
            FileIcon::Executable => config.executable.as_deref(),
            FileIcon::Symlink => config.symlink.as_deref(),
            FileIcon::RegularFile => config.regular.as_deref(),
        }
    }

    pub fn get_color(&self, config: &ColorConfig) -> Color {
        match self {
            FileIcon::Directory => config.get_directory_color(),
            FileIcon::Executable => config.get_executable_color(),
            FileIcon::Symlink => crate::config::parse_color(&config.symlink),
            FileIcon::RegularFile => config.get_regular_color(),
        }
    }

    pub fn get_icon_color(&self, config: &IconColorConfig) -> Color {
        match self {
            FileIcon::Directory => config.get_directory_color(),
            FileIcon::Executable => config.get_executable_color(),
            FileIcon::Symlink => crate::config::parse_color(&config.symlink),
            FileIcon::RegularFile => config.get_regular_color(),
        }
    }
}
