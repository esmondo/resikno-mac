//! UI module - Terminal user interface with ratatui

pub mod tree;
pub mod colors;
pub mod charts;

use anyhow::Result;
use bytesize::ByteSize;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::cleaner::{self, CleanupOptions};
use crate::scanner::{self, ScanResults, CleanupCategory, ChildEntry};

/// Application state for the TUI
pub struct App {
    pub should_quit: bool,
    pub selected_index: usize,
    pub expanded: Vec<bool>,
    pub selected_items: Vec<bool>,
    pub scan_results: ScanResults,
    pub categories: Vec<CategoryView>,
    /// Pending cleanup indices (set when user presses 'c')
    pub pending_cleanup: Option<Vec<usize>>,
    /// Tracks which items are expanded (item_idx -> children)
    pub expanded_items: HashMap<usize, Vec<ChildEntry>>,
    /// Selected child paths for cleanup (separate from main items)
    pub selected_children: HashMap<PathBuf, bool>,
}

/// A category view for the TUI tree
pub struct CategoryView {
    pub category: CleanupCategory,
    pub total_size: u64,
    pub item_indices: Vec<usize>,  // Indices into scan_results.items
    pub expanded: bool,
    pub selected: bool,
}

impl App {
    pub fn new(results: ScanResults) -> Self {
        // Group items by category
        let mut categories = Vec::new();
        let category_types = [
            CleanupCategory::SystemCaches,
            CleanupCategory::AppCaches,
            CleanupCategory::Logs,
            CleanupCategory::TempFiles,
            CleanupCategory::Downloads,
            CleanupCategory::LargeFiles,
            CleanupCategory::IOSBackups,
            CleanupCategory::XcodeData,
        ];

        for cat_type in &category_types {
            let indices: Vec<usize> = results.items.iter()
                .enumerate()
                .filter(|(_, item)| std::mem::discriminant(&item.category) == std::mem::discriminant(cat_type))
                .map(|(i, _)| i)
                .collect();

            if !indices.is_empty() {
                let total: u64 = indices.iter()
                    .map(|&i| results.items[i].size)
                    .sum();

                categories.push(CategoryView {
                    category: cat_type.clone(),
                    total_size: total,
                    item_indices: indices,
                    expanded: false,
                    selected: false,
                });
            }
        }

        let selected_items = vec![false; results.items.len()];

        Self {
            should_quit: false,
            selected_index: 0,
            expanded: vec![false; categories.len()],
            selected_items,
            scan_results: results,
            categories,
            pending_cleanup: None,
            expanded_items: HashMap::new(),
            selected_children: HashMap::new(),
        }
    }

    /// Get total number of visible rows
    pub fn visible_row_count(&self) -> usize {
        let mut count = 0;
        for (i, cat) in self.categories.iter().enumerate() {
            count += 1; // Category header
            if self.expanded.get(i).copied().unwrap_or(false) {
                for &item_idx in &cat.item_indices {
                    count += 1; // Item row
                    // Add children if item is expanded
                    if let Some(children) = self.expanded_items.get(&item_idx) {
                        count += children.len();
                    }
                }
            }
        }
        count
    }

    /// Get the item at a given visible row index
    pub fn row_info(&self, row: usize) -> Option<RowInfo> {
        let mut current_row = 0;
        for (cat_idx, cat) in self.categories.iter().enumerate() {
            if current_row == row {
                return Some(RowInfo::Category(cat_idx));
            }
            current_row += 1;

            if self.expanded.get(cat_idx).copied().unwrap_or(false) {
                for (item_offset, &item_idx) in cat.item_indices.iter().enumerate() {
                    if current_row == row {
                        return Some(RowInfo::Item { cat_idx, item_idx, item_offset });
                    }
                    current_row += 1;

                    // Check children if item is expanded
                    if let Some(children) = self.expanded_items.get(&item_idx) {
                        for (child_idx, child) in children.iter().enumerate() {
                            if current_row == row {
                                return Some(RowInfo::Child {
                                    cat_idx,
                                    item_idx,
                                    child_idx,
                                    path: child.path.clone()
                                });
                            }
                            current_row += 1;
                        }
                    }
                }
            }
        }
        None
    }

