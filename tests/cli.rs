use std::fs;
use std::os::fd::OwnedFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

/// A test directory removed on drop, including when the test panics.
struct TempDir(PathBuf);

impl std::ops::Deref for TempDir {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for TempDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<std::ffi::OsStr> for TempDir {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.0.as_os_str()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        if fs::remove_dir_all(&self.0).is_ok() {
            return;
        }
        // Tests may leave directories unreadable; restore access and retry.
        fn make_accessible(path: &Path) {
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o700));
            for entry in fs::read_dir(path).into_iter().flatten().flatten() {
                if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                    make_accessible(&entry.path());
                }
            }
        }
        make_accessible(&self.0);
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn create_temp_dir(label: &str) -> TempDir {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("lx-cli-{label}-{unique}"));
    fs::create_dir_all(&path).expect("create temp dir");
    TempDir(path)
}

#[test]
fn temp_dirs_are_removed_when_a_test_panics() {
    let path = std::panic::catch_unwind(|| {
        let temp_dir = create_temp_dir("drop-guard");
        let blocked = temp_dir.join("blocked");
        fs::create_dir(&blocked).expect("create blocked directory");
        fs::write(blocked.join("file"), "content").expect("write file");
        fs::set_permissions(&blocked, fs::Permissions::from_mode(0o000))
            .expect("make directory unreadable");
        std::panic::panic_any(temp_dir.to_path_buf());
    })
    .expect_err("closure panics")
    .downcast::<PathBuf>()
    .expect("panic carries the temp path");
    assert!(!path.exists(), "{} should be removed", path.display());
}

fn lx_command(home_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lx"));
    command
        .env("HOME", home_dir)
        .env("COLORTERM", "truecolor")
        .env("NO_COLOR", "1")
        .env_remove("CLICOLOR_FORCE");
    command
}

