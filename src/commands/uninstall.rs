//! Uninstall FastHooks

use crate::config::CONFIG_FILE_NAME;
use crate::hooks::HookInstaller;
use anyhow::Result;
use colored::Colorize;

/// Run the uninstall command
pub fn run() -> Result<()> {
    let installer = HookInstaller::new()?;
    installer.uninstall_all()?;

    println!("{} Uninstalled all FastHooks git hooks", "✓".green().bold());
    println!();
    println!(
        "Note: {} was not removed. Delete it manually if needed.",
        CONFIG_FILE_NAME.cyan()
    );

    Ok(())
}
