//! Task execution engine with parallel support, conditions, dependencies, and glob patterns

use super::{HookResult, TaskResult};
use crate::config::{Hook, Settings, Task};
use crate::hooks::GitRepository;
use anyhow::{Context, Result};
use glob::Pattern;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::process::Command;
use tokio::sync::{Mutex, Semaphore};

/// Executes hook tasks with parallel support
pub struct TaskExecutor {
    settings: Settings,
    staged_files: Vec<PathBuf>,
    current_branch: Option<String>,
    hook_args: Vec<String>,
}

impl TaskExecutor {
    /// Create a new TaskExecutor
    pub fn new(settings: Settings) -> Result<Self> {
        let repo = GitRepository::discover()?;
        let staged_files = repo.staged_files().unwrap_or_default();
        let current_branch = repo.current_branch().unwrap_or(None);

        Ok(Self {
            settings,
            staged_files,
            current_branch,
            hook_args: Vec::new(),
        })
    }

    /// Create a TaskExecutor with specific files (for manual runs)
    pub fn with_files(settings: Settings, files: Vec<PathBuf>) -> Result<Self> {
        let repo = GitRepository::discover().ok();
        let current_branch = repo.and_then(|r| r.current_branch().ok()).flatten();

        Ok(Self {
            settings,
            staged_files: files,
            current_branch,
            hook_args: Vec::new(),
        })
    }

    /// Set hook arguments (passed from git)
    pub fn with_hook_args(mut self, args: Vec<String>) -> Self {
        self.hook_args = args;
        self
    }

    /// Execute all tasks in a hook
    pub async fn execute_hook(&self, hook: &Hook) -> Result<HookResult> {
        let start = Instant::now();
        let parallel = hook.parallel.unwrap_or(self.settings.parallel);
        let fail_fast = hook.fail_fast.unwrap_or(self.settings.fail_fast);

        // Sort tasks by dependencies (topological sort)
        let sorted_tasks = self.sort_tasks_by_dependencies(&hook.tasks)?;

        // Filter tasks by conditions
        let executable_tasks: Vec<&Task> = sorted_tasks
            .into_iter()
            .filter(|t| self.evaluate_condition(t))
            .collect();

        let results = if parallel && !self.has_dependencies(&executable_tasks) {
            self.execute_parallel(&executable_tasks, fail_fast).await?
        } else {
            self.execute_with_dependencies(&executable_tasks, fail_fast, parallel)
                .await?
        };

        let total_duration = start.elapsed().as_millis() as u64;
        Ok(HookResult::new(results, total_duration))
    }

    /// Check if any task has dependencies
    fn has_dependencies(&self, tasks: &[&Task]) -> bool {
        tasks.iter().any(|t| !t.depends_on.is_empty())
    }

