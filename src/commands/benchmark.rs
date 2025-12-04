//! Benchmark FastHooks performance

use anyhow::Result;
use colored::Colorize;
use std::process::Command;
use std::time::Instant;

/// Run performance benchmark
pub fn run() -> Result<()> {
    println!("{}", "FastHooks Performance Benchmark".bold());
    println!("{}", "═".repeat(50));
    println!();

    // Benchmark FastHooks startup time
    let fasthooks_startup = benchmark_startup("fasthooks", &["--version"]);

    // Try to benchmark Husky/npx startup for comparison
    let husky_startup = benchmark_startup("npx", &["husky", "--version"]);

    // Display results
    println!("{}", "Startup Time Comparison".cyan().bold());
    println!();

    println!(
        "  {} FastHooks: {}",
        "⚡".yellow(),
        format_duration(fasthooks_startup)
    );

    if let Some(husky_time) = husky_startup {
        println!(
            "  {} Husky (npx): {}",
            "🐢".dimmed(),
            format_duration(Some(husky_time))
        );

        if let Some(fast_time) = fasthooks_startup {
            let speedup = husky_time as f64 / fast_time as f64;
            println!();
            println!("  {} FastHooks is {:.1}x faster!", "🚀".green(), speedup);

            // Calculate environmental impact
            let time_saved_ms = husky_time.saturating_sub(fast_time);
            let commits_per_day = 10;
            let developers = 100;
            let daily_savings_ms = time_saved_ms * commits_per_day * developers;
            let yearly_savings_hours = (daily_savings_ms as f64 * 365.0) / 3_600_000.0;

            println!();
            println!("{}", "Environmental Impact (estimated)".cyan().bold());
            println!();
            println!(
                "  For a team of {} developers making {} commits/day:",
                developers, commits_per_day
            );
            println!(
                "    • {} saved per commit",
                format!("{}ms", time_saved_ms).green()
            );
            println!(
                "    • {} CPU hours saved per year",
                format!("{:.0}", yearly_savings_hours).green()
            );

            // CO2 calculation (assuming 65W CPU, 475g CO2/kWh global average)
            let kwh_saved = (65.0 * yearly_savings_hours) / 1000.0;
            let co2_kg = kwh_saved * 0.475;
            println!(
                "    • {} of CO₂ emissions avoided",
                format!("{:.1}kg", co2_kg).green()
            );
        }
    } else {
        println!(
            "  {} Husky not found (npx husky --version failed)",
            "ℹ".dimmed()
        );
        println!("    Install Husky to compare: npm install -D husky");
    }

    println!();
    println!("{}", "Hook Execution Benchmark".cyan().bold());
    println!();

    // Benchmark a simple command
    let echo_time = benchmark_command("echo", &["hello"]);
    println!(
        "  {} Simple command (echo): {}",
        "→".dimmed(),
        format_duration(echo_time)
    );

    // Benchmark npm if available
    let npm_time = benchmark_command("npm", &["--version"]);
    if npm_time.is_some() {
        println!(
            "  {} npm --version: {}",
            "→".dimmed(),
            format_duration(npm_time)
        );
    }

    // Benchmark node if available
    let node_time = benchmark_command("node", &["--version"]);
    if node_time.is_some() {
        println!(
            "  {} node --version: {}",
            "→".dimmed(),
            format_duration(node_time)
        );
    }

    println!();
    println!(
        "{}",
        "Note: Actual savings depend on your hook configuration.".dimmed()
    );
    println!(
        "{}",
        "Run 'fasthooks run pre-commit' to see real execution stats.".dimmed()
    );

    Ok(())
}

/// Benchmark the startup time of a command
fn benchmark_startup(cmd: &str, args: &[&str]) -> Option<u64> {
    benchmark_command(cmd, args)
}

/// Benchmark a command execution
fn benchmark_command(cmd: &str, args: &[&str]) -> Option<u64> {
    let iterations = 5;
    let mut times = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        let result = Command::new(cmd).args(args).output();

        if result.is_err() {
            return None;
        }

        times.push(start.elapsed().as_millis() as u64);
    }

    // Return median
    times.sort();
    Some(times[iterations / 2])
}

/// Format duration for display
fn format_duration(ms: Option<u64>) -> String {
    match ms {
        Some(ms) if ms < 1000 => format!("{}ms", ms).green().to_string(),
        Some(ms) => format!("{:.2}s", ms as f64 / 1000.0).yellow().to_string(),
        None => "N/A".dimmed().to_string(),
    }
}
