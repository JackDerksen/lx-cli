use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::IsTerminal;
use std::sync::OnceLock;
use toml::{Table, Value};

const RESET: &str = "\x1b[0m";

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

impl FontStyle {
    /// Escape sequences list attributes in this order, whatever order they were configured in.
    const EMISSION_ORDER: [Self; 6] = [
        Self::Bold,
        Self::Dimmed,
        Self::Underline,
        Self::Reversed,
        Self::Italic,
        Self::Strikethrough,
    ];

    fn sgr_code(self) -> u8 {
        match self {
            Self::Bold => 1,
            Self::Dimmed => 2,
            Self::Italic => 3,
            Self::Underline => 4,
            Self::Reversed => 7,
            Self::Strikethrough => 9,
        }
    }

    fn bit(self) -> u8 {
        1 << self as u8
    }
}

/// A set of font attributes, kept as bits so styles stay `Copy`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FontStyles(u8);

impl FontStyles {
    pub fn from_slice(styles: &[FontStyle]) -> Self {
        Self(styles.iter().fold(0, |bits, style| bits | style.bit()))
    }

    pub fn contains(self, style: FontStyle) -> bool {
        self.0 & style.bit() != 0
    }

    fn iter(self) -> impl Iterator<Item = FontStyle> {
        FontStyle::EMISSION_ORDER
            .into_iter()
            .filter(move |style| self.contains(*style))
    }
}

/// A terminal colour as configured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// One of the 16 standard colours: 0–7 normal, 8–15 bright.
    Ansi(u8),
    TrueColor {
        r: u8,
        g: u8,
        b: u8,
    },
}

impl Color {
    /// Approximate RGB values of the standard colours, used to downgrade RGB colours.
    const ANSI_RGB: [(u8, u8, u8); 16] = [
        (0, 0, 0),
        (205, 0, 0),
        (0, 205, 0),
        (205, 205, 0),
        (0, 0, 238),
        (205, 0, 205),
        (0, 205, 205),
        (229, 229, 229),
        (127, 127, 127),
        (255, 0, 0),
        (0, 255, 0),
        (255, 255, 0),
        (92, 92, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ];

    /// Writes the SGR parameter, using `base` 30 for foregrounds or 40 for backgrounds.
    fn write_sgr(self, output: &mut String, base: u8, truecolor: bool) {
        match self {
            Self::Ansi(index) if index < 8 => push_number(output, base + index),
            Self::Ansi(index) => push_number(output, base + 60 + index - 8),
            Self::TrueColor { r, g, b } if truecolor => {
                let _ = write!(output, "{};2;{r};{g};{b}", base + 8);
            }
            Self::TrueColor { .. } => self.nearest_ansi().write_sgr(output, base, truecolor),
        }
    }

    fn nearest_ansi(self) -> Self {
        let Self::TrueColor { r, g, b } = self else {
            return self;
        };
        let distance = |&(ar, ag, ab): &(u8, u8, u8)| {
            u32::from(ar.abs_diff(r)).pow(2)
                + u32::from(ag.abs_diff(g)).pow(2)
                + u32::from(ab.abs_diff(b)).pow(2)
        };
        // `min_by_key` keeps the first of equally close colours.
        let index = (0..16u8)
            .min_by_key(|&index| distance(&Self::ANSI_RGB[usize::from(index)]))
            .unwrap_or(0);
        Self::Ansi(index)
    }
}

fn push_number(output: &mut String, number: u8) {
    let _ = write!(output, "{number}");
}

/// Whether and how to colour output, read from the environment once per run.
struct ColorSupport {
    enabled: bool,
    truecolor: bool,
}

impl ColorSupport {
    /// `CLICOLOR_FORCE` wins, then `NO_COLOR`, then `CLICOLOR` combined with a terminal check.
    fn get() -> &'static Self {
        static SUPPORT: OnceLock<ColorSupport> = OnceLock::new();
        SUPPORT.get_or_init(|| {
            let set = |name| std::env::var(name).ok().map(|value| value != "0");
            let enabled = if set("CLICOLOR_FORCE") == Some(true) {
                true
            } else if std::env::var_os("NO_COLOR").is_some() {
                false
            } else {
                set("CLICOLOR").unwrap_or(true) && std::io::stdout().is_terminal()
            };
            let truecolor = matches!(
                std::env::var("COLORTERM").as_deref(),
                Ok("truecolor" | "24bit")
            );
            Self { enabled, truecolor }
        })
    }
}

