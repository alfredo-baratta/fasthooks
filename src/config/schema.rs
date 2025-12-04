//! Configuration schema definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// FastHooks configuration version
    #[serde(default = "default_version")]
    pub version: String,

    /// Global settings
    #[serde(default)]
    pub settings: Settings,

    /// Hook definitions
    #[serde(default)]
    pub hooks: HashMap<String, Hook>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: default_version(),
            settings: Settings::default(),
            hooks: HashMap::new(),
        }
    }
}

fn default_version() -> String {
    "1".to_string()
}

/// Global settings for FastHooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Enable parallel execution of tasks within hooks
    #[serde(default = "default_true")]
    pub parallel: bool,

    /// Maximum number of parallel tasks (0 = auto-detect based on CPU cores)
    #[serde(default)]
    pub max_parallel: usize,

    /// Show execution time statistics
    #[serde(default = "default_true")]
    pub show_stats: bool,

    /// Show carbon savings estimate
    #[serde(default = "default_true")]
    pub show_carbon_savings: bool,

    /// Fail fast: stop on first error
    #[serde(default = "default_true")]
    pub fail_fast: bool,

    /// Skip hooks if CI environment detected
    #[serde(default)]
    pub skip_ci: bool,

    /// Colors in output
    #[serde(default = "default_true")]
    pub colors: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            parallel: true,
            max_parallel: 0,
            show_stats: true,
            show_carbon_savings: true,
            fail_fast: true,
            skip_ci: false,
            colors: true,
        }
    }
}

/// A Git hook definition
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Hook {
    /// Tasks to run for this hook
    #[serde(default)]
    pub tasks: Vec<Task>,

    /// Run tasks in parallel (overrides global setting)
    #[serde(default)]
    pub parallel: Option<bool>,

    /// Fail fast for this hook (overrides global setting)
    #[serde(default)]
    pub fail_fast: Option<bool>,

    /// Skip this hook in CI
    #[serde(default)]
    pub skip_ci: Option<bool>,
}

/// A task within a hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task name (for display)
    pub name: String,

    /// Command to execute
    pub run: String,

    /// Glob patterns for files to match (lint-staged style)
    /// Supports negation with ! prefix (e.g., "!*.test.js")
    #[serde(default)]
    pub glob: Option<String>,

    /// Only run on staged files
    #[serde(default = "default_true")]
    pub staged: bool,

    /// Working directory for the command
    #[serde(default)]
    pub cwd: Option<String>,

    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Continue even if this task fails
    #[serde(default)]
    pub allow_failure: bool,

    /// Condition to run this task (e.g., "branch == main", "branch != main", "env:CI")
    #[serde(rename = "if", default)]
    pub condition: Option<String>,

    /// Task dependencies - names of tasks that must run before this one
    #[serde(default)]
    pub depends_on: Vec<String>,
}

/// Supported Git hook types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookType {
    PreCommit,
    PrepareCommitMsg,
    CommitMsg,
    PostCommit,
    PrePush,
    PreRebase,
    PostCheckout,
    PostMerge,
    PreAutoGc,
}

impl HookType {
    /// Get the hook name as used by Git
    pub fn as_str(&self) -> &'static str {
        match self {
            HookType::PreCommit => "pre-commit",
            HookType::PrepareCommitMsg => "prepare-commit-msg",
            HookType::CommitMsg => "commit-msg",
            HookType::PostCommit => "post-commit",
            HookType::PrePush => "pre-push",
            HookType::PreRebase => "pre-rebase",
            HookType::PostCheckout => "post-checkout",
            HookType::PostMerge => "post-merge",
            HookType::PreAutoGc => "pre-auto-gc",
        }
    }

    /// Get all supported hook types
    pub fn all() -> &'static [HookType] {
        &[
            HookType::PreCommit,
            HookType::PrepareCommitMsg,
            HookType::CommitMsg,
            HookType::PostCommit,
            HookType::PrePush,
            HookType::PreRebase,
            HookType::PostCheckout,
            HookType::PostMerge,
            HookType::PreAutoGc,
        ]
    }

    /// Parse a hook type from string
    pub fn from_str(s: &str) -> Option<HookType> {
        match s {
            "pre-commit" => Some(HookType::PreCommit),
            "prepare-commit-msg" => Some(HookType::PrepareCommitMsg),
            "commit-msg" => Some(HookType::CommitMsg),
            "post-commit" => Some(HookType::PostCommit),
            "pre-push" => Some(HookType::PrePush),
            "pre-rebase" => Some(HookType::PreRebase),
            "post-checkout" => Some(HookType::PostCheckout),
            "post-merge" => Some(HookType::PostMerge),
            "pre-auto-gc" => Some(HookType::PreAutoGc),
            _ => None,
        }
    }

    /// Get the number of arguments this hook receives from Git
    #[allow(dead_code)]
    pub fn arg_count(&self) -> usize {
        match self {
            HookType::PreCommit => 0,
            HookType::PrepareCommitMsg => 3, // commit_msg_file, source, sha1
            HookType::CommitMsg => 1,        // commit_msg_file
            HookType::PostCommit => 0,
            HookType::PrePush => 2,      // remote_name, remote_url
            HookType::PreRebase => 2,    // upstream, rebased_branch
            HookType::PostCheckout => 3, // previous_head, new_head, is_branch_checkout
            HookType::PostMerge => 1,    // is_squash_merge
            HookType::PreAutoGc => 0,
        }
    }
}

impl std::fmt::Display for HookType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_type_roundtrip() {
        for hook_type in HookType::all() {
            let s = hook_type.as_str();
            let parsed = HookType::from_str(s);
            assert_eq!(parsed, Some(*hook_type));
        }
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.version, "1");
        assert!(config.settings.parallel);
        assert!(config.hooks.is_empty());
    }

    #[test]
    fn test_task_with_dependencies() {
        let toml = r#"
            name = "test"
            run = "cargo test"
            depends_on = ["lint", "format"]
        "#;
        let task: Task = toml::from_str(toml).unwrap();
        assert_eq!(task.depends_on, vec!["lint", "format"]);
    }

    #[test]
    fn test_task_with_condition() {
        let toml = r#"
            name = "deploy"
            run = "npm run deploy"
            if = "branch == main"
        "#;
        let task: Task = toml::from_str(toml).unwrap();
        assert_eq!(task.condition, Some("branch == main".to_string()));
    }
}
