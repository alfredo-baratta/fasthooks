//! Configuration file parser with detailed error reporting

use super::schema::{Config, Hook};
use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Parser for FastHooks configuration files
pub struct ConfigParser;

/// Validation error with context
#[derive(Debug)]
pub struct ValidationError {
    pub message: String,
    pub location: Option<String>,
    pub suggestion: Option<String>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(loc) = &self.location {
            write!(f, " (at {})", loc)?;
        }
        if let Some(sug) = &self.suggestion {
            write!(f, "\n  Suggestion: {}", sug)?;
        }
        Ok(())
    }
}

impl ConfigParser {
    /// Parse a TOML configuration file with detailed error messages
    pub fn parse_file(path: &Path) -> Result<Config> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        Self::parse_toml(&content).with_context(|| format!("Failed to parse {}", path.display()))
    }

    /// Parse TOML content into Config with detailed error messages
    pub fn parse_toml(content: &str) -> Result<Config> {
        match toml::from_str::<Config>(content) {
            Ok(config) => Ok(config),
            Err(e) => {
                let error_msg = Self::format_toml_error(&e, content);
                anyhow::bail!("{}", error_msg)
            }
        }
    }

    /// Format TOML parsing errors with line numbers and context
    fn format_toml_error(error: &toml::de::Error, content: &str) -> String {
        let mut msg = String::new();

        msg.push_str(&format!("{}\n", "Configuration Error".red().bold()));

        // Get span information if available
        if let Some(span) = error.span() {
            let lines: Vec<&str> = content.lines().collect();

            // Calculate line and column from byte offset
            let mut current_pos = 0;
            let mut line_num = 1;
            let mut col_num = 1;

            for (i, line) in lines.iter().enumerate() {
                let line_end = current_pos + line.len() + 1; // +1 for newline
                if span.start < line_end {
                    line_num = i + 1;
                    col_num = span.start - current_pos + 1;
                    break;
                }
                current_pos = line_end;
            }

            msg.push_str(&format!(
                "  {} Line {}, Column {}\n\n",
                "→".cyan(),
                line_num,
                col_num
            ));

            // Show context lines
            let start_line = line_num.saturating_sub(2);
            let end_line = (line_num + 2).min(lines.len());

            for i in start_line..end_line {
                let line = lines.get(i).unwrap_or(&"");
                let line_marker = if i + 1 == line_num {
                    format!("{:>4} │ ", (i + 1).to_string().red().bold())
                } else {
                    format!("{:>4} │ ", i + 1)
                };

                msg.push_str(&line_marker);
                msg.push_str(line);
                msg.push('\n');

                // Show error pointer
                if i + 1 == line_num {
                    let pointer = format!(
                        "     │ {}{}",
                        " ".repeat(col_num.saturating_sub(1)),
                        "^".red().bold()
                    );
                    msg.push_str(&pointer);
                    msg.push('\n');
                }
            }

            msg.push('\n');
        }

        // Add the actual error message
        let error_str = error.message();
        msg.push_str(&format!("  {}: {}\n", "Error".red().bold(), error_str));

        // Add helpful suggestions based on common errors
        if let Some(suggestion) = Self::suggest_fix(error_str) {
            msg.push_str(&format!("  {}: {}\n", "Hint".yellow().bold(), suggestion));
        }

        msg
    }

    /// Provide suggestions for common configuration errors
    fn suggest_fix(error_msg: &str) -> Option<String> {
        let error_lower = error_msg.to_lowercase();

        if error_lower.contains("missing field") {
            if error_lower.contains("name") {
                return Some(
                    "Each task must have a 'name' field. Example: name = \"lint\"".to_string(),
                );
            }
            if error_lower.contains("run") {
                return Some(
                    "Each task must have a 'run' field. Example: run = \"npm run lint\""
                        .to_string(),
                );
            }
        }

        if error_lower.contains("expected") && error_lower.contains("string") {
            return Some("String values must be quoted. Example: name = \"my-task\"".to_string());
        }

        if error_lower.contains("expected") && error_lower.contains("boolean") {
            return Some("Boolean values should be 'true' or 'false' (without quotes)".to_string());
        }

        if error_lower.contains("expected") && error_lower.contains("array") {
            return Some(
                "Array values use square brackets. Example: depends_on = [\"task1\", \"task2\"]"
                    .to_string(),
            );
        }

        if error_lower.contains("unknown field") {
            return Some("Check the field name for typos. Valid task fields: name, run, glob, staged, cwd, env, allow_failure, if, depends_on".to_string());
        }

        if error_lower.contains("duplicate key") {
            return Some(
                "Each key can only appear once in a section. Remove the duplicate.".to_string(),
            );
        }

        None
    }

    /// Validate a configuration and return detailed errors
    pub fn validate(config: &Config) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate version
        if config.version.is_empty() {
            errors.push(ValidationError {
                message: "Configuration version is missing".to_string(),
                location: Some("version".to_string()),
                suggestion: Some("Add: version = \"1\"".to_string()),
            });
        }

        // Validate hooks
        for (hook_name, hook) in &config.hooks {
            Self::validate_hook(hook_name, hook, &mut errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate a single hook
    fn validate_hook(hook_name: &str, hook: &Hook, errors: &mut Vec<ValidationError>) {
        if hook.tasks.is_empty() {
            errors.push(ValidationError {
                message: format!("Hook '{}' has no tasks defined", hook_name),
                location: Some(format!("hooks.{}", hook_name)),
                suggestion: Some("Add at least one task with [[hooks.<name>.tasks]]".to_string()),
            });
            return;
        }

        let mut task_names: HashSet<&str> = HashSet::new();

        for (i, task) in hook.tasks.iter().enumerate() {
            let task_loc = format!("hooks.{}.tasks[{}]", hook_name, i);

            // Check for empty task name
            if task.name.trim().is_empty() {
                errors.push(ValidationError {
                    message: "Task name cannot be empty".to_string(),
                    location: Some(task_loc.clone()),
                    suggestion: Some("Add a descriptive name for the task".to_string()),
                });
            }

            // Check for duplicate task names
            if !task_names.insert(&task.name) {
                errors.push(ValidationError {
                    message: format!("Duplicate task name: '{}'", task.name),
                    location: Some(task_loc.clone()),
                    suggestion: Some("Each task must have a unique name within a hook".to_string()),
                });
            }

            // Check for empty run command
            if task.run.trim().is_empty() {
                errors.push(ValidationError {
                    message: format!("Task '{}' has no command", task.name),
                    location: Some(task_loc.clone()),
                    suggestion: Some("Add a 'run' field with the command to execute".to_string()),
                });
            }

            // Validate dependencies exist
            for dep in &task.depends_on {
                if !hook.tasks.iter().any(|t| &t.name == dep) {
                    errors.push(ValidationError {
                        message: format!(
                            "Task '{}' depends on '{}' which doesn't exist",
                            task.name, dep
                        ),
                        location: Some(task_loc.clone()),
                        suggestion: Some(format!(
                            "Either add a task named '{}' or remove it from depends_on",
                            dep
                        )),
                    });
                }
            }

            // Validate glob pattern syntax
            if let Some(glob) = &task.glob {
                Self::validate_glob_pattern(glob, &task.name, &task_loc, errors);
            }

            // Validate condition syntax
            if let Some(condition) = &task.condition {
                Self::validate_condition(condition, &task.name, &task_loc, errors);
            }
        }
    }

    /// Validate glob pattern syntax
    fn validate_glob_pattern(
        pattern: &str,
        task_name: &str,
        location: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        let patterns: Vec<&str> = pattern
            .split([',', ' '])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for pat in patterns {
            let pat_to_check = pat.strip_prefix('!').unwrap_or(pat);

            if glob::Pattern::new(pat_to_check).is_err() {
                errors.push(ValidationError {
                    message: format!("Invalid glob pattern '{}' in task '{}'", pat, task_name),
                    location: Some(location.to_string()),
                    suggestion: Some("Valid patterns: *.js, src/**/*.ts, !*.test.js".to_string()),
                });
            }
        }
    }

    /// Validate condition syntax
    fn validate_condition(
        condition: &str,
        task_name: &str,
        location: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        let condition = condition.trim();

        let valid_prefixes = [
            "branch ==",
            "branch !=",
            "branch =~",
            "env:",
            "!env:",
            "exists:",
            "!exists:",
        ];

        let is_valid = valid_prefixes.iter().any(|p| condition.starts_with(p));

        if !is_valid {
            errors.push(ValidationError {
                message: format!(
                    "Unknown condition format '{}' in task '{}'",
                    condition, task_name
                ),
                location: Some(location.to_string()),
                suggestion: Some(
                    "Valid conditions: 'branch == main', 'branch != develop', 'env:CI', '!env:CI', 'exists:file.txt'".to_string()
                ),
            });
        }

        // Validate regex if using branch =~
        if let Some(pattern) = condition.strip_prefix("branch =~") {
            if regex::Regex::new(pattern.trim()).is_err() {
                errors.push(ValidationError {
                    message: format!(
                        "Invalid regex pattern '{}' in condition for task '{}'",
                        pattern.trim(),
                        task_name
                    ),
                    location: Some(location.to_string()),
                    suggestion: Some("Check your regex syntax".to_string()),
                });
            }
        }
    }

    /// Format validation errors for display
    pub fn format_validation_errors(errors: &[ValidationError]) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "{} Found {} validation error(s):\n\n",
            "✗".red().bold(),
            errors.len()
        ));

        for (i, error) in errors.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, error.message.red()));

            if let Some(loc) = &error.location {
                output.push_str(&format!("   {} {}\n", "Location:".dimmed(), loc));
            }

            if let Some(sug) = &error.suggestion {
                output.push_str(&format!("   {} {}\n", "Suggestion:".yellow(), sug));
            }

            output.push('\n');
        }

        output
    }

    /// Serialize Config to TOML string
    pub fn to_toml(config: &Config) -> Result<String> {
        toml::to_string_pretty(config).context("Failed to serialize configuration to TOML")
    }

    /// Generate a default configuration file content
    pub fn default_config_content() -> String {
        r#"# FastHooks Configuration
# Documentation: https://github.com/alfredo-baratta/fasthooks/blob/main/docs/configuration.md

version = "1"

[settings]
# Run tasks in parallel for maximum speed
parallel = true

# Auto-detect number of parallel tasks based on CPU cores (0 = auto)
max_parallel = 0

# Show execution time statistics after each hook
show_stats = true

# Show estimated carbon savings compared to Node.js-based tools
show_carbon_savings = true

# Stop on first error
fail_fast = true

# Skip hooks when running in CI environment
skip_ci = false

# Enable colored output
colors = true

# Pre-commit hook configuration
[hooks.pre-commit]
parallel = true

[[hooks.pre-commit.tasks]]
name = "lint"
run = "npm run lint"
glob = "*.{js,ts,jsx,tsx}"
staged = true

[[hooks.pre-commit.tasks]]
name = "format"
run = "npm run format"
glob = "*.{js,ts,jsx,tsx,json,md}"
staged = true

[[hooks.pre-commit.tasks]]
name = "typecheck"
run = "npm run typecheck"

# Pre-push hook configuration
[hooks.pre-push]
parallel = false

[[hooks.pre-push.tasks]]
name = "test"
run = "npm test"

[[hooks.pre-push.tasks]]
name = "build"
run = "npm run build"
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_default_config() {
        let content = ConfigParser::default_config_content();
        let config = ConfigParser::parse_toml(&content).unwrap();

        assert_eq!(config.version, "1");
        assert!(config.settings.parallel);
        assert!(config.hooks.contains_key("pre-commit"));
        assert!(config.hooks.contains_key("pre-push"));
    }

    #[test]
    fn test_parse_minimal_config() {
        let content = r#"
version = "1"

[hooks.pre-commit]
[[hooks.pre-commit.tasks]]
name = "test"
run = "cargo test"
"#;
        let config = ConfigParser::parse_toml(content).unwrap();
        assert_eq!(config.hooks.len(), 1);
    }

    #[test]
    fn test_parse_config_with_dependencies() {
        let content = r#"
version = "1"

[hooks.pre-commit]
[[hooks.pre-commit.tasks]]
name = "lint"
run = "npm run lint"

[[hooks.pre-commit.tasks]]
name = "test"
run = "npm test"
depends_on = ["lint"]
"#;
        let config = ConfigParser::parse_toml(content).unwrap();
        let hook = config.hooks.get("pre-commit").unwrap();
        assert_eq!(hook.tasks[1].depends_on, vec!["lint"]);
    }

    #[test]
    fn test_parse_config_with_condition() {
        let content = r#"
version = "1"

[hooks.pre-commit]
[[hooks.pre-commit.tasks]]
name = "deploy"
run = "npm run deploy"
if = "branch == main"
"#;
        let config = ConfigParser::parse_toml(content).unwrap();
        let hook = config.hooks.get("pre-commit").unwrap();
        assert_eq!(hook.tasks[0].condition, Some("branch == main".to_string()));
    }

    #[test]
    fn test_validate_empty_task_name() {
        let content = r#"
version = "1"

[hooks.pre-commit]
[[hooks.pre-commit.tasks]]
name = ""
run = "npm test"
"#;
        let config = ConfigParser::parse_toml(content).unwrap();
        let result = ConfigParser::validate(&config);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("empty")));
    }

    #[test]
    fn test_validate_missing_dependency() {
        let content = r#"
version = "1"

[hooks.pre-commit]
[[hooks.pre-commit.tasks]]
name = "test"
run = "npm test"
depends_on = ["nonexistent"]
"#;
        let config = ConfigParser::parse_toml(content).unwrap();
        let result = ConfigParser::validate(&config);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("doesn't exist")));
    }

    #[test]
    fn test_validate_duplicate_task_names() {
        let content = r#"
version = "1"

[hooks.pre-commit]
[[hooks.pre-commit.tasks]]
name = "lint"
run = "npm run lint"

[[hooks.pre-commit.tasks]]
name = "lint"
run = "npm run lint:fix"
"#;
        let config = ConfigParser::parse_toml(content).unwrap();
        let result = ConfigParser::validate(&config);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("Duplicate")));
    }

    #[test]
    fn test_parse_error_formatting() {
        let content = r#"
version = "1"

[hooks.pre-commit]
[[hooks.pre-commit.tasks]]
name = test without quotes
run = "npm test"
"#;
        let result = ConfigParser::parse_toml(content);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        // Should contain helpful error information
        assert!(error.contains("Error") || error.contains("expected"));
    }
}
