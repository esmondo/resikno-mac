//! Argument definitions using clap

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Resikno-Mak: A lightweight, transparent, and safe disk cleanup tool
#[derive(Parser, Debug)]
#[command(name = "resikno")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output as JSON (for scripting)
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan disk and open interactive TUI
    Scan {
        /// Directory to scan (defaults to home directory)
        #[arg(default_value = "~")]
        path: PathBuf,

        /// Skip interactive mode, just show results
        #[arg(long)]
        no_interactive: bool,

        /// Minimum file size in MB for Downloads/Large Files (default: 0 = no minimum)
        #[arg(long, short = 'm', default_value = "0")]
        min_size: u64,

        /// Maximum file size in MB for Downloads/Large Files (default: 0 = no maximum)
        #[arg(long, short = 'M', default_value = "0")]
        max_size: u64,
    },

    /// Clean specified categories
    Clean {
        /// Category to clean: caches, logs, temp, duplicates, all
        #[arg(default_value = "all")]
        category: String,

        /// Actually perform deletion (default is dry-run)
        #[arg(long)]
        execute: bool,

        /// Only clean items marked as SAFE
        #[arg(long)]
        safe_only: bool,
        
        /// Skip confirmation prompts
        #[arg(short, long)]
        force: bool,
    },

    /// Analyze disk usage
    Analyze {
        /// Find duplicate files
        #[arg(long)]
        duplicates: bool,

        /// Find large files (over specified MB)
        #[arg(long)]
        large: Option<u64>,

        /// Find files older than specified days
        #[arg(long)]
        old: Option<u32>,
    },

    /// Manage restore points
    Restore {
        /// Restore point date (YYYY-MM-DD) or "latest"
        date: Option<String>,

        /// Restore the most recent cleanup
        #[arg(long)]
        latest: bool,

        /// List available restore points
        #[arg(long)]
        list: bool,
    },

    /// Manage configuration
    Config {
        /// Setting to modify (e.g., "retention", "safe-dirs")
        key: Option<String>,

        /// Value to set
        value: Option<String>,
    },

    /// Update resikno to the latest version
    Update {
        /// Check for updates without installing
        #[arg(long)]
        check: bool,
    },
}
