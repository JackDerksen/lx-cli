use std::fs;
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
fn lists_a_directly_targeted_file() {
    let temp_dir = create_temp_dir("file-target");
    let file_path = temp_dir.join("hello.txt");
    fs::write(&file_path, "hello").expect("write file");

    let output = lx_command(&temp_dir)
        .arg(&file_path)
        .output()
        .expect("run lx");

    assert!(
        output.status.success(),
        "lx should succeed for file targets"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello.txt"), "stdout was: {stdout}");

    fs::remove_dir_all(&temp_dir).expect("remove temp dir");
}

#[test]
fn returns_non_zero_for_missing_paths() {
    let temp_dir = create_temp_dir("missing-path");
    let missing_path = temp_dir.join("missing.txt");

    let output = lx_command(&temp_dir)
        .arg(&missing_path)
        .output()
        .expect("run lx");

    assert!(
        !output.status.success(),
        "lx should return a non-zero exit status for missing paths"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cannot access"),
        "stderr should contain a helpful message: {stderr}"
    );

    fs::remove_dir_all(&temp_dir).expect("remove temp dir");
}
