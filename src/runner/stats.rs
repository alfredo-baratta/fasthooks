//! Execution statistics and carbon savings calculations

use super::TaskResult;
use colored::Colorize;

/// Execution statistics for a hook run
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    /// Total tasks executed
    pub total_tasks: usize,
    /// Successful tasks
    pub successful_tasks: usize,
    /// Failed tasks
    pub failed_tasks: usize,
    /// Total execution time (wall clock)
    pub wall_time_ms: u64,
    /// Sum of all task execution times (for parallel efficiency calc)
    #[allow(dead_code)]
    pub cpu_time_ms: u64,
    /// Time saved through parallelization
    pub parallel_savings_ms: u64,
    /// Estimated carbon savings
    pub carbon_savings: CarbonSavings,
}

impl ExecutionStats {
    /// Create stats from task results
    pub fn from_tasks(tasks: &[TaskResult], wall_time_ms: u64) -> Self {
        let total_tasks = tasks.len();
        let successful_tasks = tasks.iter().filter(|t| t.success).count();
        let failed_tasks = total_tasks - successful_tasks;
        let cpu_time_ms: u64 = tasks.iter().map(|t| t.duration_ms).sum();
        let parallel_savings_ms = cpu_time_ms.saturating_sub(wall_time_ms);

        // Calculate carbon savings compared to Node.js baseline
        let carbon_savings = CarbonSavings::calculate(wall_time_ms);

        Self {
            total_tasks,
            successful_tasks,
            failed_tasks,
            wall_time_ms,
            cpu_time_ms,
            parallel_savings_ms,
            carbon_savings,
        }
    }

    /// Format stats for display
    pub fn format(&self, show_carbon: bool) -> String {
        let mut output = String::new();

        // Task summary
        let status = if self.failed_tasks == 0 {
            format!("{} {} tasks passed", "✓".green(), self.total_tasks).green()
        } else {
            format!(
                "{} {}/{} tasks passed",
                "✗".red(),
                self.successful_tasks,
                self.total_tasks
            )
            .red()
        };

        output.push_str(&format!("\n{}\n", status));

        // Timing
        output.push_str(&format!(
            "  {} Completed in {}\n",
            "⏱".cyan(),
            Self::format_duration(self.wall_time_ms)
        ));

        // Parallel savings
        if self.parallel_savings_ms > 0 {
            output.push_str(&format!(
                "  {} Saved {} through parallelization\n",
                "⚡".yellow(),
                Self::format_duration(self.parallel_savings_ms)
            ));
        }

        // Carbon savings
        if show_carbon && self.carbon_savings.grams_co2 > 0.0 {
            output.push_str(&format!(
                "  {} Saved ~{:.2}g CO₂ vs Node.js-based tools\n",
                "🌱".green(),
                self.carbon_savings.grams_co2
            ));
        }

        output
    }

    /// Format duration in human-readable format
    fn format_duration(ms: u64) -> String {
        if ms < 1000 {
            format!("{}ms", ms)
        } else if ms < 60000 {
            format!("{:.2}s", ms as f64 / 1000.0)
        } else {
            let mins = ms / 60000;
            let secs = (ms % 60000) / 1000;
            format!("{}m {}s", mins, secs)
        }
    }
}

/// Carbon savings estimate
#[derive(Debug, Clone)]
pub struct CarbonSavings {
    /// Estimated grams of CO2 saved
    pub grams_co2: f64,
    /// Baseline comparison (Node.js estimated time)
    #[allow(dead_code)]
    pub baseline_ms: u64,
    /// Actual execution time
    #[allow(dead_code)]
    pub actual_ms: u64,
}

impl CarbonSavings {
    /// Average carbon intensity of electricity (gCO2/kWh) - global average
    const CARBON_INTENSITY: f64 = 475.0;

    /// Average CPU power consumption in watts
    const CPU_POWER_WATTS: f64 = 65.0;

    /// Estimated Node.js overhead factor (startup + runtime)
    /// Based on benchmarks: Node.js hooks typically take 3-10x longer
    const NODEJS_OVERHEAD_FACTOR: f64 = 5.0;

    /// Calculate carbon savings compared to Node.js baseline
    pub fn calculate(actual_ms: u64) -> Self {
        // Estimate what Node.js would have taken
        let baseline_ms = (actual_ms as f64 * Self::NODEJS_OVERHEAD_FACTOR) as u64;
        let time_saved_ms = baseline_ms.saturating_sub(actual_ms);

        // Convert to hours
        let time_saved_hours = time_saved_ms as f64 / 3_600_000.0;

        // Calculate energy saved (kWh)
        let energy_saved_kwh = (Self::CPU_POWER_WATTS * time_saved_hours) / 1000.0;

        // Calculate CO2 saved (grams)
        let grams_co2 = energy_saved_kwh * Self::CARBON_INTENSITY;

        Self {
            grams_co2,
            baseline_ms,
            actual_ms,
        }
    }

    /// Calculate cumulative savings (for monthly/yearly reports)
    #[allow(dead_code)]
    pub fn cumulative(runs: &[CarbonSavings]) -> Self {
        let total_grams: f64 = runs.iter().map(|r| r.grams_co2).sum();
        let total_baseline: u64 = runs.iter().map(|r| r.baseline_ms).sum();
        let total_actual: u64 = runs.iter().map(|r| r.actual_ms).sum();

        Self {
            grams_co2: total_grams,
            baseline_ms: total_baseline,
            actual_ms: total_actual,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_carbon_savings_calculation() {
        let savings = CarbonSavings::calculate(100);
        assert!(savings.grams_co2 >= 0.0);
        assert_eq!(savings.baseline_ms, 500); // 5x overhead
        assert_eq!(savings.actual_ms, 100);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(ExecutionStats::format_duration(500), "500ms");
        assert_eq!(ExecutionStats::format_duration(1500), "1.50s");
        assert_eq!(ExecutionStats::format_duration(65000), "1m 5s");
    }

    #[test]
    fn test_stats_from_tasks() {
        let tasks = vec![
            TaskResult::success("task1".to_string(), String::new(), String::new(), 100),
            TaskResult::success("task2".to_string(), String::new(), String::new(), 200),
        ];

        let stats = ExecutionStats::from_tasks(&tasks, 150);
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.successful_tasks, 2);
        assert_eq!(stats.cpu_time_ms, 300);
        assert_eq!(stats.parallel_savings_ms, 150);
    }
}
