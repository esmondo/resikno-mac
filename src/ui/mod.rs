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

/// Application state for the TUI
pub struct App {
    pub should_quit: bool,
    pub selected_index: usize,
    pub expanded: Vec<bool>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_quit: false,
            selected_index: 0,
            expanded: Vec::new(),
        }
    }
}

/// Run the interactive TUI
pub fn run_tui() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::default();

    // Main loop
    loop {
        terminal.draw(|f| {
            tree::render(f, &app);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Up => {
                        if app.selected_index > 0 {
                            app.selected_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        app.selected_index += 1;
                    }
                    KeyCode::Enter => {
                        // Toggle expansion
                        if app.selected_index < app.expanded.len() {
                            app.expanded[app.selected_index] = !app.expanded[app.selected_index];
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

    Ok(())
}
