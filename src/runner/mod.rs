//! Task runner module
//!
//! Handles parallel execution of hook tasks with performance tracking.

mod executor;
mod stats;

pub use executor::TaskExecutor;
pub use stats::ExecutionStats;

/// Result of a task execution
#[derive(Debug, Clone)]
pub struct TaskResult {
    /// Task name
    pub name: String,
    /// Whether the task succeeded
    pub success: bool,
    /// Exit code (available for error handling)
    #[allow(dead_code)]
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

impl TaskResult {
    /// Create a successful task result
    pub fn success(name: String, stdout: String, stderr: String, duration_ms: u64) -> Self {
        Self {
            name,
            success: true,
            exit_code: 0,
            stdout,
            stderr,
            duration_ms,
        }
    }

    /// Create a failed task result
    pub fn failure(
        name: String,
        exit_code: i32,
        stdout: String,
        stderr: String,
        duration_ms: u64,
    ) -> Self {
        Self {
            name,
            success: false,
            exit_code,
            stdout,
            stderr,
            duration_ms,
        }
    }
}

/// Result of running all tasks in a hook
#[derive(Debug)]
pub struct HookResult {
    /// Individual task results
    pub tasks: Vec<TaskResult>,
    /// Total execution time in milliseconds (used in stats)
    #[allow(dead_code)]
    pub total_duration_ms: u64,
    /// Whether all tasks succeeded
    pub success: bool,
    /// Execution statistics
    pub stats: ExecutionStats,
}

impl HookResult {
    /// Create a new HookResult from task results
    pub fn new(tasks: Vec<TaskResult>, total_duration_ms: u64) -> Self {
        let success = tasks.iter().all(|t| t.success);
        let stats = ExecutionStats::from_tasks(&tasks, total_duration_ms);

        Self {
            tasks,
            total_duration_ms,
            success,
            stats,
        }
    }
}
