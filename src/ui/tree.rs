//! Interactive tree view component

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::{App, colors::ColorScheme};

/// Render the main tree view
pub fn render(frame: &mut Frame, app: &App) {
    let colors = ColorScheme::default();

    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Footer/help
        ])
        .split(frame.size());

    // Render header
    render_header(frame, chunks[0], &colors);

    // Render main content (placeholder for now)
    render_content(frame, chunks[1], app, &colors);

    // Render footer
    render_footer(frame, chunks[2], &colors);
}

/// Render the header bar
fn render_header(frame: &mut Frame, area: Rect, colors: &ColorScheme) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled("RESIKNO-MAK", colors.header),
        Span::raw("  "),
        Span::styled("💾 Scanning...", colors.muted),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Disk Cleaner ")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(header, area);
}

/// Render the main content area
fn render_content(frame: &mut Frame, area: Rect, app: &App, colors: &ColorScheme) {
    // Placeholder items - will be replaced with real scan data
    let items: Vec<ListItem> = vec![
        create_list_item("📦 System Caches", "45.2 GB", "🔵 SAFE", 45, colors, app.selected_index == 0),
        create_list_item("📦 App Caches", "32.1 GB", "🔵 SAFE", 32, colors, app.selected_index == 1),
        create_list_item("📋 Logs", "12.5 GB", "🟢 REVIEW", 12, colors, app.selected_index == 2),
        create_list_item("📂 Downloads", "98.7 GB", "🟡 REVIEW", 98, colors, app.selected_index == 3),
        create_list_item("📱 iOS Backups", "45.0 GB", "🟢 REVIEW", 45, colors, app.selected_index == 4),
    ];

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Categories ")
                .border_style(Style::default().fg(Color::White)),
        );

    frame.render_widget(list, area);
}

/// Render the footer with keyboard shortcuts
fn render_footer(frame: &mut Frame, area: Rect, colors: &ColorScheme) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("[↑↓]", colors.header),
        Span::raw(" Navigate  "),
        Span::styled("[Enter]", colors.header),
        Span::raw(" Expand  "),
        Span::styled("[Space]", colors.header),
        Span::raw(" Select  "),
        Span::styled("[C]", colors.header),
        Span::raw(" Clean  "),
        Span::styled("[Q]", colors.header),
        Span::raw(" Quit"),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(footer, area);
}

/// Create a styled list item
fn create_list_item<'a>(
    name: &'a str,
    size: &'a str,
    safety: &'a str,
    percent: u8,
    colors: &ColorScheme,
    selected: bool,
) -> ListItem<'a> {
    let bar = super::colors::progress_bar(percent as f64, 10);

    let style = if selected {
        colors.selected
    } else {
        colors.normal
    };

    ListItem::new(Line::from(vec![
        Span::raw(if selected { " > " } else { "   " }),
        Span::raw(name),
        Span::raw("  "),
        Span::styled(size, colors.for_size(percent as u64 * 1024 * 1024 * 1024)),
        Span::raw("  "),
        Span::raw(safety),
        Span::raw("  "),
        Span::styled(bar, Style::default().fg(colors.for_percentage(percent as f64))),
    ]))
    .style(style)
}
