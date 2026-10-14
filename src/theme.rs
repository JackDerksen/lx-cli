use crate::config::{FontStyle, apply_font_styles};
use colored::{Color, Colorize};
use std::collections::HashMap;
use toml::{Table, Value};

/// A validated terminal style. Omitted overlay properties inherit their base.
#[derive(Debug, Clone, Default)]
pub struct Style {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub attributes: Vec<FontStyle>,
    foreground_present: bool,
    background_present: bool,
    attributes_present: bool,
    inline_attributes: bool,
}

impl Style {
    pub fn paint(&self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }
        let mut output = text.normal();
        if let Some(color) = self.foreground {
            output = output.color(color);
        }
        if let Some(color) = self.background {
            output = output.on_color(color);
        }
        apply_font_styles(output, &self.attributes).to_string()
    }
}

/// Resolved semantic styles; raw config keys never reach the formatters.
#[derive(Debug, Clone)]
pub struct Theme {
    styles: HashMap<String, Style>,
    legacy_field_attributes: HashMap<String, Vec<FontStyle>>,
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
        let mut result = base.clone();
        if overlay.foreground_present {
            result.foreground = overlay.foreground;
            result.foreground_present = true;
        }
        if overlay.background_present {
            result.background = overlay.background;
            result.background_present = true;
        }
        if overlay.attributes_present {
            result.attributes.clone_from(&overlay.attributes);
            result.attributes_present = true;
            result.inline_attributes = overlay.inline_attributes;
        }
        result
    }

    pub fn field_attributes_override(&self, field: &str) -> Option<&[FontStyle]> {
        self.legacy_field_attributes.get(field).map(Vec::as_slice)
    }

    /// Legacy whole-column attributes apply only in long output; inline styles win.
    pub fn field_style(&self, base: &Style, field: &str) -> Style {
        let mut result = base.clone();
        if !base.inline_attributes {
            if let Some(attributes) = self.field_attributes_override(field) {
                result.attributes = attributes.to_vec();
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
                            .insert(field.clone(), attributes);
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
                    style.attributes = attributes;
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
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" | "purple" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        "bright_black" => Some(Color::BrightBlack),
        "bright_red" => Some(Color::BrightRed),
        "bright_green" => Some(Color::BrightGreen),
        "bright_yellow" => Some(Color::BrightYellow),
        "bright_blue" => Some(Color::BrightBlue),
        "bright_magenta" | "bright_purple" => Some(Color::BrightMagenta),
        "bright_cyan" => Some(Color::BrightCyan),
        "bright_white" => Some(Color::BrightWhite),
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
