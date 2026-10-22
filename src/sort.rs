/// Handles sorting of file entries.
use crate::file_entry::FileEntry;
use crate::file_entry::FileType;
use clap::ValueEnum;
use serde::Deserialize;
use std::cmp::Reverse;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum SortField {
    #[serde(alias = "filename")]
    #[value(alias = "filename")]
    Name,
    Size,
    Modified,
    Type,
    Permissions,
    Links,
    Owner,
    Group,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SortOptions {
    pub field: Option<SortField>,
    pub order: SortOrder,
}

impl SortOptions {
    pub fn new(field: Option<SortField>, order: SortOrder) -> Self {
        Self { field, order }
    }

    pub fn is_custom(self) -> bool {
        self.field.is_some()
    }

    pub fn requires_full_metadata(self) -> bool {
        matches!(self.field, Some(SortField::Owner | SortField::Group))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DefaultSort {
    Name,
    TypeThenName,
}

/// Applies a requested sort, or the standard directory/executable/file ordering.
pub fn sort_entries(entries: &mut [FileEntry], options: SortOptions) {
    sort_entries_with_default(entries, options, DefaultSort::TypeThenName);
}

/// Default sort: by file type (directory, executable, regular), then alphabetically by name.
pub fn sort_default(entries: &mut [FileEntry]) {
    entries.sort_by_cached_key(|entry| {
        (
            entry.get_file_type(),
            entry.path.to_string_lossy().to_lowercase(),
        )
    });
}

pub fn sort_entries_with_default(
    entries: &mut [FileEntry],
    options: SortOptions,
    default_sort: DefaultSort,
) {
    if let Some(field) = options.field {
        sort_by_field(entries, field, options.order);
        return;
    }

    match default_sort {
        DefaultSort::Name => {
            entries.sort_by_cached_key(|entry| entry.path.to_string_lossy().to_lowercase());
        }
        DefaultSort::TypeThenName => {
            sort_default(entries);
        }
    }
}

/// One entry's value for the requested field, computed once per entry rather than per comparison.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum FieldKey {
    Number(u64),
    Time(SystemTime),
    Type(FileType),
    Text(String),
}

fn sort_by_field(entries: &mut [FileEntry], field: SortField, order: SortOrder) {
    let key = |entry: &FileEntry| {
        let value = match field {
            SortField::Name => FieldKey::Text(String::new()),
            SortField::Size => FieldKey::Number(entry.size),
            SortField::Modified => FieldKey::Time(entry.modified),
            SortField::Type => FieldKey::Type(entry.get_file_type()),
            SortField::Permissions => FieldKey::Text(entry.format_permissions()),
            SortField::Links => FieldKey::Number(entry.nlink),
            SortField::Owner => FieldKey::Text(entry.owner.clone()),
            SortField::Group => FieldKey::Text(entry.group.clone()),
        };
        // Ties fall back to the name; descending order reverses both.
        (value, entry_name(entry))
    };
    match order {
        SortOrder::Asc => entries.sort_by_cached_key(key),
        SortOrder::Desc => entries.sort_by_cached_key(|entry| Reverse(key(entry))),
    }
}

fn entry_name(entry: &FileEntry) -> String {
    entry.path.to_string_lossy().to_lowercase()
}
