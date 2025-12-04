//! Install Git hooks

use crate::config::{self, HookType};
use crate::hooks::HookInstaller;
use anyhow::Result;
use colored::Colorize;

/// Run the install command
pub fn run(hook: Option<String>) -> Result<()> {
    let config = config::load_config()?;
    let installer = HookInstaller::new()?;

    match hook {
        Some(hook_name) => {
            // Install specific hook
            let hook_type = HookType::from_str(&hook_name).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown hook type: {}. Valid hooks: pre-commit, pre-push, commit-msg, etc.",
                    hook_name
                )
            })?;

            if !config.hooks.contains_key(&hook_name) {
                eprintln!(
                    "{} Hook '{}' is not configured in fasthooks.toml",
                    "Warning:".yellow().bold(),
                    hook_name
                );
            }

            installer.install_hook(hook_type)?;
            println!("{} Installed {} hook", "✓".green().bold(), hook_name.cyan());
        }
        None => {
            // Install all configured hooks
            let mut installed = 0;
            for hook_name in config.hooks.keys() {
                if let Some(hook_type) = HookType::from_str(hook_name) {
                    installer.install_hook(hook_type)?;
                    println!("{} Installed {} hook", "✓".green().bold(), hook_name.cyan());
                    installed += 1;
                }
            }

            if installed == 0 {
                println!(
                    "{} No hooks configured in fasthooks.toml",
                    "Warning:".yellow().bold()
                );
            } else {
                println!();
                println!("{} Installed {} hook(s)", "✓".green().bold(), installed);
            }
        }
    }

    Ok(())
}
