//! Configuration module for FastHooks
//!
//! Handles parsing and validation of fasthooks.toml configuration files.

mod parser;
mod schema;

pub use parser::ConfigParser;
pub use schema::{Config, Hook, HookType, Settings, Task};

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Default configuration file name
pub const CONFIG_FILE_NAME: &str = "fasthooks.toml";

/// Alternative configuration file names (for compatibility)
pub const ALT_CONFIG_FILE_NAMES: &[&str] =
    &[".fasthooks.toml", "fasthooks.yaml", ".fasthooks.yaml"];

/// Find the configuration file in the repository
pub fn find_config_file() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    find_config_file_from(&current_dir)
}

/// Find the configuration file starting from a specific directory
pub fn find_config_file_from(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir.to_path_buf();

    loop {
        // Check primary config file
        let config_path = current.join(CONFIG_FILE_NAME);
        if config_path.exists() {
            return Some(config_path);
        }

        // Check alternative names
        for alt_name in ALT_CONFIG_FILE_NAMES {
            let alt_path = current.join(alt_name);
            if alt_path.exists() {
                return Some(alt_path);
            }
        }

        // Move to parent directory
        if !current.pop() {
            break;
        }
    }

    None
}

/// Load configuration from the default location
pub fn load_config() -> Result<Config> {
    let config_path = find_config_file()
        .context("No fasthooks.toml found. Run 'fasthooks init' to create one.")?;

    ConfigParser::parse_file(&config_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(CONFIG_FILE_NAME);
        fs::write(&config_path, "").unwrap();

        let found = find_config_file_from(temp_dir.path());
        assert!(found.is_some());
        assert_eq!(found.unwrap(), config_path);
    }

    #[test]
    fn test_find_config_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let found = find_config_file_from(temp_dir.path());
        assert!(found.is_none());
    }
}
