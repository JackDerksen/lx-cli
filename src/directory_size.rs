use crate::dir_scan::scan;
use crate::file_entry::FileEntry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Default)]
struct ContentsSize {
    bytes: u64,
    incomplete: bool,
}

impl ContentsSize {
    fn add(&mut self, other: Self) {
        match self.bytes.checked_add(other.bytes) {
            Some(bytes) => self.bytes = bytes,
            None => self.incomplete = true,
        }
        self.incomplete |= other.incomplete;
    }
}

/// A directory being totalled: its files are summed on entry, and its subdirectories
/// wait their turn on the traversal stack.
struct DirectoryFrame {
    path: PathBuf,
    subdirectories: std::vec::IntoIter<PathBuf>,
    size: ContentsSize,
}

impl DirectoryFrame {
    fn new(path: PathBuf) -> Self {
        let mut size = ContentsSize::default();
        let mut subdirectories = Vec::new();
        let scanned = scan(
            &path,
            |_| true,
            |entry| match entry.and_then(|entry| Ok((entry.name, entry.stat?))) {
                Ok((name, stat)) if stat.is_dir() => subdirectories.push(path.join(name)),
                Ok((_, stat)) if stat.is_file() || stat.is_symlink() => size.add(ContentsSize {
                    bytes: stat.size,
                    incomplete: false,
                }),
                Ok(_) => {}
                Err(_) => size.incomplete = true,
            },
        );
        size.incomplete |= scanned.is_err();
        Self {
            path,
            subdirectories: subdirectories.into_iter(),
            size,
        }
    }
}

/// Contents totals shared by all entries in a listing, including nested trees.
#[derive(Default)]
pub struct DirectorySizeCache {
    totals: HashMap<PathBuf, ContentsSize>,
}

impl DirectorySizeCache {
    pub fn apply(&mut self, entries: &mut [FileEntry]) {
        for entry in entries {
            if entry.is_dir {
                let size = self.contents_size(&entry.full_path);
                entry.size = size.bytes;
                entry.size_incomplete = size.incomplete;
            }
        }
    }

    fn contents_size(&mut self, path: &Path) -> ContentsSize {
        if let Some(size) = self.totals.get(path) {
            return *size;
        }

        // An explicit stack avoids overflowing the call stack on deep directories.
        let mut stack = vec![DirectoryFrame::new(path.to_path_buf())];
        while let Some(frame) = stack.last_mut() {
            if let Some(child_path) = frame.subdirectories.next() {
                if let Some(size) = self.totals.get(&child_path) {
                    frame.size.add(*size);
                } else {
                    stack.push(DirectoryFrame::new(child_path));
                }
                continue;
            }
            let completed = stack.pop().expect("directory frame exists");
            self.totals.insert(completed.path, completed.size);
            if let Some(parent) = stack.last_mut() {
                parent.size.add(completed.size);
            } else {
                return completed.size;
            }
        }
        unreachable!("root directory frame returns its total")
    }
}