    /// Toggle expansion of an item (load children if needed)
    pub fn toggle_item_expansion(&mut self, item_idx: usize) {
        if self.expanded_items.contains_key(&item_idx) {
            // Collapse - remove children
            self.expanded_items.remove(&item_idx);
        } else {
            // Expand - load children
            if let Some(item) = self.scan_results.items.get(item_idx) {
                if item.path.is_dir() {
                    if let Ok(children) = scanner::scan_directory_children(&item.path) {
                        if !children.is_empty() {
                            self.expanded_items.insert(item_idx, children);
                        }
                    }
                }
            }
        }
    }

    /// Check if an item is expanded
    pub fn is_item_expanded(&self, item_idx: usize) -> bool {
        self.expanded_items.contains_key(&item_idx)
    }

    /// Check if an item has expandable children (is a directory)
    pub fn is_item_expandable(&self, item_idx: usize) -> bool {
        self.scan_results.items.get(item_idx)
            .map(|item| item.path.is_dir())
            .unwrap_or(false)
    }

    /// Get the path of the currently selected row
    pub fn selected_path(&self) -> Option<PathBuf> {
        match self.row_info(self.selected_index)? {
            RowInfo::Category(cat_idx) => {
                // For category, return the first item's path (usually a directory)
                let cat = self.categories.get(cat_idx)?;
                let first_idx = cat.item_indices.first()?;
                Some(self.scan_results.items.get(*first_idx)?.path.clone())
            }
            RowInfo::Item { item_idx, .. } => {
                Some(self.scan_results.items.get(item_idx)?.path.clone())
            }
            RowInfo::Child { path, .. } => {
                Some(path)
            }
        }
    }

    /// Get all selected paths (both items and children)
    pub fn get_selected_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Add selected items
        for (idx, &selected) in self.selected_items.iter().enumerate() {
            if selected {
                if let Some(item) = self.scan_results.items.get(idx) {
                    paths.push(item.path.clone());
                }
            }
        }

        // Add selected children
        for (path, &selected) in &self.selected_children {
            if selected {
                paths.push(path.clone());
            }
        }

        paths
    }

    /// Calculate total size of selected items
    pub fn get_selected_size(&self) -> u64 {
        let mut total = 0u64;

        // Add selected items
        for (idx, &selected) in self.selected_items.iter().enumerate() {
            if selected {
                if let Some(item) = self.scan_results.items.get(idx) {
                    total += item.size;
                }
            }
        }

        // Add selected children
        for (path, &selected) in &self.selected_children {
            if selected {
                // Find the child size
                for children in self.expanded_items.values() {
                    if let Some(child) = children.iter().find(|c| &c.path == path) {
                        total += child.size;
                        break;
                    }
                }
            }
        }

        total
    }
}

#[derive(Debug, Clone)]
pub enum RowInfo {
    Category(usize),
    Item { cat_idx: usize, item_idx: usize, item_offset: usize },
    Child { cat_idx: usize, item_idx: usize, child_idx: usize, path: PathBuf },
}

