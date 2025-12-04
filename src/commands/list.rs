//! List configured hooks

use crate::config;
use crate::hooks;
use anyhow::Result;
use colored::Colorize;

/// List all configured hooks
pub fn run() -> Result<()> {
    let config = match config::load_config() {
        Ok(c) => c,
        Err(_) => {
            println!(
                "{} No fasthooks.toml found. Run {} to create one.",
                "Info:".cyan().bold(),
                "fasthooks init".cyan()
            );
            return Ok(());
        }
    };

    let is_installed = hooks::is_installed().unwrap_or(false);

    println!("{}", "FastHooks Configuration".bold());
    println!("{}", "═".repeat(40));
    println!();

    if config.hooks.is_empty() {
        println!("  No hooks configured.");
        println!();
        println!(
            "  Add a hook with: {}",
            "fasthooks add pre-commit \"npm run lint\"".cyan()
        );
        return Ok(());
    }

    for (hook_name, hook) in &config.hooks {
        let status = if is_installed {
            "●".green()
        } else {
            "○".yellow()
        };

        println!("{} {} hook", status, hook_name.cyan().bold());

        if hook.tasks.is_empty() {
            println!("    (no tasks)");
        } else {
            for task in &hook.tasks {
                let glob_info = task
                    .glob
                    .as_ref()
                    .map(|g| format!(" [{}]", g.dimmed()))
                    .unwrap_or_default();

                println!("    {} {}{}", "→".dimmed(), task.name, glob_info);
                println!("      {}", task.run.dimmed());
            }
        }
        println!();
    }

    // Show legend
    println!("{}", "Legend:".dimmed());
    println!(
        "  {} Installed  {} Not installed",
        "●".green(),
        "○".yellow()
    );

    Ok(())
}
