//! Hook installation and management

use super::{GitRepository, HookTemplate};
use crate::config::HookType;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Handles hook installation and uninstallation
pub struct HookInstaller {
    repo: GitRepository,
}

impl HookInstaller {
    /// Create a new HookInstaller for the current repository
    pub fn new() -> Result<Self> {
        let repo = GitRepository::discover()?;
        Ok(Self { repo })
    }

    /// Install a specific hook
    pub fn install_hook(&self, hook_type: HookType) -> Result<()> {
        let hooks_dir = self.repo.hooks_dir();

        // Ensure hooks directory exists
        fs::create_dir_all(&hooks_dir).context("Failed to create hooks directory")?;

        let hook_path = hooks_dir.join(hook_type.as_str());
        let hook_content = HookTemplate::generate(hook_type);

        // Backup existing hook if it exists and isn't ours
        if hook_path.exists() {
            let existing = fs::read_to_string(&hook_path)?;
            if !existing.contains("fasthooks") {
                let backup_path = hooks_dir.join(format!("{}.backup", hook_type.as_str()));
                fs::rename(&hook_path, &backup_path).context("Failed to backup existing hook")?;
                tracing::info!("Backed up existing {} to {}.backup", hook_type, hook_type);
            }
        }

        // Write the hook
        fs::write(&hook_path, hook_content)
            .with_context(|| format!("Failed to write {} hook", hook_type))?;

        // Make executable on Unix
        Self::make_executable(&hook_path)?;

        tracing::info!("Installed {} hook", hook_type);
        Ok(())
    }

    /// Install all configured hooks
    #[allow(dead_code)]
    pub fn install_all(&self, hooks: &[HookType]) -> Result<()> {
        for hook_type in hooks {
            self.install_hook(*hook_type)?;
        }
        Ok(())
    }

    /// Uninstall a specific hook
    pub fn uninstall_hook(&self, hook_type: HookType) -> Result<()> {
        let hooks_dir = self.repo.hooks_dir();
        let hook_path = hooks_dir.join(hook_type.as_str());

        if hook_path.exists() {
            let content = fs::read_to_string(&hook_path)?;
            if content.contains("fasthooks") {
                fs::remove_file(&hook_path)
                    .with_context(|| format!("Failed to remove {} hook", hook_type))?;

                // Restore backup if exists
                let backup_path = hooks_dir.join(format!("{}.backup", hook_type.as_str()));
                if backup_path.exists() {
                    fs::rename(&backup_path, &hook_path)
                        .context("Failed to restore backup hook")?;
                    tracing::info!("Restored backup for {}", hook_type);
                }

                tracing::info!("Uninstalled {} hook", hook_type);
            } else {
                tracing::warn!(
                    "{} hook exists but wasn't installed by fasthooks",
                    hook_type
                );
            }
        }

        Ok(())
    }

    /// Uninstall all FastHooks hooks
    pub fn uninstall_all(&self) -> Result<()> {
        for hook_type in HookType::all() {
            self.uninstall_hook(*hook_type)?;
        }
        Ok(())
    }

    /// Make a file executable (Unix only)
    #[cfg(unix)]
    fn make_executable(path: &Path) -> Result<()> {
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)?;
        Ok(())
    }

    /// No-op on Windows (executable by extension)
    #[cfg(not(unix))]
    fn make_executable(_path: &Path) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Repository;
    use tempfile::TempDir;

    #[test]
    fn test_hook_installation() {
        // Save current dir and restore after test
        let original_dir = std::env::current_dir().unwrap();

        let temp_dir = TempDir::new().unwrap();
        Repository::init(temp_dir.path()).unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = std::panic::catch_unwind(|| {
            let installer = HookInstaller::new().unwrap();
            installer.install_hook(HookType::PreCommit).unwrap();

            let hook_path = temp_dir.path().join(".git/hooks/pre-commit");
            assert!(hook_path.exists());

            let content = fs::read_to_string(hook_path).unwrap();
            assert!(content.contains("fasthooks"));
        });

        // Always restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // Re-panic if test failed
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }
}
