//! Command implementations

use anyhow::Result;
use bytesize::ByteSize;
use super::args::{Cli, Commands};
use crate::platform::{self, PlatformPaths};
use crate::scanner::{self, CleanupCategory, ScanResults, SafetyLevel};
use crate::cleaner::{self, CleanupOptions};
use crate::cleaner::backup;
use crate::ui;

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

            // Launch TUI if interactive mode
            if !no_interactive {
                println!("\nLaunching interactive mode... (press 'q' to quit)\n");
                ui::run_tui()?;
            }
        }
        Commands::Clean { category, execute, safe_only } => {
            let platform = platform::current();

            // Run scan to find cleanable items
            println!("Scanning for {} items...\n", category);
            let results = scanner::run_full_scan(&platform, None)?;

            // Filter by category if specified
            let items_to_clean: Vec<_> = results.items.iter()
                .filter(|item| {
                    if category == "all" {
                        true
                    } else {
                        match (category.as_str(), &item.category) {
                            ("caches", CleanupCategory::SystemCaches | CleanupCategory::AppCaches) => true,
                            ("logs", CleanupCategory::Logs) => true,
                            ("temp", CleanupCategory::TempFiles) => true,
                            ("downloads", CleanupCategory::Downloads) => true,
                            _ => false,
                        }
                    }
                })
                .filter(|item| {
                    if safe_only {
                        matches!(item.category.safety_level(), SafetyLevel::Safe)
                    } else {
                        true
                    }
                })
                .collect();

            if items_to_clean.is_empty() {
                println!("No items found to clean.");
                return Ok(());
            }

            let total_size: u64 = items_to_clean.iter().map(|i| i.size).sum();
            println!("Found {} items totaling {}", items_to_clean.len(), ByteSize(total_size));

            if !execute {
                println!("\n⚠️  DRY RUN - no files will be deleted");
                println!("Use --execute to actually delete files.\n");

                for item in &items_to_clean {
                    let safety = item.category.safety_level();
                    println!("  {} {} ({})", safety.emoji(), item.path.display(), ByteSize(item.size));
                }
            } else {
                println!("\n🗑️  Cleaning {} items...", items_to_clean.len());

                let paths: Vec<_> = items_to_clean.iter().map(|i| i.path.clone()).collect();
                let options = CleanupOptions {
                    execute: true,
                    create_restore_point: true,
                    safe_only,
                    force: false,
                };

                let result = cleaner::cleanup(&paths, &options)?;

                println!("\n✅ Cleanup complete!");
                println!("   Deleted: {} items", result.items_deleted);
                println!("   Freed: {}", ByteSize(result.bytes_freed));

                if let Some(restore) = result.restore_point {
                    println!("   Restore point: {}", restore.id);
                }

                if !result.errors.is_empty() {
                    println!("\n⚠️  {} errors occurred:", result.errors.len());
                    for err in &result.errors {
                        println!("   {}: {}", err.path.display(), err.message);
                    }
                }
            }
        }
        Commands::Analyze { duplicates, large, old } => {
            let analyze_platform = platform::current();

            if duplicates {
                println!("🔍 Scanning for duplicate files...\n");
                println!("⚠️  Duplicate detection coming soon.");
                println!("   This will find files with identical content using SHA-256 hashing.");
            }

            if let Some(min_mb) = large {
                let min_bytes = min_mb * 1024 * 1024;
                println!("🔍 Scanning for files larger than {} MB...\n", min_mb);

                if let Some(home) = analyze_platform.downloads_dir() {
                    let large_files = scanner::large_files::find_large_files(&[home], min_bytes)?;

                    if large_files.is_empty() {
                        println!("No files found larger than {} MB", min_mb);
                    } else {
                        let total: u64 = large_files.iter().map(|f| f.size).sum();
                        println!("Found {} large files totaling {}:\n", large_files.len(), ByteSize(total));

                        for file in large_files.iter().take(20) {
                            println!("  📁 {} - {}", file.path.display(), ByteSize(file.size));
                        }
                        if large_files.len() > 20 {
                            println!("  ... and {} more", large_files.len() - 20);
                        }
                    }
                }
            }

            if let Some(days) = old {
                println!("🔍 Scanning for files older than {} days...\n", days);
                println!("⚠️  Old file detection coming soon.");
                println!("   This will find files not accessed in {} days.", days);
            }

            if !duplicates && large.is_none() && old.is_none() {
                println!("Usage: resikno analyze [OPTIONS]\n");
                println!("Options:");
                println!("  --duplicates       Find duplicate files");
                println!("  --large <MB>       Find files larger than MB");
                println!("  --old <DAYS>       Find files older than DAYS");
            }
        }
        Commands::Restore { date, latest, list } => {
            let restore_points = backup::list_restore_points()?;

            if list || (date.is_none() && !latest) {
                // List all restore points
                if restore_points.is_empty() {
                    println!("No restore points found.");
                    println!("\nRestore points are created automatically when you run 'resikno clean --execute'");
                } else {
                    println!("Available restore points:\n");
                    for point in &restore_points {
                        println!("  📦 {} - {} items, {}",
                            point.id,
                            point.item_count,
                            ByteSize(point.total_size));
                    }
                    println!("\nTo restore, run: resikno restore <date> or resikno restore --latest");
                }
            } else {
                // Find the restore point to use
                let target = if latest {
                    restore_points.first()
                } else if let Some(ref d) = date {
                    restore_points.iter().find(|p| p.id.starts_with(d))
                } else {
                    None
                };

                match target {
                    Some(point) => {
                        println!("🔄 Restore point: {}", point.id);
                        println!("   Items: {}", point.item_count);
                        println!("   Size: {}", ByteSize(point.total_size));
                        println!("\n⚠️  Restore functionality coming soon.");
                        println!("   Manifest saved at: {}", point.manifest_path.display());
                    }
                    None => {
                        println!("❌ Restore point not found.");
                        if !restore_points.is_empty() {
                            println!("\nAvailable restore points:");
                            for point in restore_points.iter().take(5) {
                                println!("  - {}", point.id);
                            }
                        }
                    }
                }
            }
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
