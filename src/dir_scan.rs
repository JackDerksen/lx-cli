//! Reads a directory's entries together with the metadata lx needs.
//!
//! On macOS, `getattrlistbulk` returns the attributes of many entries per system call, which
//! is much faster than one `lstat` per entry. Elsewhere, and on file systems without bulk
//! attribute support, entries come from `std::fs::read_dir`.

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// The `lstat` fields lx uses, however they were obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Stat {
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub dev: u64,
    pub ino: u64,
    pub size: u64,
    pub nlink: u64,
    pub modified: SystemTime,
}

impl Stat {
    pub fn is_dir(&self) -> bool {
        self.file_type() == libc::S_IFDIR as u32
    }

    pub fn is_file(&self) -> bool {
        self.file_type() == libc::S_IFREG as u32
    }

    pub fn is_symlink(&self) -> bool {
        self.file_type() == libc::S_IFLNK as u32
    }

    fn file_type(&self) -> u32 {
        self.mode & libc::S_IFMT as u32
    }
}

impl From<&fs::Metadata> for Stat {
    fn from(metadata: &fs::Metadata) -> Self {
        Self {
            mode: metadata.mode(),
            uid: metadata.uid(),
            gid: metadata.gid(),
            dev: metadata.dev(),
            ino: metadata.ino(),
            size: metadata.len(),
            nlink: metadata.nlink(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        }
    }
}

/// One directory entry; `stat` fails when only this entry could not be examined.
pub(crate) struct ScannedEntry {
    pub name: OsString,
    pub stat: io::Result<Stat>,
}

/// Visits the entries of `directory`, excluding `.` and `..`, in file-system order.
///
/// Entries rejected by `wants` may be skipped without being examined. `visit` receives an
/// error when the listing itself fails part-way; opening the directory fails up front.
pub(crate) fn scan(
    directory: &Path,
    wants: impl Fn(&OsStr) -> bool,
    mut visit: impl FnMut(io::Result<ScannedEntry>),
) -> io::Result<()> {
    #[cfg(target_os = "macos")]
    if let Some(result) = bulk::scan(directory, &wants, &mut visit) {
        return result;
    }

    for entry in fs::read_dir(directory)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                visit(Err(error));
                continue;
            }
        };
        let name = entry.file_name();
        if !wants(&name) {
            continue;
        }
        let stat = entry.metadata().map(|metadata| Stat::from(&metadata));
        visit(Ok(ScannedEntry { name, stat }));
    }
    Ok(())
}

fn system_time(seconds: i64, nanoseconds: i64) -> SystemTime {
    // Mirrors `Metadata::modified`, including times before the epoch.
    let nanoseconds = Duration::from_nanos(nanoseconds.clamp(0, 999_999_999) as u64);
    if seconds >= 0 {
        SystemTime::UNIX_EPOCH + Duration::from_secs(seconds as u64) + nanoseconds
    } else {
        SystemTime::UNIX_EPOCH - Duration::from_secs(seconds.unsigned_abs()) + nanoseconds
    }
}

#[cfg(target_os = "macos")]
mod bulk {
    use super::{ScannedEntry, Stat, system_time};
    use std::ffi::{CString, OsStr};
    use std::fs::File;
    use std::io;
    use std::mem::MaybeUninit;
    use std::os::fd::{AsRawFd, RawFd};
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::OpenOptionsExt;
    use std::path::Path;

    const BUFFER_SIZE: usize = 128 * 1024;

    // From <sys/attr.h> and the `vtype` enum in <sys/vnode.h>, which `libc` does not define.
    const ATTR_CMN_ERROR: u32 = 0x2000_0000;
    const VREG: u32 = 1;
    const VBLK: u32 = 3;
    const VCHR: u32 = 4;
    const VLNK: u32 = 5;
    const VSOCK: u32 = 6;
    const VFIFO: u32 = 7;

