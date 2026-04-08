//! UI module - Terminal user interface with ratatui

pub mod tree;
pub mod colors;
pub mod charts;

use anyhow::Result;
use bytesize::ByteSize;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use crate::cleaner::{self, CleanupOptions};
use crate::scanner::{self, ScanResults, ScannedItem, CleanupCategory, ChildEntry};

/// Main view states for the TUI
#[derive(Debug, Clone)]
pub enum ViewState {
    /// Main menu
    Menu { selected: usize },
    /// Scanning in progress
    Scanning { progress: ScanProgress },
    /// Showing scan results
    Results,
}

/// Progress information during scanning
#[derive(Debug, Clone, Default)]
pub struct ScanProgress {
    pub current_category: String,
    pub items_found: usize,
    pub bytes_found: u64,
    pub percent_complete: u8,
    /// Animation frame counter for spinner
    pub animation_frame: usize,
}

/// Dialog state for TUI modal interactions
#[derive(Debug, Clone)]
pub enum DialogState {
    None,
    ConfirmCleanup { items: Vec<ScannedItem>, total_size: u64 },
    CleanupRunning,
    CleanupResult { success: bool, message: String },
}

/// Menu items
const MENU_ITEMS: &[(&str, &str, &str)] = &[
    ("1", "Scan System",      "Full scan for all cleanable files"),
    ("2", "Quick Scan",       "Safe items only (recommended)"),
    ("3", "Review Results",   "View previous scan results"),
    ("4", "Restore Files",    "Restore files from trash"),
    ("5", "Help",             "Keyboard shortcuts & guide"),
    ("q", "Quit",             "Exit Resikno"),
];

