use crate::config::Config;
use crate::file_entry::FileEntry;
use crate::git::GitStatus;
use crate::theme::Style;
use std::sync::OnceLock;
use unicode_width::UnicodeWidthStr;

/// Visible width stays separate from ANSI escapes and column padding.
#[derive(Default)]
pub(super) struct Cell {
    pub text: String,
    pub width: usize,
}

impl Cell {
    fn append(&mut self, text: &str, style: &Style) {
        self.width += UnicodeWidthStr::width(text);
        self.text.push_str(&style.paint(text));
    }

    pub fn padded(self, width: usize, right: bool) -> String {
        let padding = " ".repeat(width.saturating_sub(self.width));
        if right {
            format!("{padding}{}", self.text)
        } else {
            format!("{}{padding}", self.text)
        }
    }
}

pub(super) struct Renderer<'config> {
    config: &'config Config,
}

impl<'config> Renderer<'config> {
    pub fn new(config: &'config Config) -> Self {
        Self { config }
    }

    pub fn paint(&self, role: &str, text: &str) -> String {
        self.config.theme.style(role).paint(text)
    }

    fn style(&self, role: &str, field: Option<&str>) -> Style {
        let style = self.config.theme.style(role);
        field
            .map(|field| self.config.theme.field_style(style, field))
            .unwrap_or_else(|| style.clone())
    }

    fn cell(&self, role: &str, text: &str, field: Option<&str>) -> Cell {
        let mut cell = Cell::default();
        cell.append(text, &self.style(role, field));
        cell
    }

    pub fn filename(&self, entry: &FileEntry, long: bool) -> Cell {
        let theme = &self.config.theme;
        let mut style = theme.style(entry.style_role()).clone();
        if self.config.display.language_colors && entry.style_role() == "filekinds.regular" {
            if let Some(category) = crate::icon::file_category(
                entry
                    .full_path
                    .file_name()
                    .unwrap_or(entry.full_path.as_os_str()),
            ) {
                style = theme.overlay(&style, category);
            }
        }
        if entry.is_hidden && entry.symlink.is_none() {
            style = theme.overlay(&style, "filenames.hidden");
        }
        let field = long.then_some("filename");
        if let Some(field) = field {
            style = theme.field_style(&style, field);
        }
        let mut cell = Cell::default();
        let icon = entry.get_icon_custom(&self.config.icons);
        if !icon.is_empty() {
            let role = if let Some(link) = &entry.symlink {
                if link.is_broken {
                    "icons.broken_symlink"
                } else {
                    "icons.symlink"
                }
            } else if entry.is_hidden {
                "icons.hidden"
            } else if entry.is_dir {
                "icons.directory"
            } else if entry.is_executable {
                "icons.executable"
            } else {
                "icons.regular"
            };
            cell.append(&icon, theme.style(role));
            cell.text.push(' ');
            cell.width += 1;
        }
        cell.append(&entry.path.to_string_lossy(), &style);
        if let Some(link) = &entry.symlink {
            cell.append(" -> ", &self.style("punctuation", field));
            let target_role = if link.is_broken {
                "filekinds.broken_symlink"
            } else {
                "symlink_path"
            };
            cell.append(
                &link.target.to_string_lossy(),
                &self.style(target_role, field),
            );
            if link.is_broken {
                cell.append(" [broken]", &self.style(target_role, field));
            }
        }
        cell
    }

    pub fn git(&self, status: GitStatus, long: bool) -> Cell {
        self.cell(status.style_role(), status.badge(), long.then_some("git"))
    }

    pub fn field(&self, entry: &FileEntry, field: &str, modified: &str) -> Cell {
        match field {
            "filename" => self.filename(entry, true),
            "git" => entry
                .git_status
                .map(|status| self.git(status, true))
                .unwrap_or_default(),
            "permissions" => {
                let mut cell = Cell::default();
                for (index, character) in entry.format_permissions().chars().enumerate() {
                    let role = if index == 0 {
                        "permissions.type"
                    } else {
                        match character {
                            'r' => "permissions.read",
                            'w' => "permissions.write",
                            'x' => "permissions.execute",
                            's' | 'S' | 't' | 'T' => "permissions.special",
                            _ => "permissions.absent",
                        }
                    };
                    cell.append(&character.to_string(), &self.style(role, Some(field)));
                }
                cell
            }
            "size" => {
                let text = entry.format_size();
                let (number, unit) = text.split_at(text.len() - 1);
                let scale = match unit {
                    "B" => "bytes",
                    "K" => "kilobytes",
                    "M" => "megabytes",
                    _ => "gigabytes",
                };
                let mut cell = self.cell(&format!("size.{scale}.number"), number, Some(field));
                cell.append(
                    unit,
                    &self.style(&format!("size.{scale}.unit"), Some(field)),
                );
                cell
            }
            "owner" | "group" => {
                let identity = CurrentIdentity::get();
                let role = if field == "owner" {
                    if entry.user_id == 0 {
                        "users.user_root"
                    } else if entry.user_id == identity.user_id {
                        "users.user_you"
                    } else {
                        "users.user_other"
                    }
                } else if entry.group_id == 0 {
                    "users.group_root"
                } else if identity.groups.contains(&entry.group_id) {
                    "users.group_yours"
                } else {
                    "users.group_other"
                };
                self.cell(
                    role,
                    if field == "owner" {
                        &entry.owner
                    } else {
                        &entry.group
                    },
                    Some(field),
                )
            }
            "nlink" => self.cell(
                if entry.nlink > 1 {
                    "links.multiple"
                } else {
                    "links.normal"
                },
                &entry.nlink.to_string(),
                Some(field),
            ),
            "modified" => self.cell("date", modified, Some(field)),
            _ => Cell::default(),
        }
    }
}

struct CurrentIdentity {
    user_id: u32,
    groups: Vec<u32>,
}

impl CurrentIdentity {
    fn get() -> &'static Self {
        static IDENTITY: OnceLock<CurrentIdentity> = OnceLock::new();
        IDENTITY.get_or_init(|| {
            // POSIX queries write at most the supplied group buffer capacity.
            unsafe {
                let count = libc::getgroups(0, std::ptr::null_mut());
                let mut groups = vec![0; count.max(0) as usize];
                let actual_count = libc::getgroups(count.max(0), groups.as_mut_ptr());
                groups.truncate(actual_count.max(0) as usize);
                groups.push(libc::getegid());
                Self {
                    user_id: libc::geteuid(),
                    groups,
                }
            }
        })
    }
}