    const COMMON_ATTRIBUTES: u32 = libc::ATTR_CMN_RETURNED_ATTRS
        | libc::ATTR_CMN_NAME
        | libc::ATTR_CMN_DEVID
        | libc::ATTR_CMN_OBJTYPE
        | libc::ATTR_CMN_MODTIME
        | libc::ATTR_CMN_OWNERID
        | libc::ATTR_CMN_GRPID
        | libc::ATTR_CMN_ACCESSMASK
        | libc::ATTR_CMN_FILEID
        | ATTR_CMN_ERROR;
    const FILE_ATTRIBUTES: u32 = libc::ATTR_FILE_LINKCOUNT | libc::ATTR_FILE_DATALENGTH;
    /// Attributes that must all be valid to build a `Stat` without falling back to `lstat`.
    const REQUIRED_COMMON: u32 =
        COMMON_ATTRIBUTES & !(libc::ATTR_CMN_RETURNED_ATTRS | libc::ATTR_CMN_NAME | ATTR_CMN_ERROR);

    /// Returns `None` when the file system cannot report attributes in bulk.
    pub(super) fn scan(
        directory: &Path,
        wants: &dyn Fn(&OsStr) -> bool,
        visit: &mut dyn FnMut(io::Result<ScannedEntry>),
    ) -> Option<io::Result<()>> {
        let file = match File::options()
            .read(true)
            .custom_flags(libc::O_DIRECTORY)
            .open(directory)
        {
            Ok(file) => file,
            Err(error) => return Some(Err(error)),
        };
        let fd = file.as_raw_fd();
        // SAFETY: `attrlist` is plain data, and zero is valid for every field.
        let mut request: libc::attrlist = unsafe { std::mem::zeroed() };
        request.bitmapcount = libc::ATTR_BIT_MAP_COUNT;
        request.commonattr = COMMON_ATTRIBUTES;
        request.fileattr = FILE_ATTRIBUTES;
        let mut buffer = vec![0u8; BUFFER_SIZE];
        let mut first_call = true;

        loop {
            // SAFETY: the request and buffer outlive the call, and the length matches the buffer.
            let count = unsafe {
                libc::getattrlistbulk(
                    fd,
                    (&raw mut request).cast(),
                    buffer.as_mut_ptr().cast(),
                    buffer.len(),
                    u64::from(libc::FSOPT_PACK_INVAL_ATTRS),
                )
            };
            if count < 0 {
                let error = io::Error::last_os_error();
                if first_call && matches!(error.raw_os_error(), Some(libc::ENOTSUP | libc::EINVAL))
                {
                    return None;
                }
                visit(Err(error));
                return Some(Ok(()));
            }
            if count == 0 {
                return Some(Ok(()));
            }
            first_call = false;

            let mut offset = 0;
            for _ in 0..count {
                let Some(record) = buffer
                    .get(offset..)
                    .and_then(|rest| Some(&rest[..read_u32(rest, 0)? as usize]))
                else {
                    visit(Err(invalid_record()));
                    return Some(Ok(()));
                };
                offset += record.len();
                match parse_record(record) {
                    Some((name, _)) if !wants(name) => {}
                    Some((name, Some(stat))) => visit(Ok(ScannedEntry {
                        name: name.to_os_string(),
                        stat: Ok(stat),
                    })),
                    Some((name, None)) => visit(Ok(ScannedEntry {
                        name: name.to_os_string(),
                        stat: stat_at(fd, name),
                    })),
                    None => {
                        visit(Err(invalid_record()));
                        return Some(Ok(()));
                    }
                }
            }
        }
    }

