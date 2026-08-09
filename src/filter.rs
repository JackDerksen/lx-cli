use crate::file_entry::FileEntry;

#[derive(Clone)]
pub struct EntryFilter {
    files: bool,
    directories: bool,
    excluded_patterns: Vec<String>,
}

impl EntryFilter {
    pub fn new(files: bool, directories: bool, excluded_patterns: Vec<String>) -> Self {
        Self {
            files,
            directories,
            excluded_patterns,
        }
    }

    pub fn includes(&self, entry: &FileEntry) -> bool {
        let matches_type = match (self.files, self.directories) {
            (true, false) => !entry.is_dir,
            (false, true) => entry.is_dir,
            _ => true,
        };

        matches_type
            && !self
                .excluded_patterns
                .iter()
                .any(|pattern| matches_pattern(pattern, &entry.path.to_string_lossy()))
    }

    pub fn apply(&self, entries: Vec<FileEntry>) -> Vec<FileEntry> {
        entries
            .into_iter()
            .filter(|entry| self.includes(entry))
            .collect()
    }
}

fn matches_pattern(pattern: &str, name: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let name: Vec<char> = name.chars().collect();
    let (mut pattern_index, mut name_index) = (0, 0);
    let (mut star_index, mut star_match_index) = (None, 0);

    while name_index < name.len() {
        if pattern_index < pattern.len()
            && (pattern[pattern_index] == '?' || pattern[pattern_index] == name[name_index])
        {
            pattern_index += 1;
            name_index += 1;
        } else if pattern_index < pattern.len() && pattern[pattern_index] == '*' {
            star_index = Some(pattern_index);
            pattern_index += 1;
            star_match_index = name_index;
        } else if let Some(star) = star_index {
            pattern_index = star + 1;
            star_match_index += 1;
            name_index = star_match_index;
        } else {
            return false;
        }
    }

    while pattern_index < pattern.len() && pattern[pattern_index] == '*' {
        pattern_index += 1;
    }

    pattern_index == pattern.len()
}
