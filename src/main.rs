//! Resikno-Mac: A lightweight, transparent, and safe disk cleanup CLI tool
//!
//! # Features
//! - Interactive TUI for disk analysis
//! - Smart cleanup with safety ratings
//! - Restore points before every operation
//! - Cross-platform (macOS first, then Windows/Linux)
//!
//! # Safety
//! This tool is designed with safety as the top priority:
//! - Protected paths are NEVER deleted
//! - Dry-run is the default mode
//! - Restore points created before any deletion

mod cli;
mod cleaner;
mod platform;
mod scanner;
mod shell;
mod ui;

use anyhow::Result;
use clap::Parser;
use console::style;

fn main() {
    // Run the app and handle errors gracefully
    if let Err(err) = run() {
        print_error(&err);
        std::process::exit(1);
    }
}

/// Main application logic
fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        // No arguments: launch TUI directly (like Claude Code)
        launch_tui_direct()
    } else {
        // With arguments: traditional subcommand mode (backwards compatible)
        let cli = cli::Cli::parse();
        cli::commands::execute(cli)
    }
}

/// Launch TUI directly with a fresh scan
fn launch_tui_direct() -> Result<()> {
    use crate::platform;
    use crate::scanner;
    use crate::ui;
    
    // Run scan and go straight to TUI
    let platform = platform::current();
    let results = scanner::run_full_scan(&platform, None, 0, 0)?;
    ui::run_tui(results)
}

/// Print a user-friendly error message
fn print_error(err: &anyhow::Error) {
    eprintln!("{} {}", style("Error:").red().bold(), err);

    // Print the error chain for debugging
    for cause in err.chain().skip(1) {
        eprintln!("  {} {}", style("Caused by:").yellow(), cause);
    }

    // Provide helpful hints based on common errors
    let err_str = err.to_string().to_lowercase();

    if err_str.contains("permission denied") {
        eprintln!();
        eprintln!(
            "{} Try running with sudo or check file permissions.",
            style("Hint:").cyan().bold()
        );
    } else if err_str.contains("protected") {
        eprintln!();
        eprintln!(
            "{} This path is protected for your safety. \
             Use 'resikno analyze' to see what can be cleaned.",
            style("Hint:").cyan().bold()
        );
    } else if err_str.contains("not found") || err_str.contains("does not exist") {
        eprintln!();
        eprintln!(
            "{} The specified path doesn't exist. Check the path and try again.",
            style("Hint:").cyan().bold()
        );
    }
}

// ============================================================================
// Integration tests for safety-critical functionality
// ============================================================================

#[cfg(test)]
mod tests {
    use crate::scanner::{is_protected_path, requires_caution};
    use std::path::PathBuf;

    /// CRITICAL TEST: Ensure Documents are ALWAYS protected
    #[test]
    fn test_documents_always_protected() {
        let paths = [
            "/Users/mondo/Documents",
            "/Users/test/Documents/important.txt",
            "~/Documents/work",
        ];
        for path in paths {
            assert!(
                is_protected_path(&PathBuf::from(path)),
                "SAFETY FAILURE: {} should be protected!",
                path
            );
        }
    }

    /// CRITICAL TEST: Ensure SSH keys are ALWAYS protected
    #[test]
    fn test_ssh_always_protected() {
        let paths = [
            "/Users/mondo/.ssh",
            "/Users/test/.ssh/id_rsa",
            "~/.ssh/config",
        ];
        for path in paths {
            assert!(
                is_protected_path(&PathBuf::from(path)),
                "SAFETY FAILURE: {} should be protected!",
                path
            );
        }
    }

    /// CRITICAL TEST: Ensure System directories are ALWAYS protected
    #[test]
    fn test_system_always_protected() {
        let paths = [
            "/System",
            "/System/Library",
            "/usr/bin",
            "/bin/bash",
            "/sbin/mount",
        ];
        for path in paths {
            assert!(
                is_protected_path(&PathBuf::from(path)),
                "SAFETY FAILURE: {} should be protected!",
                path
            );
        }
    }

    /// TEST: Caches should NOT be protected (they're safe to clean)
    #[test]
    fn test_caches_not_protected() {
        let paths = [
            "/Users/mondo/Library/Caches",
            "/Users/test/Library/Caches/com.apple.Safari",
            "~/Library/Caches/Slack",
        ];
        for path in paths {
            assert!(
                !is_protected_path(&PathBuf::from(path)),
                "Cache path {} should NOT be protected",
                path
            );
        }
    }

    /// TEST: iOS backups should require caution
    #[test]
    fn test_ios_backups_require_caution() {
        let path = PathBuf::from("/Users/mondo/Library/Application Support/MobileSync/Backup");
        assert!(
            requires_caution(&path),
            "iOS backups should require caution"
        );
    }
}