#[test]
fn automatic_icons_recognise_system_folders_and_respect_global_overrides() {
    let temp_dir = create_temp_dir("system-icons");
    let home_dir = temp_dir.join("home");
    let config_path = home_dir.join(".config/lx/config");
    fs::create_dir_all(config_path.parent().expect("config parent"))
        .expect("create config directory");
    for folder in [
        "Documents",
        "Pictures",
        "projects/Documents",
        "projects/.config",
        ".Trash",
        ".cache",
        ".local/share/Trash",
        ".ssh",
    ] {
        fs::create_dir_all(home_dir.join(folder)).expect("create directory");
    }
    let downloads = if cfg!(target_os = "linux") {
        "Incoming"
    } else {
        "Downloads"
    };
    fs::create_dir(home_dir.join(downloads)).expect("create downloads directory");
    fs::write(home_dir.join("Documents/note"), "content").expect("write file");
    std::os::unix::fs::symlink("Documents", home_dir.join("documents-link"))
        .expect("create directory link");
    #[cfg(target_os = "linux")]
    fs::write(
        home_dir.join(".config/user-dirs.dirs"),
        "XDG_DOCUMENTS_DIR=\"$HOME/Documents\"\nXDG_DOWNLOAD_DIR=\"$HOME/Incoming\"\nXDG_PICTURES_DIR=\"$HOME/Pictures\"\n",
    )
    .expect("configure XDG user directories");
    #[cfg(target_os = "macos")]
    for folder in ["Applications", "Library"] {
        fs::create_dir(home_dir.join(folder)).expect("create macOS directory");
    }

    fs::write(&config_path, "[icons]\nenabled = true\n").expect("enable automatic icons");
    for flags in [
        vec!["--git"],
        vec!["--git", "-1"],
        vec!["-c"],
        vec!["-r"],
        vec!["-l"],
        vec!["-lr"],
    ] {
        let output = lx_command(&home_dir)
            .env("XDG_CONFIG_HOME", home_dir.join(".config"))
            .current_dir(&home_dir)
            .args(&flags)
            .arg("-a")
            .arg(".")
            .output()
            .expect("list folders with automatic icons");
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let stdout = String::from_utf8_lossy(&output.stdout);
        for expected in [
            "\u{f107f} .config",
            "\u{f0aba} .cache",
            "\u{f0253} .local",
            "\u{f1f8} .Trash",
            "\u{f0250} .ssh",
        ] {
            assert!(
                stdout.contains(expected),
                "missing hidden-folder icon for {flags:?}: {stdout}"
            );
        }
        if flags.iter().any(|flag| matches!(*flag, "-r" | "-lr")) {
            assert!(
                stdout.contains("\u{f024b} .config/"),
                "unrelated hidden folder received system icon: {stdout}"
            );
            #[cfg(target_os = "linux")]
            assert!(
                stdout.contains("\u{f1f8} Trash/"),
                "missing Linux trash icon: {stdout}"
            );
        }
        for icon in ["\u{f0c82}", "\u{f024d}", "\u{f024f}", "\u{f024b}", ""] {
            assert!(
                stdout.contains(icon),
                "missing {icon:?} for {flags:?}: {stdout}"
            );
        }
        #[cfg(target_os = "macos")]
        for icon in ["\u{f1b3}", "\u{f02d}"] {
            assert!(
                stdout.contains(icon),
                "missing macOS icon for {flags:?}: {stdout}"
            );
        }
        if flags.contains(&"-r") {
            assert!(
                stdout.contains("\u{f024b} Documents/"),
                "nested folder received system icon: {stdout}"
            );
        }
        if flags.iter().any(|flag| matches!(*flag, "-l" | "-lr")) {
            assert_eq!(
                stdout
                    .lines()
                    .next()
                    .expect("header")
                    .split_whitespace()
                    .collect::<Vec<_>>(),
                ["Permissions", "Size", "Modified", "Name"]
            );
            assert!(stdout.contains("\u{f0c82} Documents"));
        }
    }

    for (configuration, expected) in [
        ("[icons]\nenabled = true\ndirectory = 'D'\n", "D Documents"),
        ("[icons]\nenabled = false\ndirectory = 'D'\n", "D Documents"),
        ("[icons]\nenabled = true\ndirectory = ''\n", "Documents"),
        ("[icons]\nenabled = false\n", "Documents"),
    ] {
        fs::write(&config_path, configuration).expect("set directory override");
        let output = lx_command(&home_dir)
            .env("XDG_CONFIG_HOME", home_dir.join(".config"))
            .args(["--git", "-1"])
            .arg("-a")
            .arg(&home_dir)
            .output()
            .expect("list folders with overrides");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.lines().any(|line| line == expected),
            "override ignored: {stdout}"
        );
        let hidden_expected = expected.replace("Documents", ".config");
        assert!(
            stdout.lines().any(|line| line == hidden_expected),
            "hidden folder ignored global override: {stdout}"
        );
        assert!(
            !stdout.contains("\u{f0c82}"),
            "system icon survived global override"
        );
        assert!(
            !stdout.contains("\u{f024b}"),
            "generic icon survived global override"
        );
    }

    fs::write(
        &config_path,
        "[icons]\nenabled = true\ndirectory = '📁'\nregular = ''\nsymlink = ''\n[display]\nlong_format_fields = ['icon', 'filename', 'size']\n",
    )
    .expect("configure inline icons and reordered fields");
    for flag in ["-l", "-lr"] {
        let output = lx_command(&home_dir)
            .arg(flag)
            .arg(&home_dir)
            .output()
            .expect("list reordered columns with inline icons");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut lines = stdout.lines();
        let header = lines.next().expect("header");
        assert_eq!(
            header.split_whitespace().collect::<Vec<_>>(),
            ["Name", "Size"]
        );
        let width = unicode_width::UnicodeWidthStr::width(header);
        for line in lines {
            assert_eq!(
                unicode_width::UnicodeWidthStr::width(line),
                width,
                "misaligned {flag}: {line}"
            );
        }
        assert!(stdout.contains("📁 Documents"));
    }

    fs::write(&config_path, "[icons]\nenabled = true\n").expect("restore automatic icons");
    for flags in [vec![], vec!["--plain"], vec!["--zero"]] {
        let output = lx_command(&home_dir)
            .env("XDG_CONFIG_HOME", home_dir.join(".config"))
            .env("CLICOLOR_FORCE", "1")
            .env_remove("NO_COLOR")
            .args(&flags)
            .arg(&home_dir)
            .output()
            .expect("capture undecorated filenames");
        assert!(output.status.success());
        assert!(
            output.stdout.is_ascii(),
            "icons or styling leaked into plain output"
        );
        assert!(!output.stdout.contains(&0x1b));
    }
    assert!(toml::from_str::<lx_cli::config::Config>(include_str!("../config.example")).is_ok());
}

