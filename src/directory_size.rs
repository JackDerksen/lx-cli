use crate::file_entry::FileEntry;
use std::collections::HashMap;
use std::fs::{self, ReadDir};
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

struct DirectoryFrame {
    path: PathBuf,
    entries: Option<ReadDir>,
    size: ContentsSize,
}

impl DirectoryFrame {
    fn new(path: PathBuf) -> Self {
        let entries = fs::read_dir(&path).ok();
        Self {
            path,
            size: ContentsSize {
                incomplete: entries.is_none(),
                ..ContentsSize::default()
            },
            entries,
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
            match frame.entries.as_mut().and_then(Iterator::next) {
                Some(Ok(entry)) => {
                    let child_path = entry.path();
                    match fs::symlink_metadata(&child_path) {
                        Ok(metadata) if metadata.is_dir() => {
                            if let Some(size) = self.totals.get(&child_path) {
                                frame.size.add(*size);
                            } else {
                                stack.push(DirectoryFrame::new(child_path));
                            }
                        }
                        Ok(metadata) if metadata.is_file() || metadata.is_symlink() => {
                            frame.size.add(ContentsSize {
                                bytes: metadata.len(),
                                incomplete: false,
                            });
                        }
                        Ok(_) => {}
                        Err(_) => frame.size.incomplete = true,
                    }
                }
                Some(Err(_)) => frame.size.incomplete = true,
                None => {
                    let completed = stack.pop().expect("directory frame exists");
                    self.totals.insert(completed.path, completed.size);
                    if let Some(parent) = stack.last_mut() {
                        parent.size.add(completed.size);
                    } else {
                        return completed.size;
                    }
                }
            }
        }
        unreachable!("root directory frame returns its total")
    }
}
