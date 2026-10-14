use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A single state for an entry or directory summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitStatus {
    Clean,
    Modified,
    Staged,
    Untracked,
    Conflict,
    Mixed,
}

impl GitStatus {
    pub(crate) fn badge(self) -> &'static str {
        match self {
            Self::Clean => "-",
            Self::Modified => "M",
            Self::Staged => "S",
            Self::Untracked => "?",
            Self::Conflict => "!",
            Self::Mixed => "*",
        }
    }

    pub(crate) fn style_role(self) -> &'static str {
        match self {
            Self::Clean => "git.clean",
            Self::Modified => "git.modified",
            Self::Staged => "git.staged",
            Self::Untracked => "git.untracked",
            Self::Conflict => "git.conflict",
            Self::Mixed => "git.mixed",
        }
    }

    fn from_codes(codes: [u8; 2]) -> Self {
        match &codes {
            b"  " => Self::Clean,
            b"??" => Self::Untracked,
            b"DD" | b"AU" | b"UD" | b"UA" | b"DU" | b"AA" | b"UU" => Self::Conflict,
            [b' ', _] => Self::Modified,
            [_, b' '] => Self::Staged,
            _ => Self::Mixed,
        }
    }

    fn merge(&mut self, other: Self) {
        *self = match (*self, other) {
            (Self::Conflict, _) | (_, Self::Conflict) => Self::Conflict,
            (Self::Clean, status) | (status, Self::Clean) => status,
            (current, incoming) if current == incoming => current,
            _ => Self::Mixed,
        };
    }
}

/// One status snapshot for the repository containing the listing target.
#[derive(Debug)]
pub struct GitRepository {
    statuses: HashMap<PathBuf, GitStatus>,
    ignored: HashSet<PathBuf>,
}

impl GitRepository {
    /// Returns `None` outside a working tree; optionally collects ignored paths.
    pub fn load(target: &Path, include_ignored: bool) -> io::Result<Option<Self>> {
        let directory = if target.is_dir() && !target.is_symlink() {
            target
        } else {
            target
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or(Path::new("."))
        };
        let directory = directory.canonicalize()?;
        let Some(root) = directory
            .ancestors()
            .find(|path| path.join(".git").exists())
        else {
            return Ok(None);
        };

        let mut command = Command::new("git");
        command
            .arg("--no-optional-locks")
            .arg("-C")
            .arg(root)
            .args(["status", "--porcelain=v1", "-z", "--untracked-files=all"]);
        if include_ignored {
            command.arg("--ignored=matching");
        }
        let output = command.output()?;
        if !output.status.success() {
            return Err(io::Error::other(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }
        let mut repository = Self::from_porcelain(root, &output.stdout)?;
        if include_ignored {
            repository.ignored.insert(root.join(".git"));
        }
        Ok(Some(repository))
    }

    /// Annotates entries returned by `read_target` with this repository's status.
    pub fn annotate(&self, target: &Path, entries: &mut [crate::FileEntry]) -> io::Result<()> {
        let directory = if target.is_dir() && !target.is_symlink() {
            target
        } else {
            target
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or(Path::new("."))
        }
        .canonicalize()?;
        for entry in entries {
            entry.git_status = Some(self.status(&directory.join(&entry.path)));
        }
        Ok(())
    }

    pub(crate) fn status(&self, path: &Path) -> GitStatus {
        self.statuses.get(path).copied().unwrap_or(GitStatus::Clean)
    }

    pub(crate) fn is_ignored(&self, path: &Path) -> bool {
        path.ancestors()
            .any(|ancestor| self.ignored.contains(ancestor))
    }

    fn from_porcelain(root: &Path, output: &[u8]) -> io::Result<Self> {
        let mut repository = Self {
            statuses: HashMap::new(),
            ignored: HashSet::new(),
        };
        let mut records = output.split(|byte| *byte == 0);
        while let Some(record) = records.next() {
            if record.is_empty() {
                continue;
            }
            if record.len() < 4 || record[2] != b' ' {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid Git status record",
                ));
            }
            let codes = [record[0], record[1]];
            let path = root.join(OsStr::from_bytes(&record[3..]));
            if codes == *b"!!" {
                repository.ignored.insert(path);
                continue;
            }
            let status = GitStatus::from_codes(codes);
            repository.record_path(root, &path, status);

            if codes.iter().any(|code| matches!(code, b'R' | b'C')) {
                let original = records
                    .next()
                    .filter(|record| !record.is_empty())
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "missing Git rename source")
                    })?;
                if codes.contains(&b'R') {
                    let deleted = GitStatus::from_codes(
                        codes.map(|code| if code == b'R' { b'D' } else { b' ' }),
                    );
                    repository.record_path(root, &root.join(OsStr::from_bytes(original)), deleted);
                }
            }
        }
        Ok(repository)
    }

    fn record_path(&mut self, root: &Path, path: &Path, status: GitStatus) {
        self.statuses.insert(path.to_path_buf(), status);
        for ancestor in path
            .ancestors()
            .skip(1)
            .take_while(|ancestor| ancestor.starts_with(root))
        {
            self.statuses
                .entry(ancestor.to_path_buf())
                .or_insert(GitStatus::Clean)
                .merge(status);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GitRepository, GitStatus};
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    #[test]
    fn parses_paths_and_aggregates_changes_without_marking_clean_siblings() {
        let root = Path::new("/repo");
        let repository = GitRepository::from_porcelain(root, b" M src/space name\0A  src/added\0?? src/new/nested/file\0R  destination/renamed\0source/old\0UU conflict\0 M invalid-\xff\0D  deleted/staged\0 D deleted/unstaged\0!! cache/\0!! ignored-\xff\0").expect("parse status");
        assert!(repository.is_ignored(Path::new("/repo/cache/nested/file")));
        assert!(!repository.is_ignored(Path::new("/repo/cache-other")));
        assert!(repository.is_ignored(&root.join(std::ffi::OsStr::from_bytes(b"ignored-\xff"))));
        assert_eq!(
            repository.status(Path::new("/repo/cache")),
            GitStatus::Clean
        );
        assert_eq!(
            repository.status(Path::new("/repo/src/space name")),
            GitStatus::Modified
        );
        assert_eq!(repository.status(Path::new("/repo/src")), GitStatus::Mixed);
        assert_eq!(
            repository.status(Path::new("/repo/src/clean")),
            GitStatus::Clean
        );
        assert_eq!(
            repository.status(Path::new("/repo/src/new/nested/file")),
            GitStatus::Untracked
        );
        assert_eq!(
            repository.status(Path::new("/repo/destination/renamed")),
            GitStatus::Staged
        );
        assert_eq!(
            repository.status(Path::new("/repo/source")),
            GitStatus::Staged
        );
        assert_eq!(repository.status(root), GitStatus::Conflict);
        assert_eq!(
            repository.status(Path::new("/repo/deleted")),
            GitStatus::Mixed
        );
        assert_eq!(
            repository.status(&root.join(std::ffi::OsStr::from_bytes(b"invalid-\xff"))),
            GitStatus::Modified
        );
    }
}