    fn invalid_record() -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid directory attribute record",
        )
    }

    /// Parses one packed record. The `Stat` is `None` when it must come from `lstat`
    /// instead: directories (whose size has no bulk attribute), errors, and missing attributes.
    fn parse_record(record: &[u8]) -> Option<(&OsStr, Option<Stat>)> {
        let mut cursor = Cursor { record, offset: 4 };
        let returned_common = cursor.u32()?;
        cursor.skip(4)?; // Volume attributes.
        cursor.skip(4)?; // Directory attributes.
        let returned_file = cursor.u32()?;
        cursor.skip(4)?; // Fork attributes.
        // The error code directly follows the returned attributes, ahead of bitmap order.
        let error = cursor.u32()?;

        let name_start = cursor.offset;
        let name_offset = cursor.i32()?;
        let name_length = cursor.u32()? as usize;
        let name = name_start
            .checked_add_signed(name_offset as isize)
            .and_then(|start| record.get(start..start.checked_add(name_length)?))?;
        // The length includes the terminating NUL.
        let name = OsStr::from_bytes(name.strip_suffix(b"\0").unwrap_or(name));

        let dev = cursor.i32()?;
        let object_type = cursor.u32()?;
        let file_type = match object_type {
            VREG => libc::S_IFREG,
            VLNK => libc::S_IFLNK,
            VFIFO => libc::S_IFIFO,
            VSOCK => libc::S_IFSOCK,
            VBLK => libc::S_IFBLK,
            VCHR => libc::S_IFCHR,
            // Directories carry no file attributes, and their size needs `lstat`.
            _ => return Some((name, None)),
        };
        let modified_seconds = cursor.i64()?;
        let modified_nanoseconds = cursor.i64()?;
        let uid = cursor.u32()?;
        let gid = cursor.u32()?;
        let access = cursor.u32()?;
        let ino = cursor.u64()?;
        let nlink = cursor.u32()?;
        let size = cursor.i64()?;

        let complete = error == 0
            && returned_common & REQUIRED_COMMON == REQUIRED_COMMON
            && returned_file & FILE_ATTRIBUTES == FILE_ATTRIBUTES;
        if !complete {
            return Some((name, None));
        }
        Some((
            name,
            Some(Stat {
                // The access mask also carries the type bits; keep only permissions.
                mode: u32::from(file_type) | (access & 0o7777),
                uid,
                gid,
                // `dev_t` is signed here; this matches `MetadataExt::dev`.
                dev: dev as u64,
                ino,
                size: size as u64,
                nlink: u64::from(nlink),
                modified: system_time(modified_seconds, modified_nanoseconds),
            }),
        ))
    }

    fn stat_at(directory: RawFd, name: &OsStr) -> io::Result<Stat> {
        let name = CString::new(name.as_bytes())?;
        let mut stat = MaybeUninit::<libc::stat>::uninit();
        // SAFETY: `name` is NUL-terminated and `stat` is writable for one `libc::stat`.
        let result = unsafe {
            libc::fstatat(
                directory,
                name.as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: `fstatat` succeeded, so it initialised `stat`.
        let stat = unsafe { stat.assume_init() };
        Ok(Stat {
            mode: u32::from(stat.st_mode),
            uid: stat.st_uid,
            gid: stat.st_gid,
            dev: stat.st_dev as u64,
            ino: stat.st_ino,
            size: stat.st_size as u64,
            nlink: u64::from(stat.st_nlink),
            modified: system_time(stat.st_mtime, stat.st_mtime_nsec),
        })
    }

    fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
        Some(u32::from_ne_bytes(
            bytes.get(offset..offset + 4)?.try_into().ok()?,
        ))
    }

    /// Reads packed attributes in order; values are not necessarily aligned.
    struct Cursor<'a> {
        record: &'a [u8],
        offset: usize,
    }

    impl Cursor<'_> {
        fn take<const N: usize>(&mut self) -> Option<[u8; N]> {
            let bytes = self.record.get(self.offset..self.offset + N)?;
            self.offset += N;
            bytes.try_into().ok()
        }

        fn skip(&mut self, length: usize) -> Option<()> {
            self.offset = self.offset.checked_add(length)?;
            Some(())
        }

        fn u32(&mut self) -> Option<u32> {
            self.take().map(u32::from_ne_bytes)
        }

        fn i32(&mut self) -> Option<i32> {
            self.take().map(i32::from_ne_bytes)
        }

        fn u64(&mut self) -> Option<u64> {
            self.take().map(u64::from_ne_bytes)
        }

        fn i64(&mut self) -> Option<i64> {
            self.take().map(i64::from_ne_bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Stat, scan};
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::fs;
    use std::path::Path;

    /// Every scanned entry must match `lstat`, whichever way it was read.
    fn assert_matches_lstat(directory: &Path) {
        let mut scanned = BTreeMap::new();
        scan(
            directory,
            |_| true,
            |entry| {
                let entry = entry.expect("read directory");
                scanned.insert(entry.name, entry.stat.expect("stat entry"));
            },
        )
        .expect("scan directory");
        let expected: BTreeMap<OsString, Stat> = fs::read_dir(directory)
            .expect("read directory")
            .map(|entry| {
                let entry = entry.expect("directory entry");
                let metadata = fs::symlink_metadata(entry.path()).expect("lstat entry");
                (entry.file_name(), Stat::from(&metadata))
            })
            .collect();
        assert_eq!(scanned, expected, "{}", directory.display());
    }

    #[test]
    fn scanned_metadata_matches_lstat() {
        let directory = std::env::temp_dir().join(format!("lx-dir-scan-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(directory.join("folder/nested")).expect("create directories");
        fs::write(directory.join("file"), "contents").expect("write file");
        fs::write(directory.join(".hidden"), "").expect("write hidden file");
        fs::hard_link(directory.join("file"), directory.join("hard-link")).expect("hard link");
        std::os::unix::fs::symlink("file", directory.join("link")).expect("symlink");
        std::os::unix::fs::symlink("missing", directory.join("broken")).expect("broken symlink");
        let special = directory.join("special");
        fs::write(&special, "#!/bin/sh\n").expect("write special file");
        fs::set_permissions(
            &special,
            std::os::unix::fs::PermissionsExt::from_mode(0o7755),
        )
        .expect("set special bits");
        let fifo = std::ffi::CString::new(directory.join("fifo").as_os_str().as_encoded_bytes())
            .expect("fifo path");
        // SAFETY: `fifo` is a valid NUL-terminated path.
        assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o644) }, 0);
        let _socket =
            std::os::unix::net::UnixListener::bind(directory.join("socket")).expect("bind socket");

        let result = std::panic::catch_unwind(|| {
            #[cfg(target_os = "macos")]
            assert!(
                super::bulk::scan(&directory, &|_| true, &mut |_| {}).is_some(),
                "APFS supports bulk attributes"
            );
            assert_matches_lstat(&directory);
            // Real system directories add devices, setuid binaries, and many hard links.
            for system in ["/dev", "/usr/bin", "/usr/lib"] {
                if Path::new(system).is_dir() {
                    assert_matches_lstat(Path::new(system));
                }
            }
        });
        fs::remove_dir_all(&directory).expect("remove temp dir");
        if let Err(panic) = result {
            std::panic::resume_unwind(panic);
        }
    }

    /// Run with `LX_VERIFY_SCAN=/some/tree cargo test -- --ignored` to check a whole tree.
    #[test]
    #[ignore = "walks a user-supplied tree"]
    fn scanned_metadata_matches_lstat_across_tree() {
        let root = std::env::var_os("LX_VERIFY_SCAN").expect("set LX_VERIFY_SCAN");
        let mut pending = vec![std::path::PathBuf::from(root)];
        let mut directories = 0;
        while let Some(directory) = pending.pop() {
            let Ok(entries) = fs::read_dir(&directory) else {
                continue;
            };
            assert_matches_lstat(&directory);
            directories += 1;
            for entry in entries.flatten() {
                if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                    pending.push(entry.path());
                }
            }
        }
        eprintln!("verified {directories} directories");
    }
}
