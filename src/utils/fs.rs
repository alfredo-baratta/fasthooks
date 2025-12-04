//! File system utilities

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Ensure a directory exists, creating it if necessary
#[allow(dead_code)]
pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_ensure_dir() {
        let temp = TempDir::new().unwrap();
        let new_dir = temp.path().join("new").join("nested").join("dir");

        assert!(!new_dir.exists());
        ensure_dir(&new_dir).unwrap();
        assert!(new_dir.exists());
    }
}
