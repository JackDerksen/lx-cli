use crate::file_entry::FileEntry;

#[derive(Clone, Copy)]
pub struct EntryFilter {
    files: bool,
    directories: bool,
}

impl EntryFilter {
    pub fn new(files: bool, directories: bool) -> Self {
        Self { files, directories }
    }

    pub fn includes(self, entry: &FileEntry) -> bool {
        match (self.files, self.directories) {
            (true, false) => !entry.is_dir,
            (false, true) => entry.is_dir,
            _ => true,
        }
    }

    pub fn apply(self, entries: Vec<FileEntry>) -> Vec<FileEntry> {
        entries
            .into_iter()
            .filter(|entry| self.includes(entry))
            .collect()
    }
}