/// Run the interactive TUI with scan results
pub fn run_tui(results: ScanResults) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state with real scan data
    let mut app = App::new(results);

    // Main loop
    loop {
        terminal.draw(|f| {
            tree::render(f, &app);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let max_row = app.visible_row_count().saturating_sub(1);

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,

                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.selected_index > 0 {
                            app.selected_index -= 1;
                        }
                    }

                    KeyCode::Down | KeyCode::Char('j') => {
                        if app.selected_index < max_row {
                            app.selected_index += 1;
                        }
                    }

                    KeyCode::Enter => {
                        // Toggle expansion for category or item rows
                        match app.row_info(app.selected_index) {
                            Some(RowInfo::Category(cat_idx)) => {
                                if cat_idx < app.expanded.len() {
                                    app.expanded[cat_idx] = !app.expanded[cat_idx];
                                }
                            }
                            Some(RowInfo::Item { item_idx, .. }) => {
                                // Toggle item expansion (drill down into directory)
                                if app.is_item_expandable(item_idx) {
                                    app.toggle_item_expansion(item_idx);
                                }
                            }
                            _ => {}
                        }
                    }

                    KeyCode::Char(' ') => {
                        // Toggle selection
                        match app.row_info(app.selected_index) {
                            Some(RowInfo::Category(cat_idx)) => {
                                // Toggle all items in category
                                if let Some(cat) = app.categories.get(cat_idx) {
                                    let all_selected = cat.item_indices.iter()
                                        .all(|&i| app.selected_items.get(i).copied().unwrap_or(false));
                                    for &idx in &cat.item_indices {
                                        if idx < app.selected_items.len() {
                                            app.selected_items[idx] = !all_selected;
                                        }
                                    }
                                }
                            }
                            Some(RowInfo::Item { item_idx, .. }) => {
                                // Toggle single item
                                if item_idx < app.selected_items.len() {
                                    app.selected_items[item_idx] = !app.selected_items[item_idx];
                                }
                            }
                            Some(RowInfo::Child { path, .. }) => {
                                // Toggle child selection
                                let current = app.selected_children.get(&path).copied().unwrap_or(false);
                                app.selected_children.insert(path, !current);
                            }
                            None => {}
                        }
                    }

                    KeyCode::Char('c') => {
                        // Check if anything is selected (items or children)
                        let has_selected_items = app.selected_items.iter().any(|&s| s);
                        let has_selected_children = app.selected_children.values().any(|&s| s);

                        if has_selected_items || has_selected_children {
                            // Store selected indices (for items only, children handled separately)
                            let selected_indices: Vec<usize> = app.selected_items
                                .iter()
                                .enumerate()
                                .filter(|(_, &selected)| selected)
                                .map(|(i, _)| i)
                                .collect();
                            app.pending_cleanup = Some(selected_indices);
                            app.should_quit = true;
                        }
                    }

                    KeyCode::Char('a') => {
                        // Select all
                        let all_selected = app.selected_items.iter().all(|&s| s);
                        app.selected_items.iter_mut().for_each(|s| *s = !all_selected);
                    }

                    KeyCode::Char('f') => {
                        // Reveal in Finder
                        if let Some(path) = app.selected_path() {
                            let _ = std::process::Command::new("open")
                                .arg("-R")  // Reveal in Finder
                                .arg(&path)
                                .spawn();
                        }
                    }

                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Handle pending cleanup if user pressed 'c'
    if app.pending_cleanup.is_some() {
        let paths = app.get_selected_paths();
        let total_size = app.get_selected_size();
        handle_cleanup(&paths, total_size)?;
    }

    Ok(())
}

/// Handle cleanup after TUI exit
fn handle_cleanup(paths: &[PathBuf], total_size: u64) -> Result<()> {

    if paths.is_empty() {
        return Ok(());
    }

    // Show confirmation prompt
    println!();
    println!("┌─────────────────────────────────────────────────┐");
    println!("│  🧹 Cleanup Confirmation                         │");
    println!("├─────────────────────────────────────────────────┤");
    println!("│  Items:  {} files/folders", paths.len());
    println!("│  Size:   {}", ByteSize(total_size));
    println!("└─────────────────────────────────────────────────┘");
    println!();
    println!("A restore point will be created before deletion.");
    print!("Proceed with cleanup? [y/N] ");
    io::stdout().flush()?;

    // Read user confirmation
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Cleanup cancelled.");
        return Ok(());
    }

    println!();
    println!("Creating restore point...");

    let options = CleanupOptions {
        execute: true,
        create_restore_point: true,
        safe_only: false,
        force: false,
    };

    match cleaner::cleanup(&paths, &options) {
        Ok(result) => {
            println!();
            println!("┌─────────────────────────────────────────────────┐");
            println!("│  ✅ Cleanup Complete                             │");
            println!("├─────────────────────────────────────────────────┤");
            println!("│  Deleted: {} items", result.items_deleted);
            println!("│  Freed:   {}", ByteSize(result.bytes_freed));
            if let Some(restore) = &result.restore_point {
                println!("│  Restore: {}", restore.id);
            }
            println!("└─────────────────────────────────────────────────┘");

            if !result.errors.is_empty() {
                println!();
                println!("⚠️  {} errors occurred:", result.errors.len());
                for err in result.errors.iter().take(5) {
                    println!("   {}: {}", err.path.display(), err.message);
                }
                if result.errors.len() > 5 {
                    println!("   ... and {} more", result.errors.len() - 5);
                }
            }
        }
        Err(e) => {
            println!();
            println!("❌ Cleanup failed: {}", e);
        }
    }

    Ok(())
}
