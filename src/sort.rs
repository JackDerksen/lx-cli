/// Handles sorting of file entries.
use crate::file_entry::FileEntry;
use crate::reader::DiscoveredEntry;
use clap::ValueEnum;
use serde::Deserialize;
use std::cmp::Ordering;

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
    match options.field {
        Some(field) => sort_by_field(entries, field, options.order),
        None => sort_default(entries),
    }
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

pub fn sort_discovered_entries(
    entries: &mut [DiscoveredEntry],
    options: SortOptions,
    default_sort: DefaultSort,
) {
    if let Some(field) = options.field {
        entries.sort_by(|left, right| {
            compare_entries(&left.entry, &right.entry, field, options.order)
        });
        return;
    }

    match default_sort {
        DefaultSort::Name => {
            entries.sort_by_cached_key(|entry| entry.entry.path.to_string_lossy().to_lowercase());
        }
        DefaultSort::TypeThenName => {
            entries.sort_by_cached_key(|entry| {
                (
                    entry.entry.get_file_type(),
                    entry.entry.path.to_string_lossy().to_lowercase(),
                )
            });
        }
    }
}

fn sort_by_field(entries: &mut [FileEntry], field: SortField, order: SortOrder) {
    entries.sort_by(|left, right| compare_entries(left, right, field, order));
}

fn compare_entries(
    left: &FileEntry,
    right: &FileEntry,
    field: SortField,
    order: SortOrder,
) -> Ordering {
    let comparison = match field {
        SortField::Name => entry_name(left).cmp(&entry_name(right)),
        SortField::Size => left.size.cmp(&right.size),
        SortField::Modified => left.modified.cmp(&right.modified),
        SortField::Type => left.get_file_type().cmp(&right.get_file_type()),
        SortField::Permissions => left.format_permissions().cmp(&right.format_permissions()),
        SortField::Links => left.nlink.cmp(&right.nlink),
        SortField::Owner => left.owner.cmp(&right.owner),
        SortField::Group => left.group.cmp(&right.group),
    }
    .then_with(|| entry_name(left).cmp(&entry_name(right)));

    match order {
        SortOrder::Asc => comparison,
        SortOrder::Desc => comparison.reverse(),
    }
}

fn entry_name(entry: &FileEntry) -> String {
    entry.path.to_string_lossy().to_lowercase()
}