    /// Sort tasks by dependencies using topological sort
    fn sort_tasks_by_dependencies<'a>(&self, tasks: &'a [Task]) -> Result<Vec<&'a Task>> {
        let task_map: HashMap<&str, &Task> = tasks.iter().map(|t| (t.name.as_str(), t)).collect();

        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();

        // Initialize
        for task in tasks {
            in_degree.entry(task.name.as_str()).or_insert(0);
            graph.entry(task.name.as_str()).or_default();
        }

        // Build graph
        for task in tasks {
            for dep in &task.depends_on {
                if task_map.contains_key(dep.as_str()) {
                    graph.entry(dep.as_str()).or_default().push(&task.name);
                    *in_degree.entry(task.name.as_str()).or_insert(0) += 1;
                }
            }
        }

        // Kahn's algorithm
        let mut queue: Vec<&str> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(&name, _)| name)
            .collect();

        let mut sorted = Vec::new();

        while let Some(node) = queue.pop() {
            if let Some(task) = task_map.get(node) {
                sorted.push(*task);
            }

            if let Some(neighbors) = graph.get(node) {
                for &neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(neighbor);
                        }
                    }
                }
            }
        }

        // Check for cycles
        if sorted.len() != tasks.len() {
            anyhow::bail!("Circular dependency detected in tasks");
        }

        Ok(sorted)
    }

    /// Evaluate task condition
    fn evaluate_condition(&self, task: &Task) -> bool {
        let Some(condition) = &task.condition else {
            return true;
        };

        let condition = condition.trim();

        // Handle environment variable checks: "env:VAR_NAME"
        if let Some(var_name) = condition.strip_prefix("env:") {
            return std::env::var(var_name.trim()).is_ok();
        }

        // Handle negated environment variable checks: "!env:VAR_NAME"
        if let Some(var_name) = condition.strip_prefix("!env:") {
            return std::env::var(var_name.trim()).is_err();
        }

        // Handle branch conditions
        if condition.contains("branch") {
            return self.evaluate_branch_condition(condition);
        }

        // Handle file existence: "exists:path/to/file"
        if let Some(path) = condition.strip_prefix("exists:") {
            return std::path::Path::new(path.trim()).exists();
        }

        // Handle negated file existence: "!exists:path/to/file"
        if let Some(path) = condition.strip_prefix("!exists:") {
            return !std::path::Path::new(path.trim()).exists();
        }

        // Unknown condition format - default to true
        tracing::warn!("Unknown condition format: {}", condition);
        true
    }

    /// Evaluate branch-based conditions
    fn evaluate_branch_condition(&self, condition: &str) -> bool {
        let branch = self.current_branch.as_deref().unwrap_or("");

        // Parse condition: "branch == main" or "branch != main"
        if let Some(rest) = condition.strip_prefix("branch") {
            let rest = rest.trim();

            if let Some(expected) = rest.strip_prefix("==") {
                return branch == expected.trim();
            }

            if let Some(expected) = rest.strip_prefix("!=") {
                return branch != expected.trim();
            }

            // Handle "branch =~ pattern" for regex matching
            if let Some(pattern) = rest.strip_prefix("=~") {
                if let Ok(re) = regex::Regex::new(pattern.trim()) {
                    return re.is_match(branch);
                }
            }
        }

        true
    }

    /// Execute tasks sequentially
    async fn execute_sequential(
        &self,
        tasks: &[&Task],
        fail_fast: bool,
    ) -> Result<Vec<TaskResult>> {
        let mut results = Vec::with_capacity(tasks.len());

        for task in tasks {
            let files = self.filter_files(task);

            // Skip if no matching files and glob is specified
            if task.glob.is_some() && files.is_empty() {
                continue;
            }

            let result = self.execute_task(task, &files).await?;
            let failed = !result.success;
            results.push(result);

            if failed && fail_fast && !task.allow_failure {
                break;
            }
        }

        Ok(results)
    }

    /// Execute tasks with dependencies (respects dependency order, parallelizes where possible)
    async fn execute_with_dependencies(
        &self,
        tasks: &[&Task],
        fail_fast: bool,
        parallel: bool,
    ) -> Result<Vec<TaskResult>> {
        if !parallel {
            return self.execute_sequential(tasks, fail_fast).await;
        }

        let max_parallel = if self.settings.max_parallel == 0 {
            num_cpus::get()
        } else {
            self.settings.max_parallel
        };

        let semaphore = Arc::new(Semaphore::new(max_parallel));
        let completed: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
        let results: Arc<Mutex<Vec<TaskResult>>> = Arc::new(Mutex::new(Vec::new()));
        let failed = Arc::new(AtomicBool::new(false));

        // Create a map for quick task lookup
        let task_map: HashMap<&str, &Task> = tasks.iter().map(|t| (t.name.as_str(), *t)).collect();

        // Process tasks
        for task in tasks {
            // Check fail_fast
            if fail_fast && failed.load(Ordering::SeqCst) {
                break;
            }

            // Wait for dependencies
            loop {
                let completed_guard = completed.lock().await;
                let deps_satisfied = task.depends_on.iter().all(|dep| {
                    completed_guard.contains(dep) || !task_map.contains_key(dep.as_str())
                });
                drop(completed_guard);

                if deps_satisfied {
                    break;
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

                // Check if we should abort due to fail_fast
                if fail_fast && failed.load(Ordering::SeqCst) {
                    break;
                }
            }

            if fail_fast && failed.load(Ordering::SeqCst) {
                break;
            }

            let files = self.filter_files(task);

            // Skip if no matching files and glob is specified
            if task.glob.is_some() && files.is_empty() {
                completed.lock().await.insert(task.name.clone());
                continue;
            }

            let permit = semaphore.clone().acquire_owned().await?;
            let task_clone = (*task).clone();
            let files_clone = files.clone();
            let completed_clone = completed.clone();
            let results_clone = results.clone();
            let failed_clone = failed.clone();
            let settings_clone = self.settings.clone();
            let hook_args_clone = self.hook_args.clone();

            tokio::spawn(async move {
                let executor = TaskExecutor {
                    settings: settings_clone,
                    staged_files: files_clone.clone(),
                    current_branch: None,
                    hook_args: hook_args_clone,
                };

                let result = executor.execute_task(&task_clone, &files_clone).await;
                drop(permit);

                if let Ok(res) = result {
                    if !res.success && !task_clone.allow_failure {
                        failed_clone.store(true, Ordering::SeqCst);
                    }
                    results_clone.lock().await.push(res);
                }

                completed_clone.lock().await.insert(task_clone.name.clone());
            });
        }

        // Wait for all tasks to complete
        loop {
            let completed_count = completed.lock().await.len();
            let expected = tasks
                .iter()
                .filter(|t| {
                    if t.glob.is_some() {
                        !self.filter_files(t).is_empty()
                    } else {
                        true
                    }
                })
                .count();

            if completed_count >= expected || (fail_fast && failed.load(Ordering::SeqCst)) {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        let final_results = Arc::try_unwrap(results)
            .map_err(|_| anyhow::anyhow!("Failed to unwrap results"))?
            .into_inner();

        Ok(final_results)
    }

    /// Execute tasks in parallel (no dependencies)
    async fn execute_parallel(&self, tasks: &[&Task], fail_fast: bool) -> Result<Vec<TaskResult>> {
        let max_parallel = if self.settings.max_parallel == 0 {
            num_cpus::get()
        } else {
            self.settings.max_parallel
        };

        let semaphore = Arc::new(Semaphore::new(max_parallel));
        let failed = Arc::new(AtomicBool::new(false));
        let mut handles = Vec::with_capacity(tasks.len());

        for task in tasks {
            let files = self.filter_files(task);

            // Skip if no matching files and glob is specified
            if task.glob.is_some() && files.is_empty() {
                continue;
            }

            // Check fail_fast before spawning new tasks
            if fail_fast && failed.load(Ordering::SeqCst) {
                break;
            }

            let permit = semaphore.clone().acquire_owned().await?;
            let task_clone = (*task).clone();
            let files_clone = files.clone();
            let failed_clone = failed.clone();
            let settings_clone = self.settings.clone();
            let hook_args_clone = self.hook_args.clone();

            let handle = tokio::spawn(async move {
                let executor = TaskExecutor {
                    settings: settings_clone,
                    staged_files: files_clone.clone(),
                    current_branch: None,
                    hook_args: hook_args_clone,
                };

                let result = executor.execute_task(&task_clone, &files_clone).await;
                drop(permit);

                if let Ok(ref res) = result {
                    if !res.success && !task_clone.allow_failure {
                        failed_clone.store(true, Ordering::SeqCst);
                    }
                }

                result
            });

            handles.push(handle);
        }

        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(anyhow::anyhow!("Task panicked: {}", e)),
            }
        }

        Ok(results)
    }

    /// Execute a single task
    async fn execute_task(&self, task: &Task, files: &[PathBuf]) -> Result<TaskResult> {
        let start = Instant::now();

        // Build the command
        let command = self.build_command(task, files);

        let output = Command::new(self.get_shell())
            .arg(self.get_shell_arg())
            .arg(&command)
            .current_dir(task.cwd.as_deref().unwrap_or("."))
            .envs(&task.env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .with_context(|| format!("Failed to execute task: {}", task.name))?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(TaskResult::success(
                task.name.clone(),
                stdout,
                stderr,
                duration_ms,
            ))
        } else {
            let exit_code = output.status.code().unwrap_or(-1);
            Ok(TaskResult::failure(
                task.name.clone(),
                exit_code,
                stdout,
                stderr,
                duration_ms,
            ))
        }
    }

    /// Build the command string with file and argument substitution
    fn build_command(&self, task: &Task, files: &[PathBuf]) -> String {
        let files_str: String = files
            .iter()
            .map(|f| {
                let path = f.to_string_lossy();
                if path.contains(' ') {
                    format!("\"{}\"", path)
                } else {
                    path.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let mut command = task.run.clone();

        // Replace {files} placeholder with actual files
        if command.contains("{files}") {
            command = command.replace("{files}", &files_str);
        } else if task.glob.is_some() && !files.is_empty() {
            // Append files to command if glob is specified
            command = format!("{} {}", command, files_str);
        }

        // Replace hook argument placeholders: $1, $2, $3, etc.
        for (i, arg) in self.hook_args.iter().enumerate() {
            let placeholder = format!("${}", i + 1);
            command = command.replace(&placeholder, arg);
        }

        // Also support {1}, {2}, {3} style placeholders
        for (i, arg) in self.hook_args.iter().enumerate() {
            let placeholder = format!("{{{}}}", i + 1);
            command = command.replace(&placeholder, arg);
        }

        command
    }

    /// Filter staged files based on task glob pattern (supports negation with !)
    fn filter_files(&self, task: &Task) -> Vec<PathBuf> {
        let Some(glob_pattern) = &task.glob else {
            return Vec::new();
        };

        // Parse multiple patterns (comma or space separated)
        let patterns: Vec<&str> = glob_pattern
            .split([',', ' '])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let mut include_patterns: Vec<Pattern> = Vec::new();
        let mut exclude_patterns: Vec<Pattern> = Vec::new();

        for pat in patterns {
            if let Some(negated) = pat.strip_prefix('!') {
                if let Ok(p) = Pattern::new(negated) {
                    exclude_patterns.push(p);
                }
            } else if let Ok(p) = Pattern::new(pat) {
                include_patterns.push(p);
            }
        }

        // If no include patterns, nothing matches
        if include_patterns.is_empty() {
            return Vec::new();
        }

        self.staged_files
            .iter()
            .filter(|f| {
                let path_str = f.to_string_lossy();
                let filename = f.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Check if file matches any include pattern
                let included = include_patterns.iter().any(|p| {
                    p.matches(&path_str)
                        || p.matches(filename)
                        || p.matches(&path_str.replace('\\', "/"))
                });

                if !included {
                    return false;
                }

                // Check if file matches any exclude pattern
                let excluded = exclude_patterns.iter().any(|p| {
                    p.matches(&path_str)
                        || p.matches(filename)
                        || p.matches(&path_str.replace('\\', "/"))
                });

                !excluded
            })
            .cloned()
            .collect()
    }

    /// Get the appropriate shell for the current platform
    fn get_shell(&self) -> &'static str {
        if cfg!(windows) {
            "cmd"
        } else {
            "sh"
        }
    }

    /// Get the shell argument for command execution
    fn get_shell_arg(&self) -> &'static str {
        if cfg!(windows) {
            "/C"
        } else {
            "-c"
        }
    }
}

/// CPU count detection
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_executor() -> TaskExecutor {
        TaskExecutor {
            settings: Settings::default(),
            staged_files: vec![
                PathBuf::from("src/main.rs"),
                PathBuf::from("src/lib.rs"),
                PathBuf::from("tests/test.rs"),
                PathBuf::from("README.md"),
                PathBuf::from("src/utils/helper.ts"),
                PathBuf::from("src/components/Button.tsx"),
            ],
            current_branch: Some("main".to_string()),
            hook_args: vec!["arg1".to_string(), "arg2".to_string()],
        }
    }

    #[test]
    fn test_filter_files_simple_glob() {
        let executor = create_test_executor();
        let task = Task {
            name: "test".to_string(),
            run: "echo".to_string(),
            glob: Some("*.rs".to_string()),
            staged: true,
            cwd: None,
            env: HashMap::new(),
            allow_failure: false,
            condition: None,
            depends_on: vec![],
        };

        let files = executor.filter_files(&task);
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_filter_files_with_exclusion() {
        let executor = create_test_executor();
        let task = Task {
            name: "test".to_string(),
            run: "echo".to_string(),
            glob: Some("*.rs, !tests/*.rs".to_string()),
            staged: true,
            cwd: None,
            env: HashMap::new(),
            allow_failure: false,
            condition: None,
            depends_on: vec![],
        };

        let files = executor.filter_files(&task);
        assert_eq!(files.len(), 2); // main.rs, lib.rs (excluding tests/test.rs)
    }

    #[test]
    fn test_filter_files_multiple_extensions() {
        let executor = create_test_executor();
        let task = Task {
            name: "test".to_string(),
            run: "echo".to_string(),
            glob: Some("*.ts, *.tsx".to_string()),
            staged: true,
            cwd: None,
            env: HashMap::new(),
            allow_failure: false,
            condition: None,
            depends_on: vec![],
        };

        let files = executor.filter_files(&task);
        assert_eq!(files.len(), 2); // helper.ts, Button.tsx
    }

    #[test]
    fn test_evaluate_condition_branch_equals() {
        let executor = create_test_executor();
        let mut task = Task {
            name: "test".to_string(),
            run: "echo".to_string(),
            glob: None,
            staged: true,
            cwd: None,
            env: HashMap::new(),
            allow_failure: false,
            condition: Some("branch == main".to_string()),
            depends_on: vec![],
        };

        assert!(executor.evaluate_condition(&task));

        task.condition = Some("branch == develop".to_string());
        assert!(!executor.evaluate_condition(&task));
    }

    #[test]
    fn test_evaluate_condition_branch_not_equals() {
        let executor = create_test_executor();
        let mut task = Task {
            name: "test".to_string(),
            run: "echo".to_string(),
            glob: None,
            staged: true,
            cwd: None,
            env: HashMap::new(),
            allow_failure: false,
            condition: Some("branch != main".to_string()),
            depends_on: vec![],
        };

        assert!(!executor.evaluate_condition(&task));

        task.condition = Some("branch != develop".to_string());
        assert!(executor.evaluate_condition(&task));
    }

    #[test]
    fn test_evaluate_condition_env_var() {
        let executor = create_test_executor();
        let task = Task {
            name: "test".to_string(),
            run: "echo".to_string(),
            glob: None,
            staged: true,
            cwd: None,
            env: HashMap::new(),
            allow_failure: false,
            condition: Some("env:PATH".to_string()),
            depends_on: vec![],
        };

        assert!(executor.evaluate_condition(&task)); // PATH should exist
    }

    #[test]
    fn test_build_command_with_hook_args() {
        let executor = create_test_executor();
        let task = Task {
            name: "test".to_string(),
            run: "commitlint --edit $1".to_string(),
            glob: None,
            staged: true,
            cwd: None,
            env: HashMap::new(),
            allow_failure: false,
            condition: None,
            depends_on: vec![],
        };

        let command = executor.build_command(&task, &[]);
        assert_eq!(command, "commitlint --edit arg1");
    }

    #[test]
    fn test_build_command_with_files() {
        let executor = create_test_executor();
        let task = Task {
            name: "test".to_string(),
            run: "eslint {files}".to_string(),
            glob: Some("*.rs".to_string()),
            staged: true,
            cwd: None,
            env: HashMap::new(),
            allow_failure: false,
            condition: None,
            depends_on: vec![],
        };

        let files = vec![PathBuf::from("src/main.rs"), PathBuf::from("src/lib.rs")];
        let command = executor.build_command(&task, &files);
        assert!(command.contains("src/main.rs"));
        assert!(command.contains("src/lib.rs"));
    }

    #[test]
    fn test_sort_tasks_by_dependencies() {
        let executor = create_test_executor();
        let tasks = vec![
            Task {
                name: "test".to_string(),
                run: "cargo test".to_string(),
                glob: None,
                staged: true,
                cwd: None,
                env: HashMap::new(),
                allow_failure: false,
                condition: None,
                depends_on: vec!["lint".to_string()],
            },
            Task {
                name: "lint".to_string(),
                run: "cargo clippy".to_string(),
                glob: None,
                staged: true,
                cwd: None,
                env: HashMap::new(),
                allow_failure: false,
                condition: None,
                depends_on: vec![],
            },
        ];

        let sorted = executor.sort_tasks_by_dependencies(&tasks).unwrap();
        assert_eq!(sorted[0].name, "lint");
        assert_eq!(sorted[1].name, "test");
    }

    #[test]
    fn test_circular_dependency_detection() {
        let executor = create_test_executor();
        let tasks = vec![
            Task {
                name: "a".to_string(),
                run: "echo a".to_string(),
                glob: None,
                staged: true,
                cwd: None,
                env: HashMap::new(),
                allow_failure: false,
                condition: None,
                depends_on: vec!["b".to_string()],
            },
            Task {
                name: "b".to_string(),
                run: "echo b".to_string(),
                glob: None,
                staged: true,
                cwd: None,
                env: HashMap::new(),
                allow_failure: false,
                condition: None,
                depends_on: vec!["a".to_string()],
            },
        ];

        let result = executor.sort_tasks_by_dependencies(&tasks);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circular dependency"));
    }
}
