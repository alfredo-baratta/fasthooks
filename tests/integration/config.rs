//! Configuration integration tests

use std::fs;
use tempfile::TempDir;

#[test]
fn test_parse_minimal_config() {
    let config = r#"
version = "1"

[hooks.pre-commit]
[[hooks.pre-commit.tasks]]
name = "test"
run = "echo test"
"#;

    let parsed: toml::Value = toml::from_str(config).unwrap();
    assert_eq!(parsed["version"].as_str(), Some("1"));
}

#[test]
fn test_parse_full_config() {
    let config = r#"
version = "1"

[settings]
parallel = true
max_parallel = 4
show_stats = true
show_carbon_savings = true
fail_fast = true
skip_ci = false
colors = true

[hooks.pre-commit]
parallel = true

[[hooks.pre-commit.tasks]]
name = "lint"
run = "npm run lint"
glob = "*.js"
staged = true

[[hooks.pre-commit.tasks]]
name = "format"
run = "npm run format"
glob = "*.{js,ts}"
staged = true

[hooks.pre-push]
[[hooks.pre-push.tasks]]
name = "test"
run = "npm test"
"#;

    let parsed: toml::Value = toml::from_str(config).unwrap();
    assert!(parsed["hooks"]["pre-commit"]["tasks"].is_array());
    assert!(parsed["hooks"]["pre-push"]["tasks"].is_array());
}