#[test]
fn path_icons_override_defaults_but_global_icons_still_win() {
    let temp_dir = create_temp_dir("path-icons");
    let home_dir = temp_dir.join("home");
    let config_path = home_dir.join(".config/lx/config");
    fs::create_dir_all(config_path.parent().expect("config parent"))
        .expect("create config directory");
    for folder in [
        "Documents/child",
        "Pictures",
        "projects/Documents",
        ".Trash",
        ".local",
    ] {
        fs::create_dir_all(home_dir.join(folder)).expect("create directory");
    }
    fs::write(home_dir.join("note"), "content").expect("write file");
    std::os::unix::fs::symlink("Documents", home_dir.join("documents-link"))
        .expect("create directory link");
    let paths = format!(
        "[icons.paths]\n'~/Documents' = 'A'\n'projects' = 'B'\n'{}' = 'C'\n'~/Pictures' = ''\n'~/note' = 'N'\n'~/documents-link' = 'L'\n'~/missing' = 'unused'\n'~/.config' = 'H'\n'~/.Trash' = 'T'\n'~/.local' = ''\n",
        home_dir.join("projects/Documents").display(),
    );
    fs::write(&config_path, format!("[icons]\nenabled = true\n{paths}"))
        .expect("configure path icons");
    for flags in [
        vec!["--git", "-1"],
        vec!["-c"],
        vec!["-r"],
        vec!["-l"],
        vec!["-lr"],
    ] {
        let output = lx_command(&home_dir)
            .current_dir(&temp_dir)
            .args(&flags)
            .arg("-a")
            .arg("home")
            .output()
            .expect("list entries with path icons");
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let stdout = String::from_utf8_lossy(&output.stdout);
        for expected in [
            "A Documents",
            "B projects",
            "N note",
            "L documents-link -> Documents",
            "H .config",
            "T .Trash",
        ] {
            assert!(
                stdout.contains(expected),
                "missing {expected:?} for {flags:?}: {stdout}"
            );
        }
        assert!(
            !stdout.contains("unused"),
            "missing-path override affected output"
        );
        assert!(
            !stdout.contains("\u{f024f} Pictures"),
            "empty path icon did not hide default icon"
        );
        if flags == ["--git", "-1"] {
            assert!(stdout.lines().any(|line| line == "Pictures"));
            assert!(stdout.lines().any(|line| line == ".local"));
        }
        if flags.iter().any(|flag| matches!(*flag, "-r" | "-lr")) {
            assert!(
                stdout.contains("C Documents/"),
                "absolute path icon ignored: {stdout}"
            );
            assert!(
                stdout.contains("\u{f024b} child/"),
                "parent icon leaked to child: {stdout}"
            );
        }
    }

    let output = lx_command(&home_dir)
        .env("CLICOLOR_FORCE", "1")
        .env_remove("NO_COLOR")
        .args(["-al"])
        .arg(&home_dir)
        .output()
        .expect("list custom hidden icons with colours");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\x1b[38;2;104;101;111mH\x1b[0m"),
        "custom icon lost hidden colour: {stdout:?}"
    );

    for (settings, expected, hidden_expected) in [
        ("enabled = false", "A Documents", "H .config"),
        (
            "enabled = true\ndirectory = 'G'",
            "G Documents",
            "G .config",
        ),
        ("enabled = true\ndirectory = ''", "Documents", ".config"),
    ] {
        fs::write(&config_path, format!("[icons]\n{settings}\n{paths}"))
            .expect("configure global precedence");
        let output = lx_command(&home_dir)
            .args(["--git", "-1", "-a"])
            .arg(&home_dir)
            .output()
            .expect("list entries with global and path icons");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.lines().any(|line| line == expected),
            "incorrect precedence: {stdout}"
        );
        assert!(stdout.lines().any(|line| line == "N note"));
        assert!(
            stdout.lines().any(|line| line == hidden_expected),
            "incorrect hidden icon precedence: {stdout}"
        );
        assert!(
            stdout
                .lines()
                .any(|line| line == "L documents-link -> Documents")
        );
    }
}

