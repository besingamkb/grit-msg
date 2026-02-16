use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::tempdir;

#[test]
fn clear_aiapi_keys_removes_fallback_files_and_exits_successfully() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path();
    let current_fallback = home.join(".grit-msg-groq-key");
    let legacy_fallback = home.join(".git-ai-commit-groq-key");

    fs::write(&current_fallback, "gsk_current").expect("write current fallback");
    fs::write(&legacy_fallback, "gsk_legacy").expect("write legacy fallback");

    let mut cmd = binary_command();
    cmd.current_dir(home)
        .env("HOME", home)
        .env("USER", "test-user")
        .arg("--clear-aiapi-keys")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleared stored Groq API keys"));

    assert!(!current_fallback.exists());
    assert!(!legacy_fallback.exists());
}

#[test]
fn exits_with_error_when_no_staged_changes_exist() {
    let temp = tempdir().expect("tempdir");
    let repo_dir = temp.path().join("repo");
    fs::create_dir(&repo_dir).expect("create repo dir");
    git_init_repo(&repo_dir);

    let mut cmd = binary_command();
    cmd.current_dir(&repo_dir)
        .env("HOME", temp.path())
        .env("USER", "test-user")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No staged changes found"));
}

fn git_init_repo(path: &Path) {
    let status = StdCommand::new("git")
        .args(["init", "-q"])
        .current_dir(path)
        .status()
        .expect("run git init");
    assert!(status.success());
}

fn binary_command() -> StdCommand {
    StdCommand::new(env!("CARGO_BIN_EXE_grit-msg"))
}
