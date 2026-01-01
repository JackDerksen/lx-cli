use colored::Color;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub colors: ColorConfig,
    #[serde(default)]
    pub icons: IconConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ColorConfig {
    #[serde(default = "default_directory_color")]
    pub directory: String,
    #[serde(default = "default_executable_color")]
    pub executable: String,
    #[serde(default = "default_regular_color")]
    pub regular: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IconConfig {
    #[serde(default = "default_directory_icon")]
    pub directory: String,
    #[serde(default = "default_executable_icon")]
    pub executable: String,
    #[serde(default = "default_regular_icon")]
    pub regular: String,
    #[serde(default)]
    pub colors: IconColorConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IconColorConfig {
    #[serde(default = "default_directory_icon_color")]
    pub directory: String,
    #[serde(default = "default_executable_icon_color")]
    pub executable: String,
    #[serde(default = "default_regular_icon_color")]
    pub regular: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DisplayConfig {
    #[serde(default = "default_column_spacing")]
    pub column_spacing: usize,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default)]
    pub tree: TreeConfig,
    #[serde(default = "default_long_format_fields")]
    pub long_format_fields: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TreeConfig {
    #[serde(default = "default_tree_style")]
    pub style: String,
    #[serde(default = "default_recursive_long_format")]
    pub recursive_long_format: String,
}

impl Default for TreeConfig {
    fn default() -> Self {
        TreeConfig {
            style: default_tree_style(),
            recursive_long_format: default_recursive_long_format(),
        }
    }
}

impl Default for IconConfig {
    fn default() -> Self {
        IconConfig {
            directory: default_directory_icon(),
            executable: default_executable_icon(),
            regular: default_regular_icon(),
            colors: IconColorConfig::default(),
        }
    }
}

impl Default for IconColorConfig {
    fn default() -> Self {
        IconColorConfig {
            directory: default_directory_icon_color(),
            executable: default_executable_icon_color(),
            regular: default_regular_icon_color(),
        }
    }
}

impl Default for ColorConfig {
    fn default() -> Self {
        ColorConfig {
            directory: default_directory_color(),
            executable: default_executable_color(),
            regular: default_regular_color(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        DisplayConfig {
            column_spacing: default_column_spacing(),
            max_rows: default_max_rows(),
            tree: TreeConfig::default(),
            long_format_fields: default_long_format_fields(),
        }
    }
}

fn default_directory_color() -> String {
    "blue".to_string()
}

fn default_executable_color() -> String {
    "green".to_string()
}

fn default_regular_color() -> String {
    "white".to_string()
}

fn default_column_spacing() -> usize {
    2
}

fn default_max_rows() -> usize {
    0 // 0 means no limit
}

fn default_tree_style() -> String {
    "ascii".to_string()
}

fn default_recursive_long_format() -> String {
    "nested".to_string()
}

fn default_long_format_fields() -> Vec<String> {
    vec![
        "filename".to_string(),
        "icon".to_string(),
        "size".to_string(),
        "modified".to_string(),
        "owner".to_string(),
        "group".to_string(),
        "nlink".to_string(),
        "permissions".to_string(),
    ]
}

fn default_directory_icon() -> String {
    "".to_string()
}

fn default_executable_icon() -> String {
    "".to_string()
}

fn default_regular_icon() -> String {
    "".to_string()
}

fn default_directory_icon_color() -> String {
    "blue".to_string()
}

fn default_executable_icon_color() -> String {
    "green".to_string()
}

fn default_regular_icon_color() -> String {
    "white".to_string()
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
}

impl IconConfig {
    pub fn get_directory_icon(&self) -> String {
        self.directory.clone()
    }

    pub fn get_executable_icon(&self) -> String {
        self.executable.clone()
    }

    pub fn get_regular_icon(&self) -> String {
        self.regular.clone()
    }
}

fn parse_color(color_str: &str) -> Color {
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