#[test]
fn development_icons_and_patterns_match_real_names_and_paths_across_formats() {
    let temp_dir = create_temp_dir("icon-patterns");
    let home_dir = temp_dir.join("home");
    let project = home_dir.join("projectName");
    let config_path = home_dir.join(".config/lx/config");
    fs::create_dir_all(config_path.parent().expect("config parent"))
        .expect("create config directory");
    for folder in [
        ".git",
        "src",
        "target",
        "tests",
        "docs",
        "assets",
        "vendor",
        "scripts",
        "notes",
        "other/notes",
        "looks.rs",
        "other/LICENSE",
        "other/README",
        "photo.PNG",
    ] {
        fs::create_dir_all(project.join(folder)).expect("create project directory");
    }
    for name in [
        "main.rs",
        "exact.rs",
        "README.md",
        "draft.md",
        "qé.rs",
        "run.rs",
        "other/src",
        "LICENSE",
        "LICENCE.md",
        "LICENSE-MIT",
        "copying",
        "readme",
        "Cargo.toml",
        "Cargo.lock",
        "package.json",
        "settings.toml",
        ".env.local",
        ".gitignore",
        "report.PDF",
        "image.PNG",
        "archive.tar.gz",
        "app.tsx",
        "Dockerfile.dev",
        "unknown.blob",
    ] {
        fs::write(project.join(name), "content").expect("write project file");
    }
    fs::set_permissions(project.join("run.rs"), fs::Permissions::from_mode(0o755))
        .expect("make executable");
    std::os::unix::fs::symlink("main.rs", project.join("link.rs")).expect("create file link");
    fs::write(&config_path, "[icons]\nenabled = true\n").expect("enable default icons");
    let output = lx_command(&home_dir)
        .args(["-alr"])
        .arg(&project)
        .output()
        .expect("list default development icons");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "\u{e702} .git/",
        "\u{f121} src/",
        "\u{f19fc} target/",
        "\u{f197e} tests/",
        "\u{f0c82} docs/",
        "\u{f024f} assets/",
        "\u{f0253} vendor/",
        "\u{f120} scripts/",
        " src",
        "\u{e60a} LICENSE",
        "\u{e60a} LICENCE.md",
        "\u{e60a} LICENSE-MIT",
        "\u{e60a} copying",
        "\u{f024b} LICENSE/",
        "\u{f02d} README.md",
        "\u{f02d} readme",
        "\u{f024b} README/",
        "\u{e68b} main.rs",
        "\u{e68b} run.rs",
        " link.rs -> main.rs",
        "\u{e68b} Cargo.toml",
        "\u{e672} Cargo.lock",
        "\u{e616} package.json",
        "\u{e615} settings.toml",
        "\u{e615} .env.local",
        "\u{e702} .gitignore",
        "\u{e67d} report.PDF",
        "\u{e60d} image.PNG",
        "\u{f1c6} archive.tar.gz",
        "\u{e625} app.tsx",
        "\u{e650} Dockerfile.dev",
        " unknown.blob",
        "\u{f024b} photo.PNG/",
    ] {
        assert!(
            stdout.contains(expected),
            "missing default icon {expected:?}: {stdout}"
        );
    }

    let rules = r#"
