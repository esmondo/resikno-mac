//! Command implementations

use anyhow::Result;
use bytesize::ByteSize;
use super::args::{Cli, Commands};
use crate::platform;
use crate::scanner::{self, CleanupCategory, ScanResults};

/// Execute the CLI command
pub fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Scan { path, no_interactive } => {
            let platform = platform::current();

            // Run the scan
            println!("Scanning system for cleanable files...\n");
            let scan_path = if path.to_string_lossy() == "~" {
                None
            } else {
                Some(path.as_path())
            };
            let results = scanner::run_full_scan(&platform, scan_path)?;

            // Display results (text mode for now)
            display_scan_results(&results);

            // TODO: Launch TUI if !no_interactive
            if !no_interactive {
                println!("\nInteractive TUI mode coming soon. Use --no-interactive for now.");
            }
        }
        Commands::Clean { category, execute, safe_only } => {
            println!("Cleaning: {} (execute: {}, safe_only: {})", category, execute, safe_only);
            // TODO: Implement clean
        }
        Commands::Analyze { duplicates, large, old } => {
            println!("Analyzing (duplicates: {}, large: {:?}, old: {:?})", duplicates, large, old);
            // TODO: Implement analyze
        }
        Commands::Restore { date, latest, list } => {
            if list {
                println!("Listing restore points...");
            } else if latest {
                println!("Restoring latest...");
            } else if let Some(d) = date {
                println!("Restoring: {}", d);
            }
            // TODO: Implement restore
        }
        Commands::Config { key, value } => {
            match (key, value) {
                (Some(k), Some(v)) => println!("Setting {} = {}", k, v),
                (Some(k), None) => println!("Getting: {}", k),
                _ => println!("Opening config..."),
            }
            // TODO: Implement config
        }
    }
    Ok(())
}

/// Display scan results in a formatted text output
fn display_scan_results(results: &ScanResults) {
    println!("Found {} items totaling {}",
        results.items.len(),
        ByteSize(results.total_size));
    println!("Safely recoverable: {}\n", ByteSize(results.total_recoverable));

    // Group by category
    let categories = [
        CleanupCategory::SystemCaches,
        CleanupCategory::AppCaches,
        CleanupCategory::Downloads,
        CleanupCategory::Logs,
        CleanupCategory::TempFiles,
        CleanupCategory::LargeFiles,
    ];

    for category in &categories {
        let items: Vec<_> = results.items.iter()
            .filter(|i| std::mem::discriminant(&i.category) == std::mem::discriminant(category))
            .collect();

        if !items.is_empty() {
            let total: u64 = items.iter().map(|i| i.size).sum();
            println!("{} {} - {}", category.icon(), category.name(), ByteSize(total));

            for item in items.iter().take(5) {
                let safety = item.category.safety_level();
                println!("  {} {}", safety.emoji(), item.path.display());
            }
            if items.len() > 5 {
                println!("  ... and {} more", items.len() - 5);
            }
            println!();
        }
    }
}
