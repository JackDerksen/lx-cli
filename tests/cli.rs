use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
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