[[icons.patterns]]
pattern = "exact.rs"
icon = "P"
[[icons.patterns]]
pattern = "q?.rs"
kind = "file"
icon = "Q"
[[icons.patterns]]
pattern = "*.rs"
kind = "file"
icon = "R"
[[icons.patterns]]
pattern = "*.rs"
icon = "later-rule"
[[icons.patterns]]
pattern = "draft.md"
icon = ""
[[icons.patterns]]
pattern = "*.md"
kind = "file"
icon = "M"
[[icons.patterns]]
pattern = "*/projectName/notes/"
icon = "N"
[[icons.patterns]]
pattern = "src/"
icon = "S"
[[icons.patterns]]
pattern = "tests"
kind = "directory"
icon = "T"
"#;
    let configuration = format!(
        "[icons]\nenabled = false\n[icons.paths]\n'{}' = 'E'\n[display]\nlong_format_fields = ['filename', 'size']\n{rules}",
        project.join("exact.rs").display()
    );
    fs::write(&config_path, &configuration).expect("configure ordered patterns");
    for flag in ["-c", "-l", "-r", "-lr"] {
        let output = lx_command(&home_dir)
            .current_dir(&home_dir)
            .args([flag, "-a", "projectName"])
            .output()
            .expect("list entries with patterns");
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let stdout = String::from_utf8_lossy(&output.stdout);
        for expected in [
            "R main.rs",
            "R run.rs",
            "R link.rs -> main.rs",
            "E exact.rs",
            "Q qé.rs",
            "M README.md",
            "M LICENCE.md",
            "N notes",
            "S src",
            "T tests",
        ] {
            assert!(
                stdout.contains(expected),
                "missing {expected:?} for {flag}: {stdout}"
            );
        }
        assert!(
            !stdout.contains("P exact.rs"),
            "pattern beat exact path override"
        );
        assert!(
            !stdout.contains("\u{e60a}"),
            "automatic licence icon appeared while disabled"
        );
        assert!(
            !stdout.contains("\u{f02d}"),
            "automatic README icon appeared while disabled"
        );
        assert!(
            !stdout.contains("M draft.md"),
            "empty icon rule did not win"
        );
        assert!(!stdout.contains("later-rule main.rs"), "rule order ignored");
        if matches!(flag, "-l" | "-lr") {
            let mut lines = stdout.lines();
            let header = lines.next().expect("header");
            assert_eq!(
                header.split_whitespace().collect::<Vec<_>>(),
                ["Name", "Size"]
            );
            let width = unicode_width::UnicodeWidthStr::width(header);
            for line in lines {
                assert_eq!(
                    unicode_width::UnicodeWidthStr::width(line),
                    width,
                    "pattern changed alignment for {flag}: {line}"
                );
            }
        }
        if matches!(flag, "-r" | "-lr") {
            let source_file = stdout
                .lines()
                .find(|line| line.contains("src") && !line.contains("src/"))
                .expect("file named src");
            assert!(
                !source_file.contains("S src"),
                "directory-only pattern matched file: {source_file}"
            );
            let other_notes = stdout
                .lines()
                .filter(|line| line.contains("notes/"))
                .collect::<Vec<_>>();
            assert_eq!(other_notes.len(), 2);
            assert_eq!(
                other_notes
                    .iter()
                    .filter(|line| line.contains("N notes/"))
                    .count(),
                1,
                "path rule matched wrong notes directory"
            );
        }
    }

    fs::write(
        &config_path,
        configuration.replacen(
            "enabled = false",
            "enabled = false\ndirectory = 'G'\nregular = 'F'\nsymlink = 'L'",
            1,
        ),
    )
    .expect("set global icons over patterns");
    let output = lx_command(&home_dir)
        .arg("-l")
        .arg(&project)
        .output()
        .expect("list with global overrides");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "G src",
        "G notes",
        "F main.rs",
        "F exact.rs",
        "L link.rs -> main.rs",
        "R run.rs",
    ] {
        assert!(
            stdout.contains(expected),
            "global precedence failed: {stdout}"
        );
    }
    let output = lx_command(&home_dir)
        .arg("--plain")
        .arg(&project)
        .output()
        .expect("capture plain filenames");
    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "main.rs")
    );
}