/// A validated terminal style. Omitted overlay properties inherit their base.
#[derive(Debug, Clone, Copy, Default)]
pub struct Style {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub attributes: FontStyles,
    foreground_present: bool,
    background_present: bool,
    attributes_present: bool,
    inline_attributes: bool,
}

impl Style {
    pub fn paint(&self, text: &str) -> String {
        let mut output = String::new();
        self.paint_into(&mut output, text);
        output
    }

    /// Appends `text` wrapped in this style's escape sequences.
    pub fn paint_into(&self, output: &mut String, text: &str) {
        if text.is_empty() {
            return;
        }
        let support = ColorSupport::get();
        if !support.enabled
            || (self.foreground.is_none() && self.background.is_none() && self.attributes.0 == 0)
        {
            output.push_str(text);
            return;
        }
        let start = output.len();
        output.push_str("\x1b[");
        let mut separator = false;
        for attribute in self.attributes.iter() {
            if separator {
                output.push(';');
            }
            push_number(output, attribute.sgr_code());
            separator = true;
        }
        for (color, base) in [(self.background, 40), (self.foreground, 30)] {
            if let Some(color) = color {
                if separator {
                    output.push(';');
                }
                color.write_sgr(output, base, support.truecolor);
                separator = true;
            }
        }
        output.push('m');
        if text.contains(RESET) {
            // Restore this style after any reset embedded in the text.
            let prefix = output[start..].to_string();
            output.push_str(&text.replace(RESET, &format!("{RESET}{prefix}")));
        } else {
            output.push_str(text);
        }
        output.push_str(RESET);
    }
}

/// Resolved semantic styles; raw config keys never reach the formatters.
#[derive(Debug, Clone)]
pub struct Theme {
    styles: HashMap<String, Style>,
    legacy_field_attributes: HashMap<String, FontStyles>,
    plain: Style,
}

impl Default for Theme {
    fn default() -> Self {
        let document: Table = toml::from_str(include_str!("../themes/default.toml"))
            .expect("built-in theme must be valid TOML");
        let mut theme = Self {
            styles: HashMap::new(),
            legacy_field_attributes: HashMap::new(),
            plain: Style::default(),
        };
        let mut diagnostics = Vec::new();
        theme.apply_table(
            document["style"].as_table().expect("built-in style table"),
            "",
            false,
            true,
            &mut diagnostics,
        );
        assert!(
            diagnostics.is_empty(),
            "invalid built-in theme: {diagnostics:?}"
        );
        theme
    }
}

impl Theme {
    pub fn style(&self, role: &str) -> &Style {
        self.styles.get(role).unwrap_or(&self.plain)
    }

    pub fn overlay(&self, base: &Style, role: &str) -> Style {
        let overlay = self.style(role);
        let mut result = *base;
        if overlay.foreground_present {
            result.foreground = overlay.foreground;
            result.foreground_present = true;
        }
        if overlay.background_present {
            result.background = overlay.background;
            result.background_present = true;
        }
        if overlay.attributes_present {
            result.attributes = overlay.attributes;
            result.attributes_present = true;
            result.inline_attributes = overlay.inline_attributes;
        }
        result
    }

    pub fn field_attributes_override(&self, field: &str) -> Option<FontStyles> {
        self.legacy_field_attributes.get(field).copied()
    }

    /// Legacy whole-column attributes apply only in long output; inline styles win.
    pub fn field_style(&self, base: &Style, field: &str) -> Style {
        let mut result = *base;
        if !base.inline_attributes {
            if let Some(attributes) = self.field_attributes_override(field) {
                result.attributes = attributes;
            }
        }
        result
    }

