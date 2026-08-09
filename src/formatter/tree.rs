use crate::file_entry::FileEntry;
use crate::reader::{DiscoveredEntry, MetadataMode, read_directory_entries};
use std::io;
use std::path::Path;

pub struct TreeEntry {
    pub entry: FileEntry,
    pub branch: String,
}

pub struct TreeRenderer<'a> {
    style: &'a str,
    indents_for_icons: bool,
}

impl<'a> TreeRenderer<'a> {
    pub fn new(style: &'a str, indents_for_icons: bool) -> Self {
        Self {
            style,
            indents_for_icons,
        }
    }

    pub fn collect(
        &self,
        path: &Path,
        show_hidden: bool,
        metadata_mode: MetadataMode,
        sort_entries: fn(&mut [DiscoveredEntry]),
    ) -> io::Result<Vec<TreeEntry>> {
        let mut tree_entries = Vec::new();
        self.collect_directory(
            path,
            show_hidden,
            metadata_mode,
            sort_entries,
            "",
            &mut tree_entries,
        )?;
        Ok(tree_entries)
    }

    fn collect_directory(
        &self,
        path: &Path,
        show_hidden: bool,
        metadata_mode: MetadataMode,
        sort_entries: fn(&mut [DiscoveredEntry]),
        prefix: &str,
        tree_entries: &mut Vec<TreeEntry>,
    ) -> io::Result<()> {
        let mut entries = read_directory_entries(path, show_hidden, metadata_mode)?;
        sort_entries(&mut entries);

        for (index, discovered_entry) in entries.iter().enumerate() {
            let is_last = index == entries.len() - 1;
            let (connector, child_prefix) = self.branch(prefix, is_last);

            tree_entries.push(TreeEntry {
                entry: discovered_entry.entry.clone(),
                branch: format!("{prefix}{connector}"),
            });

            if discovered_entry.entry.is_dir {
                self.collect_directory(
                    &discovered_entry.full_path,
                    show_hidden,
                    metadata_mode,
                    sort_entries,
                    &child_prefix,
                    tree_entries,
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
