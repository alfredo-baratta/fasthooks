//! Manually run a hook

use crate::config;
use crate::runner::TaskExecutor;
use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;

/// Run a hook manually
pub fn run(hook_name: String, files: Option<Vec<String>>, args: Vec<String>) -> Result<()> {
    let config = config::load_config()?;

    let hook = config
        .hooks
        .get(&hook_name)
        .with_context(|| format!("Hook '{}' not found in configuration", hook_name))?;

    println!("{} Running {} hook...", "→".cyan().bold(), hook_name.cyan());
    println!();

    // Create executor
    let executor = if let Some(file_list) = files {
        let paths: Vec<PathBuf> = file_list.into_iter().map(PathBuf::from).collect();
        TaskExecutor::with_files(config.settings.clone(), paths)?
    } else {
        TaskExecutor::new(config.settings.clone())?
    };

    // Add hook arguments if provided
    let executor = executor.with_hook_args(args);

    // Run the hook
    let runtime = tokio::runtime::Runtime::new()?;
    let result = runtime.block_on(executor.execute_hook(hook))?;

    // Display task results
    for task_result in &result.tasks {
        let status = if task_result.success {
            format!("{} {}", "✓".green(), task_result.name)
        } else {
            format!("{} {}", "✗".red(), task_result.name)
        };
        println!("  {} ({}ms)", status, task_result.duration_ms);

        // Show output for failed tasks
        if !task_result.success {
            if !task_result.stdout.is_empty() {
                println!("{}", task_result.stdout);
            }
            if !task_result.stderr.is_empty() {
                eprintln!("{}", task_result.stderr.red());
            }
        }
    }

    // Display stats
    println!(
        "{}",
        result.stats.format(config.settings.show_carbon_savings)
    );

    if !result.success {
        std::process::exit(1);
    }

    Ok(())
}
