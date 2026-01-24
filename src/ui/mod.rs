//! UI module - Terminal user interface with ratatui

pub mod tree;
pub mod colors;
pub mod charts;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

use crate::scanner::{ScanResults, CleanupCategory};

/// Application state for the TUI
pub struct App {
    pub should_quit: bool,
    pub selected_index: usize,
    pub expanded: Vec<bool>,
    pub selected_items: Vec<bool>,
    pub scan_results: ScanResults,
    pub categories: Vec<CategoryView>,
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
        }
    }

    /// Get total number of visible rows
    pub fn visible_row_count(&self) -> usize {
        let mut count = 0;
        for (i, cat) in self.categories.iter().enumerate() {
            count += 1; // Category header
            if self.expanded.get(i).copied().unwrap_or(false) {
                count += cat.item_indices.len(); // Child items
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
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub enum RowInfo {
    Category(usize),
    Item { cat_idx: usize, item_idx: usize, item_offset: usize },
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
                        // Toggle expansion for category rows
                        if let Some(RowInfo::Category(cat_idx)) = app.row_info(app.selected_index) {
                            if cat_idx < app.expanded.len() {
                                app.expanded[cat_idx] = !app.expanded[cat_idx];
                            }
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
                            None => {}
                        }
                    }

                    KeyCode::Char('c') => {
                        // Count selected items and show info (cleanup would happen here)
                        let selected_count = app.selected_items.iter().filter(|&&s| s).count();
                        if selected_count > 0 {
                            // For now, just quit - full cleanup would be implemented later
                            // In production: show confirmation dialog, then call cleaner
                            app.should_quit = true;
                        }
                    }

                    KeyCode::Char('a') => {
                        // Select all
                        let all_selected = app.selected_items.iter().all(|&s| s);
                        app.selected_items.iter_mut().for_each(|s| *s = !all_selected);
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

    Ok(())
}