    pub(crate) fn apply_document(
        &mut self,
        document: &Table,
        inline: bool,
        diagnostics: &mut Vec<String>,
    ) {
        if let Some(value) = document.get("style") {
            if let Some(styles) = value.as_table() {
                self.apply_table(styles, "", inline, false, diagnostics);
            } else {
                diagnostics.push("style must be a table".into());
            }
        }
    }

    fn apply_table(
        &mut self,
        table: &Table,
        prefix: &str,
        inline: bool,
        builtin: bool,
        diagnostics: &mut Vec<String>,
    ) {
        for (name, value) in table {
            let role = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}.{name}")
            };
            let Some(properties) = value.as_table() else {
                diagnostics.push(format!("style.{role} must be a style table"));
                continue;
            };
            let is_style = self.styles.contains_key(&role)
                || builtin
                    && properties.keys().any(|key| {
                        matches!(key.as_str(), "foreground" | "background" | "attributes")
                    });
            if is_style {
                let style = self.styles.entry(role.clone()).or_default();
                update_style(style, properties, inline, &role, diagnostics);
            } else if builtin
                || self
                    .styles
                    .keys()
                    .any(|known| known.starts_with(&format!("{role}.")))
            {
                self.apply_table(properties, &role, inline, builtin, diagnostics);
            } else {
                diagnostics.push(format!("unknown style role: style.{role}"));
            }
        }
    }

    pub(crate) fn apply_legacy(&mut self, document: &Table, diagnostics: &mut Vec<String>) {
        if let Some(colors) = document.get("colors").and_then(Value::as_table) {
            for (name, value) in colors {
                if name == "git" {
                    if let Some(git) = value.as_table() {
                        for (status, color) in git {
                            self.legacy_property(
                                &format!("git.{status}"),
                                "foreground",
                                color,
                                diagnostics,
                            );
                        }
                    }
                } else {
                    self.legacy_filename_property(name, "foreground", value, diagnostics);
                }
            }
        }
        if let Some(styles) = document.get("styles").and_then(Value::as_table) {
            for (name, attributes) in styles {
                self.legacy_filename_property(name, "attributes", attributes, diagnostics);
            }
        }
        if let Some(colors) = document
            .get("icons")
            .and_then(|icons| icons.get("colors"))
            .and_then(Value::as_table)
        {
            for (name, color) in colors {
                self.legacy_property(&format!("icons.{name}"), "foreground", color, diagnostics);
            }
        }
        let Some(display) = document.get("display").and_then(Value::as_table) else {
            return;
        };
        if let Some(color) = display.get("long_format_title_color") {
            self.legacy_property("header", "foreground", color, diagnostics);
        }
        if let Some(attributes) = display.get("long_format_title_style") {
            self.legacy_property("header", "attributes", attributes, diagnostics);
        }
        if let Some(fields) = display.get("long_format_styles").and_then(Value::as_table) {
            for (field, value) in fields {
                if !matches!(
                    field.as_str(),
                    "filename"
                        | "permissions"
                        | "nlink"
                        | "owner"
                        | "group"
                        | "size"
                        | "modified"
                        | "git"
                ) {
                    diagnostics.push(format!("unknown long format style field: {field}"));
                    continue;
                }
                match value.clone().try_into::<Vec<FontStyle>>() {
                    Ok(attributes) => {
                        self.legacy_field_attributes
                            .insert(field.clone(), FontStyles::from_slice(&attributes));
                    }
                    Err(error) => {
                        diagnostics.push(format!("display.long_format_styles.{field}: {error}"))
                    }
                }
            }
        }
    }

    fn legacy_filename_property(
        &mut self,
        name: &str,
        property: &str,
        value: &Value,
        diagnostics: &mut Vec<String>,
    ) {
        let mut roles = vec![filename_role(name)];
        if name == "regular" {
            roles.extend(
                self.styles
                    .keys()
                    .filter(|role| role.starts_with("file_type."))
                    .cloned(),
            );
        } else if name == "symlink" {
            roles.push("symlink_path".to_string());
        }
        for role in roles {
            self.legacy_property(&role, property, value, diagnostics);
        }
    }

    fn legacy_property(
        &mut self,
        role: &str,
        property: &str,
        value: &Value,
        diagnostics: &mut Vec<String>,
    ) {
        let Some(style) = self.styles.get_mut(role) else {
            diagnostics.push(format!("unknown legacy style role: {role}"));
            return;
        };
        let properties = Table::from_iter([(property.to_string(), value.clone())]);
        update_style(style, &properties, false, role, diagnostics);
    }
}

