//! Git repository operations using libgit2
//!
//! Provides a high-level wrapper around libgit2 for git operations
//! needed by FastHooks, including staged file detection and repository info.

use anyhow::{Context, Result};
use git2::Repository;
use std::path::PathBuf;

/// Wrapper around git2::Repository for common operations
#[allow(dead_code)]
pub struct GitRepository {
    repo: Repository,
}

#[allow(dead_code)]
impl GitRepository {
    /// Discover the Git repository from the current directory
    pub fn discover() -> Result<Self> {
        let repo = Repository::discover(".")
            .context("Not a git repository (or any of the parent directories)")?;
        Ok(Self { repo })
    }

    /// Open a Git repository at a specific path
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let repo = Repository::open(path)
            .with_context(|| format!("Failed to open repository at {}", path.display()))?;
        Ok(Self { repo })
    }

    /// Get the path to the .git directory
    pub fn git_dir(&self) -> PathBuf {
        self.repo.path().to_path_buf()
    }

    /// Get the path to the hooks directory
    pub fn hooks_dir(&self) -> PathBuf {
        self.repo.path().join("hooks")
    }

    /// Get the repository root (working directory)
    pub fn workdir(&self) -> Option<PathBuf> {
        self.repo.workdir().map(|p| p.to_path_buf())
    }

    /// Get list of staged files
    pub fn staged_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let head = self.repo.head().ok();
        let head_tree = head.as_ref().and_then(|h| h.peel_to_tree().ok());

        let diff = self
            .repo
            .diff_tree_to_index(head_tree.as_ref(), None, None)
            .context("Failed to get staged changes")?;

        diff.foreach(
            &mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    files.push(path.to_path_buf());
                }
                true
            },
            None,
            None,
            None,
        )
        .context("Failed to iterate over staged files")?;

        Ok(files)
    }

    /// Get the current branch name
    pub fn current_branch(&self) -> Result<Option<String>> {
        let head = match self.repo.head() {
            Ok(head) => head,
            Err(_) => return Ok(None),
        };

        if head.is_branch() {
            Ok(head.shorthand().map(|s| s.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Check if we're in a detached HEAD state
    pub fn is_detached(&self) -> bool {
        self.repo.head_detached().unwrap_or(false)
    }

    /// Get the remote URL for 'origin'
    pub fn origin_url(&self) -> Option<String> {
        self.repo
            .find_remote("origin")
            .ok()
            .and_then(|r| r.url().map(|s| s.to_string()))
    }

    /// Stash uncommitted changes
    pub fn stash(&self, message: &str) -> Result<git2::Oid> {
        let signature = self
            .repo
            .signature()
            .context("Failed to get default signature")?;

        let mut repo =
            Repository::discover(".").context("Failed to reopen repository for stashing")?;

        repo.stash_save(
            &signature,
            message,
            Some(git2::StashFlags::INCLUDE_UNTRACKED),
        )
        .context("Failed to stash changes")
    }

    /// Pop the most recent stash
    pub fn stash_pop(&self) -> Result<()> {
        let mut repo =
            Repository::discover(".").context("Failed to reopen repository for stash pop")?;

        repo.stash_pop(0, None).context("Failed to pop stash")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn init_test_repo() -> (TempDir, GitRepository) {
        let temp_dir = TempDir::new().unwrap();
        Repository::init(temp_dir.path()).unwrap();
        let repo = GitRepository::open(temp_dir.path()).unwrap();
        (temp_dir, repo)
    }

    #[test]
    fn test_hooks_dir() {
        let (_temp_dir, repo) = init_test_repo();
        let hooks_dir = repo.hooks_dir();
        // Just verify it ends with .git/hooks - path prefix may differ due to symlinks
        // (e.g., macOS /var -> /private/var)
        assert!(hooks_dir.ends_with("hooks"));
        assert!(hooks_dir.to_string_lossy().contains(".git"));
    }

    #[test]
    fn test_staged_files_empty() {
        let (_temp_dir, repo) = init_test_repo();
        let files = repo.staged_files().unwrap();
        assert!(files.is_empty());
    }
}
