use colored::Color;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub colors: ColorConfig,
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
pub struct DisplayConfig {
    #[serde(default = "default_column_spacing")]
    pub column_spacing: usize,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            colors: ColorConfig::default(),
            display: DisplayConfig::default(),
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
