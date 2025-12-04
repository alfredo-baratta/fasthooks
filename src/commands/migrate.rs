//! Migrate from Husky to FastHooks

use crate::config::{Config, ConfigParser, Settings, Task, CONFIG_FILE_NAME};
use crate::hooks::HookInstaller;
use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Run the migrate command
pub fn run() -> Result<()> {
    println!("{} Migrating from Husky to FastHooks...", "→".cyan().bold());
    println!();

    // Check for Husky configuration
    let husky_dir = Path::new(".husky");
    let has_husky = husky_dir.exists() && husky_dir.is_dir();

    // Check for lint-staged configuration
    let lint_staged_config = find_lint_staged_config();

    if !has_husky && lint_staged_config.is_none() {
        println!(
            "{} No Husky or lint-staged configuration found.",
            "Warning:".yellow().bold()
        );
        println!("  Looking for:");
        println!("    - .husky/ directory");
        println!("    - lint-staged config in package.json");
        println!("    - .lintstagedrc file");
        return Ok(());
    }

    let mut config = Config {
        version: "1".to_string(),
        settings: Settings::default(),
        hooks: HashMap::new(),
    };

    // Migrate Husky hooks
    if has_husky {
        println!("{} Found .husky/ directory", "✓".green());
        migrate_husky_hooks(husky_dir, &mut config)?;
    }

    // Migrate lint-staged config
    if let Some(lint_staged) = lint_staged_config {
        println!("{} Found lint-staged configuration", "✓".green());
        migrate_lint_staged(&lint_staged, &mut config)?;
    }

    // Write new config
    let config_content = ConfigParser::to_toml(&config)?;
    fs::write(CONFIG_FILE_NAME, config_content).context("Failed to write fasthooks.toml")?;

    println!();
    println!("{} Created {}", "✓".green().bold(), CONFIG_FILE_NAME.cyan());

    // Install hooks
    let installer = HookInstaller::new()?;
    for hook_name in config.hooks.keys() {
        if let Some(hook_type) = crate::config::HookType::from_str(hook_name) {
            installer.install_hook(hook_type)?;
        }
    }

    println!();
    println!("{}", "Migration complete!".green().bold());
    println!();
    println!("Next steps:");
    println!(
        "  1. Review {} and adjust as needed",
        CONFIG_FILE_NAME.cyan()
    );
    println!(
        "  2. Remove .husky/ directory: {}",
        "rm -rf .husky".dimmed()
    );
    println!("  3. Remove husky from package.json dependencies");
    println!("  4. Test with: {}", "fasthooks run pre-commit".cyan());

    Ok(())
}

/// Migrate hooks from .husky directory
fn migrate_husky_hooks(husky_dir: &Path, config: &mut Config) -> Result<()> {
    let entries = fs::read_dir(husky_dir)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().unwrap().to_string_lossy();

            // Skip underscore files (like _)
            if filename.starts_with('_') || filename.starts_with('.') {
                continue;
            }

            // Check if it's a valid hook name
            if crate::config::HookType::from_str(&filename).is_some() {
                let content = fs::read_to_string(&path)?;
                let commands = parse_husky_script(&content);

                if !commands.is_empty() {
                    let hook = config.hooks.entry(filename.to_string()).or_default();

                    for cmd in commands {
                        hook.tasks.push(Task {
                            name: extract_task_name(&cmd),
                            run: cmd,
                            glob: None,
                            staged: true,
                            cwd: None,
                            env: HashMap::new(),
                            allow_failure: false,
                            condition: None,
                            depends_on: Vec::new(),
                        });
                    }

                    println!("  {} Migrated {} hook", "→".dimmed(), filename.cyan());
                }
            }
        }
    }

    Ok(())
}

/// Parse a Husky shell script to extract commands
fn parse_husky_script(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && !trimmed.starts_with("#!/")
                && !trimmed.starts_with(". \"")  // Husky init line
                && !trimmed.contains("husky.sh")
        })
        .map(|s| s.trim().to_string())
        .collect()
}

/// Find lint-staged configuration
fn find_lint_staged_config() -> Option<LintStagedConfig> {
    // Check package.json
    if let Ok(content) = fs::read_to_string("package.json") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(lint_staged) = json.get("lint-staged") {
                return Some(LintStagedConfig::from_json(lint_staged));
            }
        }
    }

    // Check .lintstagedrc
    if let Ok(content) = fs::read_to_string(".lintstagedrc") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            return Some(LintStagedConfig::from_json(&json));
        }
    }

    // Check .lintstagedrc.json
    if let Ok(content) = fs::read_to_string(".lintstagedrc.json") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            return Some(LintStagedConfig::from_json(&json));
        }
    }

    None
}

/// Lint-staged configuration structure
struct LintStagedConfig {
    patterns: HashMap<String, Vec<String>>,
}

impl LintStagedConfig {
    fn from_json(json: &serde_json::Value) -> Self {
        let mut patterns = HashMap::new();

        if let Some(obj) = json.as_object() {
            for (pattern, commands) in obj {
                let cmds: Vec<String> = match commands {
                    serde_json::Value::String(s) => vec![s.clone()],
                    serde_json::Value::Array(arr) => arr
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect(),
                    _ => vec![],
                };
                patterns.insert(pattern.clone(), cmds);
            }
        }

        Self { patterns }
    }
}

/// Migrate lint-staged configuration
fn migrate_lint_staged(lint_staged: &LintStagedConfig, config: &mut Config) -> Result<()> {
    let hook = config.hooks.entry("pre-commit".to_string()).or_default();

    for (pattern, commands) in &lint_staged.patterns {
        for cmd in commands {
            hook.tasks.push(Task {
                name: extract_task_name(cmd),
                run: cmd.clone(),
                glob: Some(pattern.clone()),
                staged: true,
                cwd: None,
                env: HashMap::new(),
                allow_failure: false,
                condition: None,
                depends_on: Vec::new(),
            });

            println!(
                "  {} Migrated: {} → {}",
                "→".dimmed(),
                pattern.dimmed(),
                cmd.cyan()
            );
        }
    }

    Ok(())
}

/// Extract a task name from a command
fn extract_task_name(cmd: &str) -> String {
    cmd.split_whitespace().take(2).collect::<Vec<_>>().join(" ")
}
