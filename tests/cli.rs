use std::fs;
use std::os::fd::OwnedFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn create_temp_dir(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("lx-cli-{label}-{unique}"));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

fn lx_command(home_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lx"));
    command.env("HOME", home_dir);
    command
}

#[test]
fn recursive_listing_reports_unreadable_directories() {
    let temp_dir = create_temp_dir("recursive-permissions");
    let blocked_dir = temp_dir.join("blocked");
    fs::create_dir(&blocked_dir).expect("create blocked directory");
    fs::set_permissions(&blocked_dir, fs::Permissions::from_mode(0o000))
        .expect("make directory unreadable");

    let output = lx_command(&temp_dir)
        .args(["-r", temp_dir.to_str().expect("UTF-8 temp path")])
        .output()
        .expect("run lx");

    fs::set_permissions(&blocked_dir, fs::Permissions::from_mode(0o700))
        .expect("restore directory permissions");
    fs::remove_dir_all(&temp_dir).expect("remove temp dir");

    assert!(
        !output.status.success(),
        "recursive lx should return a non-zero exit status for unreadable directories"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Permission denied"),
        "stderr should report the directory error: {stderr}"
    );
}

fn git_command(directory: &Path, arguments: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args([
            "-c",
            "user.name=LX Tests",
            "-c",
            "user.email=lx@example.invalid",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(arguments)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {arguments:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn git_status_works_across_display_modes_and_worktrees() {
    let temp_dir = create_temp_dir("git-status");
    let repository = temp_dir.join("repository");
    let home_dir = temp_dir.join("home");
    fs::create_dir_all(repository.join("src")).expect("create repository");
    fs::create_dir_all(home_dir.join(".config/lx")).expect("create config directory");
    fs::write(home_dir.join(".config/lx/config"), "[icons]\ndirectory = ''\nexecutable = ''\nregular = ''\n[display]\nlong_format_fields = ['filename']\nlong_format_titles = true\n").expect("write config");
    fs::write(repository.join(".gitignore"), "*.ignored\n").expect("write ignore rules");
    for name in ["clean", "staged", "modified name", "old", "deleted"] {
        fs::write(repository.join("src").join(name), "original\n").expect("write tracked file");
    }
    git_command(&repository, &["init", "-q"]);
    git_command(&repository, &["add", "."]);
    git_command(&repository, &["commit", "-qm", "initial"]);
    fs::write(repository.join("src/staged"), "staged\n").expect("modify staged file");
    git_command(&repository, &["add", "src/staged"]);
    fs::write(repository.join("src/modified name"), "modified\n").expect("modify unstaged file");
    git_command(&repository, &["mv", "src/old", "src/renamed name"]);
    fs::remove_file(repository.join("src/deleted")).expect("delete tracked file");
    fs::create_dir(repository.join("new")).expect("create untracked directory");
    fs::write(repository.join("new/untracked"), "new\n").expect("write untracked file");
    fs::write(repository.join("new/cache.ignored"), "ignored\n").expect("write ignored file");

    for flags in [
        vec!["--git"],
        vec!["--git", "-c"],
        vec!["--git", "-1"],
        vec!["--git", "-r"],
        vec!["--git", "-l"],
        vec!["--git", "-lr"],
        vec!["--git", "-c", "-s", "name", "desc"],
    ] {
        let output = lx_command(&home_dir)
            .args(&flags)
            .arg(&repository)
            .output()
            .expect("list repository");
        assert!(
            output.status.success(),
            "{flags:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.stderr.is_empty(),
            "unexpected Git warning: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains('*') && stdout.contains("src"),
            "missing directory summary for {flags:?}: {stdout}"
        );
        assert!(
            stdout.contains('?') && stdout.contains("new"),
            "missing untracked status for {flags:?}: {stdout}"
        );
        if flags.iter().any(|flag| matches!(*flag, "-l" | "-lr")) {
            assert!(stdout.contains("Git"), "missing Git column: {stdout}");
        }
        if flags.contains(&"-r") {
            let ignored = stdout
                .lines()
                .find(|line| line.contains("cache.ignored"))
                .expect("ignored entry remains visible");
            assert!(
                ignored.contains("- cache.ignored"),
                "ignored file marked untracked: {ignored}"
            );
        }
    }

    let output = lx_command(&home_dir)
        .current_dir(&repository)
        .args(["--git", "-1", "src"])
        .output()
        .expect("list subdirectory");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in ["- clean", "S staged", "M modified name", "S renamed name"] {
        assert!(
            stdout.lines().any(|line| line == expected),
            "missing {expected:?}: {stdout}"
        );
    }
    let output = lx_command(&home_dir)
        .current_dir(&repository)
        .args(["--git", "-1", "src/modified name"])
        .output()
        .expect("list single file");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "M modified name\n");

    let output = lx_command(&home_dir)
        .env("CLICOLOR_FORCE", "1")
        .env_remove("NO_COLOR")
        .args(["--git", "-1"])
        .arg(repository.join("src/clean"))
        .output()
        .expect("list clean entry with colours");
    assert!(String::from_utf8_lossy(&output.stdout).starts_with("\x1b[90m-\x1b[0m "));

    let config_path = home_dir.join(".config/lx/config");
    let mut configuration = fs::read_to_string(&config_path).expect("read config");
    configuration.push_str("\n[colors.git]\nclean = 'bright_blue'\nmodified = 'magenta'\n");
    fs::write(config_path, configuration).expect("customise Git badge colours");
    for flags in [
        vec!["--git"],
        vec!["--git", "-c"],
        vec!["--git", "-1"],
        vec!["--git", "-r"],
        vec!["--git", "-l"],
        vec!["--git", "-lr"],
    ] {
        let output = lx_command(&home_dir)
            .env("CLICOLOR_FORCE", "1")
            .env_remove("NO_COLOR")
            .args(&flags)
            .arg(repository.join("src"))
            .output()
            .expect("list entries with custom Git colours");
        assert!(output.status.success());
        assert!(output.stderr.is_empty(), "invalid colour configuration");
        let stdout = String::from_utf8_lossy(&output.stdout);
        for badge in ["\x1b[94m-", "\x1b[35mM", "\x1b[32mS"] {
            assert!(
                stdout.contains(badge),
                "missing badge colour for {flags:?}: {stdout:?}"
            );
        }
    }

    let worktree = temp_dir.join("worktree");
    git_command(
        &repository,
        &[
            "worktree",
            "add",
            "--detach",
            worktree.to_str().expect("UTF-8 worktree path"),
        ],
    );
    fs::write(worktree.join("src/staged"), "worktree change\n").expect("modify worktree file");
    let output = lx_command(&home_dir)
        .args(["--git", "-1"])
        .arg(worktree.join("src"))
        .output()
        .expect("list worktree");
    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "M staged")
    );

    let output = lx_command(&home_dir)
        .env("PATH", temp_dir.join("missing-git"))
        .args(["--git", "-1"])
        .arg(&repository)
        .output()
        .expect("list without Git installed");
    assert!(
        output.status.success(),
        "Git failure should not prevent listing"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("lx: cannot read Git status"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains('*'));

    fs::write(home_dir.join("standalone"), "outside Git\n").expect("write standalone file");
    let plain = lx_command(&home_dir)
        .args(["-1"])
        .arg(&home_dir)
        .output()
        .expect("plain listing");
    let with_git = lx_command(&home_dir)
        .args(["--git", "-1"])
        .arg(&home_dir)
        .output()
        .expect("listing outside Git");
    assert!(with_git.status.success());
    assert!(with_git.stderr.is_empty());
    assert_eq!(plain.stdout, with_git.stdout);
    fs::remove_dir_all(temp_dir).expect("remove test directory");
}

#[test]
fn plain_and_zero_output_preserve_filename_bytes_and_listing_options() {
    let temp_dir = create_temp_dir("plain-output");
    let directory = temp_dir.join("entries");
    fs::create_dir_all(directory.join("child")).expect("create listing directory");
    for (name, content) in [
        ("alpha", "a"),
        ("trailing ", "ab"),
        ("two words", "abc"),
        (".hidden", "hidden"),
    ] {
        fs::write(directory.join(name), content).expect("write entry");
    }
    for flags in [vec![], vec!["-1"], vec!["--plain"]] {
        let output = lx_command(&temp_dir)
            .env("CLICOLOR_FORCE", "1")
            .args(flags)
            .arg(&directory)
            .output()
            .expect("capture plain listing");
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert_eq!(output.stdout, b"child\nalpha\ntrailing \ntwo words\n");
    }

    fs::write(directory.join("line\nbreak"), "abcd").expect("write newline name");
    let invalid_name = std::ffi::OsStr::from_bytes(b"invalid-\xff");
    let supports_non_utf8 = match fs::write(directory.join(invalid_name), "abcde") {
        Ok(()) => true,
        // macOS filesystems can require valid UTF-8 filenames.
        Err(error)
            if cfg!(target_os = "macos")
                && matches!(error.raw_os_error(), Some(libc::EPERM | libc::EINVAL)) =>
        {
            false
        }
        Err(error) => panic!("write non-UTF-8 name: {error}"),
    };
    let output = lx_command(&temp_dir)
        .env("CLICOLOR_FORCE", "1")
        .args(["--zero", "-a"])
        .arg(&directory)
        .output()
        .expect("capture NUL-separated listing");
    assert!(output.status.success());
    let mut expected = b"child\0.hidden\0alpha\0".to_vec();
    if supports_non_utf8 {
        expected.extend_from_slice(b"invalid-\xff\0");
    }
    expected.extend_from_slice(b"line\nbreak\0trailing \0two words\0");
    assert_eq!(output.stdout, expected);

    let output = lx_command(&temp_dir)
        .args(["-0fs", "size", "desc", "-x", "two*"])
        .arg(&directory)
        .output()
        .expect("capture sorted and filtered listing");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut expected = Vec::new();
    if supports_non_utf8 {
        expected.extend_from_slice(b"invalid-\xff\0");
    }
    expected.extend_from_slice(b"line\nbreak\0trailing \0alpha\0");
    assert_eq!(output.stdout, expected);

    let target = directory.join(invalid_name);
    let output = lx_command(&temp_dir)
        .arg("--zero")
        .arg(&target)
        .output()
        .expect("list non-UTF-8 target");
    if supports_non_utf8 {
        assert!(output.status.success());
        let mut expected = target.as_os_str().as_bytes().to_vec();
        expected.push(0);
        assert_eq!(output.stdout, expected);
    } else {
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("lx: cannot access"));
    }

    let output = lx_command(&temp_dir)
        .arg("--zero")
        .arg(directory.join("child"))
        .output()
        .expect("list empty directory");
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    fs::remove_dir_all(temp_dir).expect("remove test directory");
}

#[test]
fn closed_output_exits_quietly_across_display_modes() {
    let temp_dir = create_temp_dir("closed-output");
    fs::write(temp_dir.join("entry"), "example").expect("write entry");
    for flags in [
        vec![],
        vec!["--plain"],
        vec!["--zero"],
        vec!["-c"],
        vec!["-l"],
        vec!["-r"],
        vec!["-lr"],
        vec!["--git", "-1"],
    ] {
        let (writer, reader) = UnixStream::pair().expect("create output stream");
        drop(reader);
        let writer: OwnedFd = writer.into();
        let output = lx_command(&temp_dir)
            .args(&flags)
            .arg(&temp_dir)
            .stdout(Stdio::from(writer))
            .output()
            .expect("write to a closed consumer");
        assert!(
            output.status.success(),
            "failed for {flags:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.stderr.is_empty(),
            "unexpected diagnostic for {flags:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fs::remove_dir_all(temp_dir).expect("remove test directory");
}
