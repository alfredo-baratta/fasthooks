//! FastHooks - Blazing fast Git hooks manager
//!
//! A high-performance, Rust-based Git hooks manager designed as a drop-in
//! replacement for Husky with 27x faster execution time.

mod cli;
mod commands;
mod config;
mod hooks;
mod runner;
mod utils;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { force } => commands::init::run(force),
        Commands::Install { hook } => commands::install::run(hook),
        Commands::Uninstall => commands::uninstall::run(),
        Commands::Run { hook, files, args } => commands::run::run(hook, files, args),
        Commands::Add { hook, command } => commands::add::run(hook, command),
        Commands::List => commands::list::run(),
        Commands::Validate => commands::validate::run(),
        Commands::Migrate => commands::migrate::run(),
        Commands::Benchmark => commands::benchmark::run(),
    }
}
