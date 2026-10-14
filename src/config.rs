use crate::sort::{SortField, SortOptions, SortOrder};
use crate::theme::Theme;
use colored::{ColoredString, Colorize};
use serde::{Deserialize, Deserializer};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use toml::Table;

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub theme: Theme,
    pub icons: IconConfig,
    pub display: DisplayConfig,
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<DeserializerType>(
        deserializer: DeserializerType,
    ) -> Result<Self, DeserializerType::Error>
    where
        DeserializerType: Deserializer<'de>,
    {
        let document = Table::deserialize(deserializer)?;
        resolve_config(document, None, &mut Vec::new()).map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize, Default)]
struct Settings {
    #[serde(default)]
    theme: Option<String>,
    #[serde(default)]
    icons: IconConfig,
    #[serde(default)]
    display: DisplayConfig,
}

/// Terminal font attributes shared by every semantic style.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FontStyle {
    Bold,
    #[serde(alias = "dim")]
    Dimmed,
    Italic,
    Underline,
    Strikethrough,
    Reversed,
}

pub(crate) fn apply_font_styles(mut text: ColoredString, styles: &[FontStyle]) -> ColoredString {
    for style in styles {
        text = match style {
            FontStyle::Bold => text.bold(),
            FontStyle::Dimmed => text.dimmed(),
            FontStyle::Italic => text.italic(),
            FontStyle::Underline => text.underline(),
            FontStyle::Strikethrough => text.strikethrough(),
            FontStyle::Reversed => text.reversed(),
        };
    }
    text
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct IconConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub directory: Option<String>,
    #[serde(default)]
    pub executable: Option<String>,
    #[serde(default)]
    pub regular: Option<String>,
    #[serde(default)]
    pub symlink: Option<String>,
    /// Icons by filesystem identity, resolved from path keys when reading configuration.
    #[serde(default, deserialize_with = "deserialize_path_icons")]
    pub paths: HashMap<(u64, u64), String>,
    #[serde(default)]
    pub patterns: Vec<IconPattern>,
}

/// An ordered icon rule matching an entry's name or absolute path.
#[derive(Debug, Deserialize, Clone)]
pub struct IconPattern {
    pub pattern: String,
    pub icon: String,
    #[serde(default)]
    pub kind: IconPatternKind,
}

#[derive(Debug, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum IconPatternKind {
    #[default]
    Any,
    File,
    Directory,
}

fn deserialize_path_icons<'de, DeserializerType>(
    deserializer: DeserializerType,
) -> Result<HashMap<(u64, u64), String>, DeserializerType::Error>
where
    DeserializerType: Deserializer<'de>,
{
    let paths = BTreeMap::<String, String>::deserialize(deserializer)?;
    Ok(paths
        .into_iter()
        .filter_map(|(path, icon)| {
            if path.is_empty() {
                return None;
            }
            let path = PathBuf::from(path);
            let path = if path.is_absolute() {
                path
            } else {
                dirs::home_dir()?.join(path.strip_prefix("~").unwrap_or(&path))
            };
            let metadata = fs::symlink_metadata(path).ok()?;
            Some(((metadata.dev(), metadata.ino()), icon))
        })
        .collect())
}

