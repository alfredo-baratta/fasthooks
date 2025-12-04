//! Add a command to a hook

use crate::config::{self, Config, ConfigParser, Hook, HookType, Task, CONFIG_FILE_NAME};
use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;

/// Add a command to a hook
pub fn run(hook_name: String, command: String) -> Result<()> {
    // Validate hook name
    if HookType::from_str(&hook_name).is_none() {
        return Err(anyhow::anyhow!(
            "Unknown hook type: '{}'. Valid hooks: pre-commit, pre-push, commit-msg, etc.",
            hook_name
        ));
    }

    // Load or create config
    let mut config = match config::load_config() {
        Ok(c) => c,
        Err(_) => {
            println!(
                "{} No config found, creating {}",
                "→".cyan(),
                CONFIG_FILE_NAME
            );
            Config::default()
        }
    };

    // Add the task to the hook
    let hook = config
        .hooks
        .entry(hook_name.clone())
        .or_insert_with(Hook::default);

    // Generate task name from command
    let task_name = command
        .split_whitespace()
        .take(2)
        .collect::<Vec<_>>()
        .join(" ");

    let task = Task {
        name: task_name.clone(),
        run: command.clone(),
        glob: None,
        staged: true,
        cwd: None,
        env: Default::default(),
        allow_failure: false,
        condition: None,
        depends_on: Vec::new(),
    };

    hook.tasks.push(task);

    // Save config
    let config_content = ConfigParser::to_toml(&config)?;
    fs::write(CONFIG_FILE_NAME, config_content).context("Failed to write configuration file")?;

    println!(
        "{} Added task '{}' to {} hook",
        "✓".green().bold(),
        task_name.cyan(),
        hook_name.cyan()
    );

    println!();
    println!("Run {} to update git hooks", "fasthooks install".cyan());

    Ok(())
}
