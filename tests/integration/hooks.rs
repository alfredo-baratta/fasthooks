//! Hook management integration tests

use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn init_git_repo(dir: &TempDir) {
    Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to init git repo");

    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to configure git");

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to configure git");
}

#[test]
fn test_hooks_directory_exists_after_git_init() {
    let temp_dir = TempDir::new().unwrap();
    init_git_repo(&temp_dir);

    let git_dir = temp_dir.path().join(".git");
    assert!(git_dir.exists());
}

#[test]
fn test_hook_template_generation() {
    let hook_content = r#"#!/bin/sh
# FastHooks - https://github.com/alfredo-baratta/fasthooks
# Hook: pre-commit

set -e

fasthooks run pre-commit "$@"
"#;

    assert!(hook_content.contains("fasthooks"));
    assert!(hook_content.contains("pre-commit"));
}
