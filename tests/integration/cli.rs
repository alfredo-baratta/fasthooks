//! CLI integration tests

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("fasthooks").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("fasthooks"));
}

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("fasthooks").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Blazing fast Git hooks manager"));
}

#[test]
fn test_list_no_config() {
    let mut cmd = Command::cargo_bin("fasthooks").unwrap();
    cmd.arg("list")
        .current_dir(std::env::temp_dir())
        .assert()
        .success();
}
