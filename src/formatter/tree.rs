use crate::directory_size::DirectorySizeCache;
use crate::file_entry::FileEntry;
use crate::filter::EntryFilter;
use crate::formatter::TreeOptions;
use crate::git::GitRepository;
use crate::reader::{MetadataMode, read_directory_entries, report_error};
use crate::sort::{DefaultSort, SortOptions, sort_entries_with_default};
use std::io;
use std::path::Path;

pub struct TreeEntry {
    pub entry: FileEntry,
    pub branch: String,
}

pub struct TreeRenderer<'a> {
    style: &'a str,
    indents_for_icons: bool,
    filter: &'a EntryFilter,
    sort: SortOptions,
    default_sort: DefaultSort,
    git: Option<&'a GitRepository>,
    options: &'a TreeOptions<'a>,
}

impl<'a> TreeRenderer<'a> {
    pub fn new(
        style: &'a str,
        indents_for_icons: bool,
        filter: &'a EntryFilter,
        sort: SortOptions,
        default_sort: DefaultSort,
        git: Option<&'a GitRepository>,
        options: &'a TreeOptions<'a>,
    ) -> Self {
        Self {
            style,
            indents_for_icons,
            filter,
            sort,
            default_sort,
            git,
            options,
        }
    }

    pub fn collect(
        &self,
        path: &Path,
        metadata_mode: MetadataMode,
        directory_sizes: Option<&mut DirectorySizeCache>,
    ) -> io::Result<Vec<TreeEntry>> {
        let mut tree_entries = Vec::new();
        let canonical_path = self
            .git
            .or(self.options.ignored)
            .map(|_| path.canonicalize())
            .transpose()?;
        let path = canonical_path.as_deref().unwrap_or(path);
        self.collect_directory(
            path,
            metadata_mode,
            "",
            self.options.depth,
            &mut tree_entries,
            directory_sizes,
        )?;
        Ok(tree_entries)
    }

    fn collect_directory(
        &self,
        path: &Path,
        metadata_mode: MetadataMode,
        prefix: &str,
        remaining_depth: Option<usize>,
        tree_entries: &mut Vec<TreeEntry>,
        mut directory_sizes: Option<&mut DirectorySizeCache>,
    ) -> io::Result<()> {
        if remaining_depth == Some(0)
            || self
                .options
                .ignored
                .is_some_and(|repository| repository.is_ignored(path))
        {
            return Ok(());
        }
        let mut entries =
            match read_directory_entries(path, self.options.show_hidden, metadata_mode) {
                Ok(entries) => entries,
                Err(error) => {
                    report_error(format_args!(
                        "cannot open directory '{}': {error}",
                        path.display()
                    ));
                    return Ok(());
                }
            };
        entries.retain(|entry| {
            self.filter.includes(entry)
                && !self
                    .options
                    .ignored
                    .is_some_and(|repository| repository.is_ignored(&entry.full_path))
        });
        if let Some(repository) = self.git {
            for entry in &mut entries {
                entry.git_status = Some(repository.status(&entry.full_path));
            }
        }
        if let Some(sizes) = &mut directory_sizes {
            sizes.apply(&mut entries);
        }
        sort_entries_with_default(&mut entries, self.sort, self.default_sort);

        for (index, entry) in entries.iter().enumerate() {
            let is_last = index == entries.len() - 1;
            let (connector, child_prefix) = self.branch(prefix, is_last);

            tree_entries.push(TreeEntry {
                entry: entry.clone(),
                branch: format!("{prefix}{connector}"),
            });

            if entry.is_dir {
                self.collect_directory(
                    &entry.full_path,
                    metadata_mode,
                    &child_prefix,
                    remaining_depth.map(|depth| depth - 1),
                    tree_entries,
                    directory_sizes.as_deref_mut(),
                )?;
            }
        }

        Ok(())
    }

    fn branch(&self, prefix: &str, is_last: bool) -> (&str, String) {
        let content_indent = if self.indents_for_icons { "  " } else { " " };

        if self.style == "ascii" {
            if is_last {
                ("╰─", format!("{prefix}  {content_indent}"))
            } else {
                ("├─", format!("{prefix}│ {content_indent}"))
            }
        } else {
            ("", format!("{prefix}  {content_indent}"))
        }
    }
}
