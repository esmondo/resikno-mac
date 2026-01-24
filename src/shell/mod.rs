//! Interactive shell (REPL) for Resikno
//!
//! Provides a persistent shell experience where users can run commands
//! without exiting back to the terminal after each operation.

mod welcome;

use anyhow::Result;
use bytesize::ByteSize;
use console::Style;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::cleaner::{self, CleanupOptions};
use crate::platform::{self, PlatformPaths};
use crate::scanner::{self, CleanupCategory, SafetyLevel, ScanResults};
use crate::ui;

/// Interactive shell state
pub struct Shell {
    editor: DefaultEditor,
    last_scan: Option<ScanResults>,
}

impl Shell {
    /// Create a new shell instance
    pub fn new() -> Result<Self> {
        let editor = DefaultEditor::new()?;
        Ok(Self {
            editor,
            last_scan: None,
        })
    }

    /// Run the interactive shell loop
    pub fn run(&mut self) -> Result<()> {
        welcome::display();

        // Create styled prompt
        let cyan = Style::new().cyan();
        let green = Style::new().green();
        let prompt = format!("{} {} ", cyan.apply_to("resikno"), green.apply_to("❯"));

        loop {
            let readline = self.editor.readline(&prompt);
            match readline {
                Ok(line) => {
                    let _ = self.editor.add_history_entry(&line);

                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    match trimmed {
                        "exit" | "quit" | "q" => break,
                        "help" | "?" => self.print_help(),
                        _ => self.execute_line(trimmed),
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    let dim = Style::new().dim();
                    let cyan_style = Style::new().cyan();
                    println!(
                        "{} (type '{}' to quit)",
                        dim.apply_to("^C"),
                        cyan_style.apply_to("exit")
                    );
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    let dim = Style::new().dim();
                    println!("{}", dim.apply_to("exit"));
                    break;
                }
                Err(err) => {
                    let red = Style::new().red();
                    eprintln!("{} {:?}", red.apply_to("Error:"), err);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Execute a command line
    fn execute_line(&mut self, line: &str) {
        let args = match shlex::split(line) {
            Some(args) => args,
            None => {
                eprintln!("Error: Invalid command syntax");
                return;
            }
        };

        if args.is_empty() {
            return;
        }

        match args[0].as_str() {
            "scan" => self.cmd_scan(&args[1..]),
            "clean" => self.cmd_clean(&args[1..]),
            "analyze" => self.cmd_analyze(&args[1..]),
            "restore" => self.cmd_restore(&args[1..]),
            "review" => self.cmd_review(),
            "status" => self.cmd_status(),
            cmd => {
                let red = Style::new().red();
                let cyan = Style::new().cyan();
                eprintln!(
                    "{} Unknown command: '{}'. Type '{}' for commands.",
                    red.apply_to("Error:"),
                    cmd,
                    cyan.apply_to("help")
                );
            }
        }
    }

    /// Print help message
    fn print_help(&self) {
        let cyan = Style::new().cyan().bold();
        let gray = Style::new().color256(243);
        let white = Style::new().white();

        println!();
        println!("  {}:", white.apply_to("Commands"));
        println!(
            "    {}    {}",
            cyan.apply_to("scan"),
            gray.apply_to("Scan for cleanable files (opens TUI)")
        );
        println!(
            "    {}  {}",
            cyan.apply_to("review"),
            gray.apply_to("Review last scan results in TUI")
        );
        println!(
            "    {}   {}",
            cyan.apply_to("clean"),
            gray.apply_to("Clean specified category")
        );
        println!(
            "    {} {}",
            cyan.apply_to("analyze"),
            gray.apply_to("Analyze disk usage")
        );
        println!(
            "    {} {}",
            cyan.apply_to("restore"),
            gray.apply_to("Manage restore points")
        );
        println!(
            "    {}  {}",
            cyan.apply_to("status"),
            gray.apply_to("Show last scan summary")
        );
        println!(
            "    {}    {}",
            cyan.apply_to("help"),
            gray.apply_to("Show this help")
        );
        println!(
            "    {}    {}",
            cyan.apply_to("exit"),
            gray.apply_to("Exit resikno")
        );
        println!();
        println!(
            "  {}: {}",
            white.apply_to("Categories"),
            gray.apply_to("caches, logs, temp, downloads, all")
        );
        println!();
        println!("  {}:", white.apply_to("Scan options"));
        println!(
            "    {}  {}",
            cyan.apply_to("-m, --min-size <MB>"),
            gray.apply_to("Minimum file size (default: 0 = no min)")
        );
        println!(
            "    {}  {}",
            cyan.apply_to("-M, --max-size <MB>"),
            gray.apply_to("Maximum file size (default: 0 = no max)")
        );
        println!();
    }

    /// Scan command
    fn cmd_scan(&mut self, args: &[String]) {
        let min_size = self.parse_min_size(args);
        let max_size = self.parse_max_size(args);
        let platform = platform::current();

        let size_filter_msg = match (min_size, max_size) {
            (0, 0) => "no size filter".to_string(),
            (min, 0) => format!("min: {} MB", min),
            (0, max) => format!("max: {} MB", max),
            (min, max) => format!("{}-{} MB", min, max),
        };
        println!("Scanning system for cleanable files ({})...\n", size_filter_msg);

        match scanner::run_full_scan(&platform, None, min_size, max_size) {
            Ok(results) => {
                self.display_scan_summary(&results);
                self.last_scan = Some(results.clone());

                // Launch TUI unless --no-interactive
                if !args.iter().any(|a| a == "--no-interactive") {
                    if let Err(e) = ui::run_tui(results) {
                        eprintln!("TUI error: {}", e);
                    }
                }
            }
            Err(e) => eprintln!("Scan error: {}", e),
        }
    }

    /// Review last scan results
    fn cmd_review(&mut self) {
        match &self.last_scan {
            Some(results) => {
                if let Err(e) = ui::run_tui(results.clone()) {
                    eprintln!("TUI error: {}", e);
                }
            }
            None => {
                println!("No scan results available. Run 'scan' first.");
            }
        }
    }

    /// Show status of last scan
    fn cmd_status(&self) {
        match &self.last_scan {
            Some(results) => {
                self.display_scan_summary(results);
            }
            None => {
                println!("No scan results. Run 'scan' to analyze your system.");
            }
        }
    }

    /// Clean command
    fn cmd_clean(&mut self, args: &[String]) {
        let category = args.first().map(|s| s.as_str()).unwrap_or("all");
        let execute = args.iter().any(|a| a == "--execute");
        let safe_only = args.iter().any(|a| a == "--safe-only");

        // Run scan if we don't have results
        if self.last_scan.is_none() {
            println!("Running scan first...\n");
            let platform = platform::current();
            match scanner::run_full_scan(&platform, None, 0, 0) {
                Ok(results) => self.last_scan = Some(results),
                Err(e) => {
                    eprintln!("Scan error: {}", e);
                    return;
                }
            }
        }

        let results = self.last_scan.as_ref().unwrap();

        // Filter items by category
        let items_to_clean: Vec<_> = results
            .items
            .iter()
            .filter(|item| {
                if category == "all" {
                    true
                } else {
                    match (category, &item.category) {
                        ("caches", CleanupCategory::SystemCaches | CleanupCategory::AppCaches) => {
                            true
                        }
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
            return;
        }

        let total_size: u64 = items_to_clean.iter().map(|i| i.size).sum();
        println!(
            "Found {} items totaling {}",
            items_to_clean.len(),
            ByteSize(total_size)
        );

        if !execute {
            println!("\nDRY RUN - no files will be deleted");
            println!("Use 'clean {} --execute' to actually delete files.\n", category);

            for item in items_to_clean.iter().take(10) {
                let safety = item.category.safety_level();
                println!(
                    "  {} {} ({})",
                    safety.emoji(),
                    item.path.display(),
                    ByteSize(item.size)
                );
            }
            if items_to_clean.len() > 10 {
                println!("  ... and {} more", items_to_clean.len() - 10);
            }
        } else {
            println!("\nCleaning {} items...", items_to_clean.len());

            let paths: Vec<_> = items_to_clean.iter().map(|i| i.path.clone()).collect();
            let options = CleanupOptions {
                execute: true,
                create_restore_point: true,
                safe_only,
                force: false,
            };

            match cleaner::cleanup(&paths, &options) {
                Ok(result) => {
                    println!("\nCleanup complete!");
                    println!("   Deleted: {} items", result.items_deleted);
                    println!("   Freed: {}", ByteSize(result.bytes_freed));

                    if let Some(restore) = result.restore_point {
                        println!("   Restore point: {}", restore.id);
                    }

                    if !result.errors.is_empty() {
                        println!("\n{} errors occurred:", result.errors.len());
                        for err in result.errors.iter().take(5) {
                            println!("   {}: {}", err.path.display(), err.message);
                        }
                    }

                    // Clear cached scan results since files were deleted
                    self.last_scan = None;
                }
                Err(e) => eprintln!("Cleanup error: {}", e),
            }
        }
    }

    /// Analyze command
    fn cmd_analyze(&self, args: &[String]) {
        let duplicates = args.iter().any(|a| a == "--duplicates");
        let large: Option<u64> = args
            .iter()
            .position(|a| a == "--large")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse().ok());

        if duplicates {
            println!("Scanning for duplicate files...\n");
            println!("Duplicate detection coming soon.");
        }

        if let Some(min_mb) = large {
            let platform = platform::current();
            let min_bytes = min_mb * 1024 * 1024;
            println!("Scanning for files larger than {} MB...\n", min_mb);

            if let Some(downloads) = platform.downloads_dir() {
                match scanner::large_files::find_large_files(&[downloads], min_bytes, 0) {
                    Ok(files) => {
                        if files.is_empty() {
                            println!("No files found larger than {} MB", min_mb);
                        } else {
                            let total: u64 = files.iter().map(|f| f.size).sum();
                            println!(
                                "Found {} large files totaling {}:\n",
                                files.len(),
                                ByteSize(total)
                            );
                            for file in files.iter().take(20) {
                                println!("  {} - {}", file.path.display(), ByteSize(file.size));
                            }
                            if files.len() > 20 {
                                println!("  ... and {} more", files.len() - 20);
                            }
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }

        if !duplicates && large.is_none() {
            println!("Usage: analyze [OPTIONS]\n");
            println!("Options:");
            println!("  --duplicates       Find duplicate files");
            println!("  --large <MB>       Find files larger than MB");
        }
    }

    /// Restore command
    fn cmd_restore(&self, args: &[String]) {
        let list = args.iter().any(|a| a == "--list") || args.is_empty();

        if list {
            match crate::cleaner::backup::list_restore_points() {
                Ok(points) => {
                    if points.is_empty() {
                        println!("No restore points found.");
                        println!(
                            "\nRestore points are created automatically when you run 'clean --execute'"
                        );
                    } else {
                        println!("Available restore points:\n");
                        for point in &points {
                            println!(
                                "  {} - {} items, {}",
                                point.id,
                                point.item_count,
                                ByteSize(point.total_size)
                            );
                        }
                        println!("\nTo restore, run: restore <date> or restore --latest");
                    }
                }
                Err(e) => eprintln!("Error listing restore points: {}", e),
            }
        } else {
            println!("Restore functionality coming soon.");
        }
    }

    /// Parse --min-size argument (default: 0 = no minimum)
    fn parse_min_size(&self, args: &[String]) -> u64 {
        args.iter()
            .position(|a| a == "--min-size" || a == "-m")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    /// Parse --max-size argument (default: 0 = no maximum)
    fn parse_max_size(&self, args: &[String]) -> u64 {
        args.iter()
            .position(|a| a == "--max-size" || a == "-M")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    /// Display scan summary
    fn display_scan_summary(&self, results: &ScanResults) {
        println!(
            "Found {} items totaling {}",
            results.items.len(),
            ByteSize(results.total_size)
        );
        println!("Safely recoverable: {}\n", ByteSize(results.total_recoverable));

        // Group by category
        let categories = [
            CleanupCategory::SystemCaches,
            CleanupCategory::AppCaches,
            CleanupCategory::Downloads,
            CleanupCategory::Logs,
            CleanupCategory::TempFiles,
        ];

        for category in &categories {
            let items: Vec<_> = results
                .items
                .iter()
                .filter(|i| std::mem::discriminant(&i.category) == std::mem::discriminant(category))
                .collect();

            if !items.is_empty() {
                let total: u64 = items.iter().map(|i| i.size).sum();
                println!("{} {} - {}", category.icon(), category.name(), ByteSize(total));
            }
        }
    }
}
