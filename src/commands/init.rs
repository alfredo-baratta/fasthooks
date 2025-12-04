//! Initialize FastHooks in a repository

use crate::config::{self, ConfigParser, CONFIG_FILE_NAME};
use crate::hooks::HookInstaller;
use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

/// Run the init command
pub fn run(force: bool) -> Result<()> {
    let config_path = Path::new(CONFIG_FILE_NAME);

    // Check if config already exists
    if config_path.exists() && !force {
        eprintln!(
            "{} {} already exists. Use --force to overwrite.",
            "Error:".red().bold(),
            CONFIG_FILE_NAME
        );
        std::process::exit(1);
    }

    // Create default config
    let config_content = ConfigParser::default_config_content();
    fs::write(config_path, &config_content)
        .with_context(|| format!("Failed to create {}", CONFIG_FILE_NAME))?;

    println!("{} Created {}", "✓".green().bold(), CONFIG_FILE_NAME.cyan());

    // Install hooks
    let installer = HookInstaller::new()?;
    let config = config::load_config()?;

    for hook_name in config.hooks.keys() {
        if let Some(hook_type) = crate::config::HookType::from_str(hook_name) {
            installer.install_hook(hook_type)?;
            println!("{} Installed {} hook", "✓".green().bold(), hook_name.cyan());
        }
    }

    println!();
    println!("{}", "FastHooks initialized successfully!".green().bold());
    println!();
    println!("Next steps:");
    println!(
        "  1. Edit {} to configure your hooks",
        CONFIG_FILE_NAME.cyan()
    );
    println!("  2. Run {} to apply changes", "fasthooks install".cyan());
    println!();

    Ok(())
}
