use crate::sort::{SortField, SortOptions, SortOrder};
use colored::{Color, ColoredString, Colorize};
use serde::{Deserialize, Deserializer};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub colors: ColorConfig,
    #[serde(default)]
    pub styles: FontStyleConfig,
    #[serde(default)]
    pub icons: IconConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

/// Terminal font attributes that can be combined with existing colours.
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum FontStyle {
    Bold,
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

/// Filename attributes; hidden entries inherit their file type unless overridden.
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct FontStyleConfig {
    pub directory: Vec<FontStyle>,
    pub executable: Vec<FontStyle>,
    pub regular: Vec<FontStyle>,
    pub hidden: Option<Vec<FontStyle>>,
    pub symlink: Vec<FontStyle>,
    pub broken_symlink: Vec<FontStyle>,
}

impl Default for FontStyleConfig {
    fn default() -> Self {
        Self {
            directory: vec![FontStyle::Bold],
            executable: vec![FontStyle::Bold],
            regular: Vec::new(),
            hidden: None,
            symlink: Vec::new(),
            broken_symlink: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ColorConfig {
    #[serde(default = "default_directory_color")]
    pub directory: String,
    #[serde(default = "default_executable_color")]
    pub executable: String,
    #[serde(default = "default_regular_color")]
    pub regular: String,
    #[serde(default = "default_hidden_color")]
    pub hidden: String,
    #[serde(default = "default_symlink_color")]
    pub symlink: String,
    #[serde(default = "default_broken_symlink_color")]
    pub broken_symlink: String,
    #[serde(default)]
    pub git: GitColorConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct GitColorConfig {
    pub clean: String,
    pub modified: String,
    pub staged: String,
    pub untracked: String,
    pub conflict: String,
    pub mixed: String,
}

impl Default for GitColorConfig {
    fn default() -> Self {
        Self {
            clean: "bright_black".to_string(),
            modified: "yellow".to_string(),
            staged: "green".to_string(),
            untracked: "cyan".to_string(),
            conflict: "red".to_string(),
            mixed: "yellow".to_string(),
        }
    }
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
    #[serde(default)]
    pub colors: IconColorConfig,
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
pub struct IconColorConfig {
    #[serde(default = "default_directory_icon_color")]
    pub directory: String,
    #[serde(default = "default_executable_icon_color")]
    pub executable: String,
    #[serde(default = "default_regular_icon_color")]
    pub regular: String,
    #[serde(default = "default_hidden_icon_color")]
    pub hidden: String,
    #[serde(default = "default_symlink_color")]
    pub symlink: String,
    #[serde(default = "default_broken_symlink_color")]
    pub broken_symlink: String,
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
    #[serde(default = "default_long_format_title_color")]
    pub long_format_title_color: String,
    #[serde(default = "default_long_format_title_style")]
    pub long_format_title_style: Vec<FontStyle>,
    #[serde(default)]
    pub long_format_styles: HashMap<String, Vec<FontStyle>>,
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

impl Default for IconColorConfig {
    fn default() -> Self {
        IconColorConfig {
            directory: default_directory_icon_color(),
            executable: default_executable_icon_color(),
            regular: default_regular_icon_color(),
            hidden: default_hidden_icon_color(),
            symlink: default_symlink_color(),
            broken_symlink: default_broken_symlink_color(),
        }
    }
}

impl Default for ColorConfig {
    fn default() -> Self {
        ColorConfig {
            directory: default_directory_color(),
            executable: default_executable_color(),
            regular: default_regular_color(),
            hidden: default_hidden_color(),
            symlink: default_symlink_color(),
            broken_symlink: default_broken_symlink_color(),
            git: GitColorConfig::default(),
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
            long_format_title_color: default_long_format_title_color(),
            long_format_title_style: default_long_format_title_style(),
            long_format_styles: HashMap::new(),
        }
    }
}

fn default_directory_color() -> String {
    "blue".to_string()
}

fn default_executable_color() -> String {
    "red".to_string()
}

fn default_regular_color() -> String {
    "white".to_string()
}

fn default_hidden_color() -> String {
    "bright_black".to_string()
}

fn default_symlink_color() -> String {
    "cyan".to_string()
}

fn default_broken_symlink_color() -> String {
    "bright_red".to_string()
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

fn default_long_format_title_color() -> String {
    "bright_black".to_string()
}

fn default_long_format_title_style() -> Vec<FontStyle> {
    vec![FontStyle::Underline]
}

fn default_directory_icon_color() -> String {
    "blue".to_string()
}

fn default_executable_icon_color() -> String {
    "red".to_string()
}

fn default_regular_icon_color() -> String {
    "white".to_string()
}

fn default_hidden_icon_color() -> String {
    "bright_black".to_string()
}

impl ColorConfig {
    pub fn get_directory_color(&self) -> Color {
        parse_color(&self.directory)
    }

    pub fn get_executable_color(&self) -> Color {
        parse_color(&self.executable)
    }

    pub fn get_regular_color(&self) -> Color {
        parse_color(&self.regular)
    }

    pub fn get_hidden_color(&self) -> Color {
        parse_color(&self.hidden)
    }
}

impl IconColorConfig {
    pub fn get_directory_color(&self) -> Color {
        parse_color(&self.directory)
    }

    pub fn get_executable_color(&self) -> Color {
        parse_color(&self.executable)
    }

    pub fn get_regular_color(&self) -> Color {
        parse_color(&self.regular)
    }

    pub fn get_hidden_color(&self) -> Color {
        parse_color(&self.hidden)
    }
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

    pub fn get_long_format_title_color(&self) -> Color {
        parse_color(&self.long_format_title_color)
    }

    pub fn default_sort(&self) -> SortOptions {
        SortOptions::new(self.sort_field, self.sort_order)
    }
}

pub(crate) fn parse_color(color_str: &str) -> Color {
    match color_str.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "bright_black" => Color::BrightBlack,
        "bright_red" => Color::BrightRed,
        "bright_green" => Color::BrightGreen,
        "bright_yellow" => Color::BrightYellow,
        "bright_blue" => Color::BrightBlue,
        "bright_magenta" => Color::BrightMagenta,
        "bright_cyan" => Color::BrightCyan,
        "bright_white" => Color::BrightWhite,
        _ => Color::White, // Default fallback
    }
}

pub fn load_config() -> Config {
    let config_path = get_config_path();

    if !config_path.exists() {
        return Config::default();
    }

    match fs::read_to_string(&config_path) {
        Ok(contents) => match toml::from_str(&contents) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Warning: Failed to parse config file: {}", e);
                Config::default()
            }
        },
        Err(_) => Config::default(),
    }
}

fn get_config_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".config");
    path.push("lx");
    path.push("config");
    path
}
