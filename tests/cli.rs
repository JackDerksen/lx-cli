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