/// Application state for the TUI
pub struct App {
    pub should_quit: bool,
    pub selected_index: usize,
    /// Scroll offset - which row is at the top of the visible area
    pub scroll_offset: usize,
    pub expanded: Vec<bool>,
    pub selected_items: Vec<bool>,
    pub scan_results: ScanResults,
    pub categories: Vec<CategoryView>,
    /// Tracks which items are expanded (item_idx -> children)
    pub expanded_items: HashMap<usize, Vec<ChildEntry>>,
    /// Selected child paths for cleanup (separate from main items)
    pub selected_children: HashMap<PathBuf, bool>,
    /// Current dialog state for modal interactions
    pub dialog: DialogState,
    /// Which button is selected in confirmation dialog (0 = Yes, 1 = No)
    pub dialog_button_selected: usize,
    /// Current main view state
    pub view_state: ViewState,
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
            scroll_offset: 0,
            expanded: vec![false; categories.len()],
            selected_items,
            scan_results: results,
            categories,
            expanded_items: HashMap::new(),
            selected_children: HashMap::new(),
            dialog: DialogState::None,
            dialog_button_selected: 0,
            view_state: ViewState::Menu { selected: 0 },
        }
    }

    /// Ensure selected index is visible in the viewport
    /// Call this after changing selected_index
    pub fn ensure_selection_visible(&mut self, viewport_height: usize) {
        let total_rows = self.visible_row_count();
        
        // Clamp selected_index to valid range
        if self.selected_index >= total_rows {
            self.selected_index = total_rows.saturating_sub(1);
        }
        
        // Adjust scroll_offset to keep selection in view
        if self.selected_index < self.scroll_offset {
            // Selection above viewport - scroll up
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + viewport_height {
            // Selection below viewport - scroll down
            self.scroll_offset = self.selected_index.saturating_sub(viewport_height - 1);
        }
        
        // Ensure scroll_offset doesn't exceed valid range
        let max_offset = total_rows.saturating_sub(viewport_height);
        if self.scroll_offset > max_offset {
            self.scroll_offset = max_offset;
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

    /// Get all selected items with full metadata
    /// Note: Child selections are converted to ScannedItems with TempFiles category
    pub fn get_selected_items(&self) -> Vec<ScannedItem> {
        let mut items = Vec::new();

        // Add selected items
        for (idx, &selected) in self.selected_items.iter().enumerate() {
            if selected {
                if let Some(item) = self.scan_results.items.get(idx) {
                    items.push(item.clone());
                }
            }
        }

        // Add selected children as new ScannedItems
        for (path, &selected) in &self.selected_children {
            if selected {
                // Find the child to get its size
                for children in self.expanded_items.values() {
                    if let Some(child) = children.iter().find(|c| &c.path == path) {
                        items.push(ScannedItem {
                            path: path.clone(),
                            size: child.size,
                            category: crate::scanner::CleanupCategory::TempFiles, // Conservative default
                            last_accessed: None,
                            last_modified: None,
                        });
                        break;
                    }
                }
            }
        }

        items
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
            match event::read()? {
                Event::Key(key) => {
                    // Handle menu state first
                    if let ViewState::Menu { selected } = app.view_state {
                        match key.code {
                            KeyCode::Up => {
                                if selected > 0 {
                                    app.view_state = ViewState::Menu { selected: selected - 1 };
                                }
                            }
                            KeyCode::Down => {
                                if selected < MENU_ITEMS.len() - 1 {
                                    app.view_state = ViewState::Menu { selected: selected + 1 };
                                }
                            }
                            KeyCode::Enter => {
                                handle_menu_selection(&mut app, selected)?;
                            }
                            KeyCode::Char('1') => handle_menu_selection(&mut app, 0)?,
                            KeyCode::Char('2') => handle_menu_selection(&mut app, 1)?,
                            KeyCode::Char('3') => handle_menu_selection(&mut app, 2)?,
                            KeyCode::Char('4') => handle_menu_selection(&mut app, 3)?,
                            KeyCode::Char('5') => handle_menu_selection(&mut app, 4)?,
                            KeyCode::Char('q') | KeyCode::Char('Q') => {
                                if selected == 5 {
                                    app.should_quit = true;
                                } else {
                                    app.view_state = ViewState::Menu { selected: 5 };
                                }
                            }
                            KeyCode::Esc => app.should_quit = true,
                            _ => {}
                        }
                        continue;
                    }
                    
                    // Handle scanning state
                    if let ViewState::Scanning { .. } = app.view_state {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                // Cancel scanning
                                app.view_state = ViewState::Menu { selected: 0 };
                            }
                            _ => {}
                        }
                        continue;
                    }
                    
                    // Handle dialog states
                    match &app.dialog {
                        DialogState::ConfirmCleanup { items, total_size: _ } => {
                            match key.code {
                                KeyCode::Left => {
                                    // Move to Yes button
                                    app.dialog_button_selected = 0;
                                }
                                KeyCode::Right => {
                                    // Move to No button
                                    app.dialog_button_selected = 1;
                                }
                                KeyCode::Enter => {
                                    // Execute based on selected button
                                    if app.dialog_button_selected == 0 {
                                        // Yes - Execute cleanup
                                        let items_clone = items.clone();
                                        app.dialog = DialogState::CleanupRunning;
                                        
                                        let options = CleanupOptions {
                                            execute: true,
                                            create_restore_point: true,
                                            safe_only: false,
                                            force: false,
                                        };
                                        
                                        match cleaner::cleanup_items(&items_clone, &options) {
                                            Ok(result) => {
                                                let msg = format!(
                                                    "Deleted: {} items\nFreed: {}\nRestore: {}",
                                                    result.items_deleted,
                                                    ByteSize(result.bytes_freed),
                                                    result.restore_point.as_ref()
                                                        .map(|r| r.id.clone())
                                                        .unwrap_or_else(|| "none".to_string())
                                                );
                                                app.dialog = DialogState::CleanupResult { 
                                                    success: true, 
                                                    message: msg 
                                                };
                                            }
                                            Err(e) => {
                                                app.dialog = DialogState::CleanupResult { 
                                                    success: false, 
                                                    message: format!("Cleanup failed: {}", e) 
                                                };
                                            }
                                        }
                                    } else {
                                        // No - Cancel
                                        app.dialog = DialogState::None;
                                    }
                                }
                                KeyCode::Char('y') | KeyCode::Char('Y') => {
                                    // Shortcut: Yes
                                    let items_clone = items.clone();
                                    app.dialog = DialogState::CleanupRunning;
                                    
                                    let options = CleanupOptions {
                                        execute: true,
                                        create_restore_point: true,
                                        safe_only: false,
                                        force: false,
                                    };
                                    
                                    match cleaner::cleanup_items(&items_clone, &options) {
                                        Ok(result) => {
                                            let msg = format!(
                                                "Deleted: {} items\nFreed: {}\nRestore: {}",
                                                result.items_deleted,
                                                ByteSize(result.bytes_freed),
                                                result.restore_point.as_ref()
                                                    .map(|r| r.id.clone())
                                                    .unwrap_or_else(|| "none".to_string())
                                            );
                                            app.dialog = DialogState::CleanupResult { 
                                                success: true, 
                                                message: msg 
                                            };
                                        }
                                        Err(e) => {
                                            app.dialog = DialogState::CleanupResult { 
                                                success: false, 
                                                message: format!("Cleanup failed: {}", e) 
                                            };
                                        }
                                    }
                                }
                                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                    app.dialog = DialogState::None;
                                }
                                _ => {}
                            }
                            continue;
                        }
                        DialogState::CleanupResult { .. } => {
                            // Any key dismisses the result dialog
                            app.dialog = DialogState::None;
                            continue;
                        }
                        _ => {} // No dialog active, process normal keys
                    }
                    
                    let max_row = app.visible_row_count().saturating_sub(1);
                    const VIEWPORT_HEIGHT: usize = 15; // Approximate content area height

                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,

                        KeyCode::Up | KeyCode::Char('k') => {
                            if app.selected_index > 0 {
                                app.selected_index -= 1;
                            }
                            app.ensure_selection_visible(VIEWPORT_HEIGHT);
                        }

                        KeyCode::Down | KeyCode::Char('j') => {
                            if app.selected_index < max_row {
                                app.selected_index += 1;
                            }
                            app.ensure_selection_visible(VIEWPORT_HEIGHT);
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
                                // Get selected items for confirmation dialog
                                let selected_items = app.get_selected_items();
                                let total_size = app.get_selected_size();
                                app.dialog = DialogState::ConfirmCleanup { 
                                    items: selected_items, 
                                    total_size 
                                };
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

                        KeyCode::Char('m') => {
                            // Return to main menu
                            app.view_state = ViewState::Menu { selected: 0 };
                        }

                        _ => {}
                    }
                }
                
                Event::Mouse(mouse_event) => {
                    let max_row = app.visible_row_count().saturating_sub(1);
                    const VIEWPORT_HEIGHT: usize = 15;
                    
                    match mouse_event.kind {
                        MouseEventKind::ScrollDown => {
                            // Scroll viewport down without moving selection
                            let max_scroll = max_row.saturating_sub(VIEWPORT_HEIGHT - 1);
                            if app.scroll_offset < max_scroll {
                                app.scroll_offset += 1;
                            }
                            // If selection is now above viewport, move it down
                            if app.selected_index < app.scroll_offset {
                                app.selected_index = app.scroll_offset;
                            }
                        }
                        MouseEventKind::ScrollUp => {
                            // Scroll viewport up without moving selection
                            if app.scroll_offset > 0 {
                                app.scroll_offset -= 1;
                            }
                            // If selection is now below viewport, move it up
                            let viewport_end = app.scroll_offset + VIEWPORT_HEIGHT - 1;
                            if app.selected_index > viewport_end {
                                app.selected_index = viewport_end;
                            }
                        }
                        _ => {} // Ignore other mouse events (clicks, etc.)
                    }
                }
                
                _ => {} // Ignore other events (resize, etc.)
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

/// Handle menu selection
fn handle_menu_selection(app: &mut App, selected: usize) -> Result<()> {
    use crate::platform;
    use crate::scanner;
    
    match selected {
        0 => {
            // Scan System - Full scan
            app.view_state = ViewState::Scanning { 
                progress: ScanProgress {
                    current_category: "System Caches".to_string(),
                    items_found: 0,
                    bytes_found: 0,
                    percent_complete: 0,
                    animation_frame: 0,
                }
            };
            
            // Perform scan
            let platform = platform::current();
            match scanner::run_full_scan(&platform, None, 0, 0) {
                Ok(results) => {
                    // Rebuild app with new results
                    *app = App::new(results);
                    app.view_state = ViewState::Results;
                }
                Err(e) => {
                    app.dialog = DialogState::CleanupResult {
                        success: false,
                        message: format!("Scan failed: {}", e),
                    };
                    app.view_state = ViewState::Menu { selected: 0 };
                }
            }
        }
        1 => {
            // Quick Scan - Safe items only
            app.view_state = ViewState::Scanning { 
                progress: ScanProgress {
                    current_category: "Safe Items".to_string(),
                    items_found: 0,
                    bytes_found: 0,
                    percent_complete: 0,
                    animation_frame: 0,
                }
            };
            
            // Perform scan with safe-only filtering
            let platform = platform::current();
            match scanner::run_full_scan(&platform, None, 0, 0) {
                Ok(mut results) => {
                    // Filter to safe items only
                    results.items.retain(|item| {
                        matches!(item.category.safety_level(), 
                            scanner::SafetyLevel::Safe | scanner::SafetyLevel::MostlySafe)
                    });
                    results.total_size = results.items.iter().map(|i| i.size).sum();
                    results.total_recoverable = results.total_size;
                    
                    *app = App::new(results);
                    app.view_state = ViewState::Results;
                }
                Err(e) => {
                    app.dialog = DialogState::CleanupResult {
                        success: false,
                        message: format!("Scan failed: {}", e),
                    };
                    app.view_state = ViewState::Menu { selected: 0 };
                }
            }
        }
        2 => {
            // Review Results - only if we have results
            if !app.scan_results.items.is_empty() {
                app.view_state = ViewState::Results;
            } else {
                app.dialog = DialogState::CleanupResult {
                    success: false,
                    message: "No previous scan results.\nRun 'Scan System' first.".to_string(),
                };
            }
        }
        3 => {
            // Restore Files - placeholder
            app.dialog = DialogState::CleanupResult {
                success: false,
                message: "Restore feature coming soon!\nUse 'resikno restore' in terminal.".to_string(),
            };
        }
        4 => {
            // Help - placeholder
            app.dialog = DialogState::CleanupResult {
                success: false,
                message: "Keyboard Shortcuts:\n\n↑↓ or jk - Navigate\nSpace - Select item\nEnter - Expand/collapse\nc - Clean selected\na - Select all\nf - Open in Finder\nq - Quit\n\nPress any key to close.".to_string(),
            };
        }
        5 => {
            // Quit
            app.should_quit = true;
        }
        _ => {}
    }
    
    Ok(())
}
