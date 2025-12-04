//! Git hooks management module
//!
//! Handles installation, uninstallation, and execution of Git hooks.

mod git;
mod installer;
mod template;

pub use git::GitRepository;
pub use installer::HookInstaller;
pub use template::HookTemplate;

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Get the Git hooks directory for the current repository
pub fn get_hooks_dir() -> Result<PathBuf> {
    let repo = GitRepository::discover()?;
    Ok(repo.hooks_dir())
}

/// Check if FastHooks is installed in the current repository
pub fn is_installed() -> Result<bool> {
    let hooks_dir = get_hooks_dir()?;
    let pre_commit = hooks_dir.join("pre-commit");

    if pre_commit.exists() {
        let content =
            std::fs::read_to_string(&pre_commit).context("Failed to read pre-commit hook")?;
        Ok(content.contains("fasthooks"))
    } else {
        Ok(false)
    }
}