#[derive(Debug, Deserialize, Clone)]
pub struct DisplayConfig {
    #[serde(default = "default_column_spacing")]
    pub column_spacing: usize,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_compact_max_rows")]
    pub compact_max_rows: usize,
    #[serde(default)]
    pub sort_field: Option<SortField>,
    #[serde(default)]
    pub sort_order: SortOrder,
    #[serde(default = "default_datetime_format")]
    pub datetime_format: String,
    #[serde(default)]
    pub tree: TreeConfig,
    #[serde(default = "default_long_format_fields")]
    pub long_format_fields: Vec<String>,
    #[serde(default = "default_long_format_titles")]
    pub long_format_titles: bool,
    #[serde(default)]
    pub language_colors: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TreeConfig {
    #[serde(default = "default_tree_style")]
    pub style: String,
}

impl Default for TreeConfig {
    fn default() -> Self {
        TreeConfig {
            style: default_tree_style(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        DisplayConfig {
            column_spacing: default_column_spacing(),
            max_rows: default_max_rows(),
            compact_max_rows: default_compact_max_rows(),
            sort_field: None,
            sort_order: SortOrder::Asc,
            datetime_format: default_datetime_format(),
            tree: TreeConfig::default(),
            long_format_fields: default_long_format_fields(),
            long_format_titles: default_long_format_titles(),
            language_colors: false,
        }
    }
}

fn default_column_spacing() -> usize {
    2
}

fn default_max_rows() -> usize {
    0 // 0 means no limit
}

fn default_compact_max_rows() -> usize {
    10
}

fn default_datetime_format() -> String {
    "%-d %b %H:%M".to_string()
}

fn default_tree_style() -> String {
    "ascii".to_string()
}

fn default_long_format_fields() -> Vec<String> {
    vec![
        "permissions".to_string(),
        "size".to_string(),
        "owner".to_string(),
        "modified".to_string(),
        "git".to_string(),
        "filename".to_string(),
    ]
}

fn default_long_format_titles() -> bool {
    true
}

impl IconConfig {
    pub fn get_directory_icon(&self) -> String {
        crate::icon::FileIcon::Directory.as_str_custom(self)
    }

    pub fn get_executable_icon(&self) -> String {
        crate::icon::FileIcon::Executable.as_str_custom(self)
    }

    pub fn get_regular_icon(&self) -> String {
        crate::icon::FileIcon::RegularFile.as_str_custom(self)
    }

    pub fn get_symlink_icon(&self) -> String {
        crate::icon::FileIcon::Symlink.as_str_custom(self)
    }

    /// Whether any file type has a visible icon after applying overrides.
    pub fn has_icons(&self) -> bool {
        [
            self.get_directory_icon(),
            self.get_executable_icon(),
            self.get_regular_icon(),
            self.get_symlink_icon(),
        ]
        .iter()
        .any(|icon| !icon.is_empty())
            || self.paths.values().any(|icon| !icon.is_empty())
            || self.patterns.iter().any(|rule| !rule.icon.is_empty())
    }
}

impl DisplayConfig {
    pub(crate) fn long_format_fields(&self) -> Vec<String> {
        self.long_format_fields
            .iter()
            .filter(|field| field.as_str() != "icon")
            .cloned()
            .collect()
    }

    pub fn default_sort(&self) -> SortOptions {
        SortOptions::new(self.sort_field, self.sort_order)
    }
}

pub fn load_config() -> Config {
    let directory = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/lx");
    let preferred = directory.join("config.toml");
    let config_path = match fs::symlink_metadata(&preferred) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => directory.join("config"),
        _ => preferred,
    };
    let contents = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(error) => {
            if error.kind() != std::io::ErrorKind::NotFound || config_path.ends_with("config.toml")
            {
                eprintln!(
                    "lx: cannot read config '{}': {error}",
                    config_path.display()
                );
            }
            return Config::default();
        }
    };
    let document = match toml::from_str::<Table>(&contents) {
        Ok(document) => document,
        Err(error) => {
            eprintln!(
                "lx: cannot parse config '{}': {error}",
                config_path.display()
            );
            return Config::default();
        }
    };
    let mut diagnostics = Vec::new();
    let result = resolve_config(document, Some(&directory), &mut diagnostics);
    for diagnostic in diagnostics {
        eprintln!("lx: config '{}': {diagnostic}", config_path.display());
    }
    result.unwrap_or_else(|error| {
        eprintln!(
            "lx: cannot parse config '{}': {error}",
            config_path.display()
        );
        Config::default()
    })
}

fn resolve_config(
    document: Table,
    directory: Option<&Path>,
    diagnostics: &mut Vec<String>,
) -> Result<Config, toml::de::Error> {
    let settings: Settings = toml::Value::Table(document.clone()).try_into()?;
    let mut theme = Theme::default();
    if let (Some(name), Some(directory)) = (settings.theme.as_deref(), directory) {
        apply_named_theme(&mut theme, name, directory, diagnostics);
    }
    theme.apply_legacy(&document, diagnostics);
    theme.apply_document(&document, true, diagnostics);
    Ok(Config {
        theme,
        icons: settings.icons,
        display: settings.display,
    })
}

fn apply_named_theme(
    theme: &mut Theme,
    name: &str,
    directory: &Path,
    diagnostics: &mut Vec<String>,
) {
    if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\']) {
        diagnostics.push(format!("invalid theme name: {name:?}"));
        return;
    }
    let path = directory.join("themes").join(format!("{name}.toml"));
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if name == "default" && error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            diagnostics.push(format!("cannot read theme '{}': {error}", path.display()));
            return;
        }
    };
    match toml::from_str::<Table>(&contents) {
        Ok(document) => {
            let mut theme_diagnostics = Vec::new();
            for key in document.keys().filter(|key| key.as_str() != "style") {
                theme_diagnostics.push(format!("unknown key {key:?}; themes contain only style"));
            }
            let mut candidate = theme.clone();
            candidate.apply_document(&document, false, &mut theme_diagnostics);
            if theme_diagnostics.is_empty() {
                *theme = candidate;
            }
            diagnostics.extend(
                theme_diagnostics
                    .into_iter()
                    .map(|message| format!("theme '{}': {message}", path.display())),
            );
        }
        Err(error) => diagnostics.push(format!("cannot parse theme '{}': {error}", path.display())),
    }
}