#[test]
fn recursive_listing_reports_unreadable_directories() {
    let temp_dir = create_temp_dir("recursive-permissions");
    let listing = temp_dir.join("listing");
    let blocked_dir = listing.join("blocked");
    fs::create_dir_all(&blocked_dir).expect("create blocked directory");
    fs::set_permissions(&blocked_dir, fs::Permissions::from_mode(0o000))
        .expect("make directory unreadable");

    let output = lx_command(&temp_dir)
        .arg("-r")
        .arg(&listing)
        .output()
        .expect("run lx");

    for arguments in [vec!["-l"], vec!["-lr", "--depth", "0"]] {
        let metadata = lx_command(&temp_dir)
            .args(arguments)
            .arg(&listing)
            .output()
            .expect("list metadata without scanning unreadable descendants");
        assert!(metadata.status.success());
        assert!(metadata.stderr.is_empty());
        assert!(!metadata.stdout.contains(&b'?'));
    }
    for arguments in [vec!["-l"], vec!["-lr", "--depth", "0"]] {
        let totals = lx_command(&temp_dir)
            .args(arguments)
            .arg("--total-size")
            .arg(&listing)
            .output()
            .expect("list incomplete totals");
        assert!(totals.status.success());
        assert!(
            String::from_utf8_lossy(&totals.stdout).contains("0B?"),
            "unreadable descendants must mark the total as incomplete"
        );
    }

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
        vec!["-g"],
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
    assert!(String::from_utf8_lossy(&output.stdout).starts_with("\x1b[38;2;104;101;111m-\x1b[0m "));

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
        for badge in ["\x1b[94m-", "\x1b[35mM", "\x1b[38;2;188;240;146mS"] {
            assert!(
                stdout.contains(badge),
                "missing badge colour for {flags:?}: {stdout:?}"
            );
        }
    }

    let config_path = home_dir.join(".config/lx/config");
    let configuration = fs::read_to_string(&config_path).expect("read icon config");
    fs::write(
        &config_path,
        configuration.replacen("[icons]", "[icons]\nenabled = true", 1),
    )
    .expect("enable Git icons with empty file icon overrides");
    for flags in [
        vec!["--git"],
        vec!["--git", "-c"],
        vec!["--git", "-1"],
        vec!["--git", "-r"],
        vec!["--git", "-l"],
        vec!["--git", "-lr"],
    ] {
        let output = lx_command(&home_dir)
            .args(&flags)
            .arg(repository.join("src"))
            .output()
            .expect("render Git icons");
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let stdout = String::from_utf8_lossy(&output.stdout);
        for (icon, filename) in [
            ("-", "clean"),
            ("\u{f040}", "modified name"),
            ("\u{f058}", "staged"),
        ] {
            let line = stdout
                .lines()
                .find(|line| line.contains(filename))
                .expect("Git status row");
            assert!(line.contains(icon), "missing icon for {flags:?}: {line}");
        }
        if flags.contains(&"-l") {
            let mut lines = stdout.lines();
            let header = lines.next().expect("Git header");
            let position = header.find("Name").expect("Name column");
            for line in lines {
                let filename_position = line.rfind("  ").expect("column spacing") + 2;
                assert_eq!(
                    unicode_width::UnicodeWidthStr::width(&line[..filename_position]),
                    position,
                    "Git icon changed column alignment: {line}"
                );
            }
        }
    }
    let summary = lx_command(&home_dir)
        .args(["--git", "-1"])
        .arg(&repository)
        .output()
        .expect("render Git directory icons");
    let stdout = String::from_utf8_lossy(&summary.stdout);
    assert!(stdout.lines().any(|line| line == "\u{f074} src"));
    assert!(stdout.lines().any(|line| line == "\u{f059} new"));
    let coloured = lx_command(&home_dir)
        .env_remove("NO_COLOR")
        .env("CLICOLOR_FORCE", "1")
        .args(["--git", "-1"])
        .arg(repository.join("src/clean"))
        .output()
        .expect("render themed Git icon");
    assert!(String::from_utf8_lossy(&coloured.stdout).starts_with("\x1b[94m-\x1b[0m "));
    for flags in [vec!["--plain"], vec!["--zero"]] {
        let output = lx_command(&home_dir)
            .args(flags)
            .arg(repository.join("src/staged"))
            .output()
            .expect("render plain output with icons enabled");
        assert!(output.status.success());
        assert!(!String::from_utf8_lossy(&output.stdout).contains('\u{f058}'));
    }
    fs::write(&config_path, configuration).expect("restore letter Git badges");

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
                && matches!(
                    error.raw_os_error(),
                    Some(libc::EPERM | libc::EINVAL | libc::EILSEQ)
                ) =>
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
}

