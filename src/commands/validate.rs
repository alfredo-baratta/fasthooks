//! Configuration validation command

use crate::config::{self, ConfigParser};
use anyhow::Result;
use colored::Colorize;

/// Run the validate command
pub fn run() -> Result<()> {
    println!("{} Validating configuration...\n", "→".cyan().bold());

    // Find and load config
    let config_path = match config::find_config_file() {
        Some(path) => path,
        None => {
            println!("{} No configuration file found.\n", "✗".red().bold());
            println!("  Create one with: {}", "fasthooks init".cyan());
            return Ok(());
        }
    };

    println!("  {} {}\n", "Config file:".dimmed(), config_path.display());

    // Parse the configuration
    let config = match ConfigParser::parse_file(&config_path) {
        Ok(config) => config,
        Err(e) => {
            // Show the full error chain for detailed error messages
            println!("{} Parse error:\n", "✗".red().bold());
            for cause in e.chain() {
                println!("{}", cause);
            }
            return Ok(());
        }
    };

    // Validate the configuration
    match ConfigParser::validate(&config) {
        Ok(()) => {
            println!("{} Configuration is valid!\n", "✓".green().bold());

            // Show summary
            println!("{}", "Summary:".bold());
            println!("  {} Version: {}", "•".dimmed(), config.version);
            println!(
                "  {} Hooks configured: {}",
                "•".dimmed(),
                config.hooks.len()
            );

            for (hook_name, hook) in &config.hooks {
                println!(
                    "    {} {} ({} task{})",
                    "→".cyan(),
                    hook_name,
                    hook.tasks.len(),
                    if hook.tasks.len() == 1 { "" } else { "s" }
                );

                for task in &hook.tasks {
                    let mut extras = Vec::new();

                    if let Some(glob) = &task.glob {
                        extras.push(format!("glob: {}", glob));
                    }
                    if let Some(condition) = &task.condition {
                        extras.push(format!("if: {}", condition));
                    }
                    if !task.depends_on.is_empty() {
                        extras.push(format!("depends_on: {:?}", task.depends_on));
                    }

                    let extra_str = if extras.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", extras.join(", "))
                    };

                    println!("      {} {}{}", "•".dimmed(), task.name, extra_str.dimmed());
                }
            }

            println!();
            println!("{} Settings:", "•".dimmed());
            println!(
                "    Parallel: {}, Fail fast: {}, Show stats: {}",
                if config.settings.parallel {
                    "yes".green()
                } else {
                    "no".red()
                },
                if config.settings.fail_fast {
                    "yes".green()
                } else {
                    "no".red()
                },
                if config.settings.show_stats {
                    "yes".green()
                } else {
                    "no".red()
                }
            );
        }
        Err(errors) => {
            print!("{}", ConfigParser::format_validation_errors(&errors));
        }
    }

    Ok(())
}