fn filename_role(name: &str) -> String {
    if name == "hidden" {
        "filenames.hidden".to_string()
    } else {
        format!("filekinds.{name}")
    }
}

fn update_style(
    style: &mut Style,
    properties: &Table,
    inline: bool,
    role: &str,
    diagnostics: &mut Vec<String>,
) {
    for (property, value) in properties {
        match property.as_str() {
            "foreground" | "background" => {
                let parsed = value
                    .as_str()
                    .ok_or("must be a colour string")
                    .and_then(parse_color);
                match parsed {
                    Ok(color) if property == "foreground" => {
                        style.foreground = color;
                        style.foreground_present = true;
                    }
                    Ok(color) => {
                        style.background = color;
                        style.background_present = true;
                    }
                    Err(error) => diagnostics.push(format!("style.{role}.{property}: {error}")),
                }
            }
            "attributes" => match value.clone().try_into::<Vec<FontStyle>>() {
                Ok(attributes) => {
                    style.attributes = FontStyles::from_slice(&attributes);
                    style.attributes_present = true;
                    style.inline_attributes = inline;
                }
                Err(error) => diagnostics.push(format!("style.{role}.attributes: {error}")),
            },
            _ => diagnostics.push(format!("unknown style property: style.{role}.{property}")),
        }
    }
}

fn parse_color(value: &str) -> Result<Option<Color>, &'static str> {
    let value = value.to_ascii_lowercase();
    let named = match value.as_str() {
        "default" => return Ok(None),
        "black" => Some(Color::Ansi(0)),
        "red" => Some(Color::Ansi(1)),
        "green" => Some(Color::Ansi(2)),
        "yellow" => Some(Color::Ansi(3)),
        "blue" => Some(Color::Ansi(4)),
        "magenta" | "purple" => Some(Color::Ansi(5)),
        "cyan" => Some(Color::Ansi(6)),
        "white" => Some(Color::Ansi(7)),
        "bright_black" => Some(Color::Ansi(8)),
        "bright_red" => Some(Color::Ansi(9)),
        "bright_green" => Some(Color::Ansi(10)),
        "bright_yellow" => Some(Color::Ansi(11)),
        "bright_blue" => Some(Color::Ansi(12)),
        "bright_magenta" | "bright_purple" => Some(Color::Ansi(13)),
        "bright_cyan" => Some(Color::Ansi(14)),
        "bright_white" => Some(Color::Ansi(15)),
        _ => None,
    };
    if named.is_some() {
        return Ok(named);
    }
    let digits = value.strip_prefix('#').unwrap_or(&value);
    let digits = if value.starts_with('#') && digits.len() == 3 {
        digits
            .chars()
            .flat_map(|digit| [digit, digit])
            .collect::<String>()
    } else {
        digits.to_string()
    };
    if digits.len() != 6 || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("expected a named terminal colour, default, #RGB or #RRGGBB");
    }
    let number = u32::from_str_radix(&digits, 16).map_err(|_| "invalid RGB colour")?;
    Ok(Some(Color::TrueColor {
        r: ((number >> 16) & 0xff) as u8,
        g: ((number >> 8) & 0xff) as u8,
        b: (number & 0xff) as u8,
    }))
}
