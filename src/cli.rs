//! Command-line interface definition for FastHooks

use clap::{Parser, Subcommand};

/// FastHooks - Blazing fast Git hooks manager
///
/// A high-performance, Rust-based Git hooks manager designed as a drop-in
/// replacement for Husky with 27x faster execution time.
#[derive(Parser, Debug)]
#[command(name = "fasthooks")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize FastHooks in the current repository
    Init {
        /// Overwrite existing configuration
        #[arg(short, long)]
        force: bool,
    },

    /// Install Git hooks
    Install {
        /// Specific hook to install (e.g., pre-commit, pre-push)
        /// If not specified, installs all configured hooks
        #[arg(short = 'H', long)]
        hook: Option<String>,
    },

    /// Uninstall all FastHooks Git hooks
    Uninstall,

    /// Manually run a hook
    Run {
        /// Hook name to run (e.g., pre-commit)
        hook: String,

        /// Specific files to run the hook on
        #[arg(short, long)]
        files: Option<Vec<String>>,

        /// Hook arguments passed by Git (e.g., commit message file for commit-msg hook)
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Add a command to a hook
    Add {
        /// Hook name (e.g., pre-commit)
        hook: String,

        /// Command to execute
        command: String,
    },

    /// List all configured hooks
    List,

    /// Validate the configuration file
    Validate,

    /// Migrate from Husky to FastHooks
    Migrate,

    /// Run performance benchmark comparing FastHooks vs Husky
    Benchmark,
}