fn without_ansi(text: &str) -> String {
    let mut characters = text.chars().peekable();
    let mut plain = String::new();
    while let Some(character) = characters.next() {
        if character == '\u{1b}' && characters.peek() == Some(&'[') {
            characters.next();
            for character in characters.by_ref() {
                if character == 'm' {
                    break;
                }
            }
        } else {
            plain.push(character);
        }
    }
    plain
}

#[test]
fn toml_themes_merge_properties_and_keep_styled_columns_aligned() {
    let example: lx_cli::Config =
        toml::from_str(include_str!("../config.example")).expect("example config");
    assert!(!example.display.language_colors);
    let builtin: lx_cli::Config =
        toml::from_str(include_str!("../themes/default.toml")).expect("default theme");
    assert_eq!(
        builtin.theme.style("filekinds.regular").foreground,
        Some(colored::Color::TrueColor {
            r: 226,
            g: 226,
            b: 227
        })
    );
    let temp_dir = create_temp_dir("themes");
    let home_dir = temp_dir.join("home");
    let config_dir = home_dir.join(".config/lx");
    let directory = temp_dir.join("entries");
    fs::create_dir_all(config_dir.join("themes")).expect("create theme directory");
    fs::create_dir_all(directory.join(".hidden")).expect("create entries");
    fs::create_dir(directory.join("folder")).expect("create folder");
    fs::write(directory.join("界.rs"), "source").expect("write source");
    fs::write(directory.join("README.md"), vec![b'x'; 1500]).expect("write document");
    fs::write(
        config_dir.join("config"),
        "[display]\nlong_format_fields = ['owner']\n",
    )
    .expect("write legacy config to check preferred path");
    fs::write(
        config_dir.join("themes/custom.toml"),
        r##"
[style.filekinds]
directory = { foreground = "#123456", attributes = ["bold", "underline"] }
regular = { foreground = "#ABC" }
[style.file_type]
source = { foreground = "ABCDEF", attributes = ["italic"] }
[style.filenames]
hidden = { foreground = "default" }
[style.header]
attributes = ["bold", "underline"]
[style.permissions]
read = { foreground = "#F1FA8C", attributes = ["italic"] }
[style.size.kilobytes]
number = { foreground = "#112233" }
unit = { foreground = "#445566", attributes = ["bold"] }
[style.tree]
foreground = "#112233"
"##,
    )
    .expect("write custom theme");
    let configuration = r##"
theme = "custom"
[display]
long_format_fields = ["filename", "size", "owner", "modified", "permissions"]
column_spacing = 3
language_colors = true
[display.long_format_styles]
permissions = ["bold"]
[colors]
directory = "red"
[style.filekinds]
directory = { foreground = "#9BE3ED" }
[style.file_type]
source = { attributes = [] }
[style.header]
attributes = []
[style.permissions]
write = { attributes = ["italic"], background = "#222125" }
"##;
    let config_path = config_dir.join("config.toml");
    fs::write(&config_path, configuration).expect("write preferred config");
    for flags in [vec!["-al"], vec!["-alr"]] {
        let styled = lx_command(&home_dir)
            .env_remove("NO_COLOR")
            .env("CLICOLOR_FORCE", "1")
            .args(&flags)
            .arg(&directory)
            .output()
            .expect("render theme");
        assert!(styled.status.success());
        assert!(
            styled.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&styled.stderr)
        );
        let stdout = String::from_utf8_lossy(&styled.stdout);
        assert!(
            stdout.contains("\x1b[1;4;38;2;155;227;237mfolder"),
            "property merge failed: {stdout:?}"
        );
        assert!(
            stdout.contains("\x1b[1;4m.hidden"),
            "hidden attributes did not inherit: {stdout:?}"
        );
        assert!(
            stdout.contains("\x1b[38;2;171;205;239m界.rs"),
            "empty attributes did not clear theme: {stdout:?}"
        );
        assert!(
            stdout.contains("\x1b[1;38;2;241;250;140mr"),
            "inline/theme permission attributes lost: {stdout:?}"
        );
        assert!(
            stdout.contains("\x1b[3;48;2;34;33;37;38;2;255;199;158mw"),
            "inline attributes or background lost: {stdout:?}"
        );
        assert!(
            stdout.contains("\x1b[38;2;17;34;51m1.5"),
            "size number role missing: {stdout:?}"
        );
        assert!(
            stdout.contains("\x1b[1;38;2;68;85;102mK"),
            "size unit role missing: {stdout:?}"
        );
        let plain = lx_command(&home_dir)
            .args(&flags)
            .arg(&directory)
            .output()
            .expect("render without colour");
        assert_eq!(without_ansi(&stdout).as_bytes(), plain.stdout);
        let lines: Vec<_> = String::from_utf8(plain.stdout)
            .expect("UTF-8 output")
            .lines()
            .map(str::to_owned)
            .collect();
        let permission_positions: Vec<_> = lines
            .iter()
            .skip(1)
            .map(|line| {
                unicode_width::UnicodeWidthStr::width(
                    line.get(..line.len() - 10).expect("permission suffix"),
                )
            })
            .collect();
        assert!(
            permission_positions
                .iter()
                .all(|position| *position == permission_positions[0]),
            "styling changed alignment: {lines:?}"
        );
    }

    for invalid_theme in [
        "[style.filekinds]\ndirectory = { foreground = 'invalid' }\n",
        "[style.filekinds]\ndirectory = { foreground = 'green', attributes = ['italic'] }\n[style.typo]\nforeground = 'red'\n",
        "[display]\ncolumn_spacing = 99\n",
    ] {
        fs::write(config_dir.join("themes/custom.toml"), invalid_theme)
            .expect("write malformed theme");
        let output = lx_command(&home_dir)
            .env_remove("NO_COLOR")
            .env("CLICOLOR_FORCE", "1")
            .args(["-al"])
            .arg(&directory)
            .output()
            .expect("list despite malformed theme");
        assert!(output.status.success());
        assert!(
            !output.stderr.is_empty(),
            "invalid theme should produce a diagnostic"
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("38;2;155;227;237mfolder"),
            "config overrides lost after theme failure: {stdout:?}"
        );
        assert!(
            !stdout.contains("\x1b[3;38;2;155;227;237mfolder"),
            "invalid theme applied partially"
        );
        assert_eq!(
            without_ansi(&stdout)
                .lines()
                .next()
                .expect("header")
                .split_whitespace()
                .collect::<Vec<_>>(),
            ["Name", "Size", "Owner", "Modified", "Permissions"]
        );
    }
    fs::remove_file(config_dir.join("themes/custom.toml")).expect("remove theme");
    let missing = lx_command(&home_dir)
        .args(["-l"])
        .arg(&directory)
        .output()
        .expect("list despite missing theme");
    assert!(missing.status.success());
    assert!(!missing.stderr.is_empty());
    fs::write(&config_path, "theme = '../escape'\n").expect("set invalid theme name");
    let traversal = lx_command(&home_dir)
        .args(["-l"])
        .arg(&directory)
        .output()
        .expect("reject path theme name");
    assert!(traversal.status.success());
    assert!(!traversal.stderr.is_empty());
}
