//! Interactive tree view component

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use bytesize::ByteSize;

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
    render_header(frame, chunks[0], app, &colors);

    // Render main content
    render_content(frame, chunks[1], app, &colors);

    // Render footer
    render_footer(frame, chunks[2], &colors);
}

/// Render the header bar with real stats
fn render_header(frame: &mut Frame, area: Rect, app: &App, colors: &ColorScheme) {
    let item_count = app.scan_results.items.len();
    let total_size = ByteSize(app.scan_results.total_size);
    let recoverable = ByteSize(app.scan_results.total_recoverable);

    let selected_count = app.selected_items.iter().filter(|&&s| s).count();
    let selected_size: u64 = app.scan_results.items.iter()
        .enumerate()
        .filter(|(i, _)| app.selected_items.get(*i).copied().unwrap_or(false))
        .map(|(_, item)| item.size)
        .sum();

    let status = if selected_count > 0 {
        format!("{} selected ({}) | Total: {} in {} items",
            selected_count, ByteSize(selected_size), total_size, item_count)
    } else {
        format!("Found {} ({} recoverable) in {} items",
            total_size, recoverable, item_count)
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled("RESIKNO", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(status, colors.muted),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Disk Cleaner ")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(header, area);
}

/// Render the main content area with real scan data
fn render_content(frame: &mut Frame, area: Rect, app: &App, colors: &ColorScheme) {
    let mut items: Vec<ListItem> = Vec::new();
    let mut current_row = 0;

    // Find the maximum size for alignment
    let max_size: u64 = app.categories.iter()
        .map(|c| c.total_size)
        .max()
        .unwrap_or(1);

    for (cat_idx, cat) in app.categories.iter().enumerate() {
        let is_selected = app.selected_index == current_row;
        let is_expanded = app.expanded.get(cat_idx).copied().unwrap_or(false);

        // Check if all items in category are selected
        let all_items_selected = cat.item_indices.iter()
            .all(|&i| app.selected_items.get(i).copied().unwrap_or(false));
        let some_items_selected = cat.item_indices.iter()
            .any(|&i| app.selected_items.get(i).copied().unwrap_or(false));

        // Category row
        let checkbox = if all_items_selected {
            "[x]"
        } else if some_items_selected {
            "[-]"
        } else {
            "[ ]"
        };

        let expand_indicator = if is_expanded { "▼" } else { "▶" };
        let size_str = format!("{:>10}", ByteSize(cat.total_size));
        let safety = cat.category.safety_level();
        let percent = (cat.total_size as f64 / max_size as f64 * 100.0) as u8;
        let bar = super::colors::progress_bar(percent as f64, 15);

        // Bar color matches safety level, NOT size
        let bar_color = match safety {
            crate::scanner::SafetyLevel::Safe => Color::Blue,
            crate::scanner::SafetyLevel::MostlySafe => Color::Green,
            crate::scanner::SafetyLevel::ReviewCarefully => Color::Yellow,
            crate::scanner::SafetyLevel::Caution => Color::Red,
            crate::scanner::SafetyLevel::Protected => Color::White,
        };

        let style = if is_selected {
            colors.selected
        } else {
            colors.normal
        };

        items.push(ListItem::new(Line::from(vec![
            Span::raw(if is_selected { " >" } else { "  " }),
            Span::raw(format!(" {} ", checkbox)),
            Span::raw(format!("{} ", expand_indicator)),
            Span::raw(cat.category.icon()),
            Span::raw(" "),
            Span::styled(format!("{:<16}", cat.category.name()), Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(size_str, colors.for_size(cat.total_size)),
            Span::raw("  "),
            Span::styled(format!("{:<8}", safety.label()), match safety {
                crate::scanner::SafetyLevel::Safe => Style::default().fg(Color::Blue),
                crate::scanner::SafetyLevel::MostlySafe => Style::default().fg(Color::Green),
                crate::scanner::SafetyLevel::ReviewCarefully => Style::default().fg(Color::Yellow),
                crate::scanner::SafetyLevel::Caution => Style::default().fg(Color::Red),
                crate::scanner::SafetyLevel::Protected => Style::default().fg(Color::White),
            }),
            Span::styled(bar, Style::default().fg(bar_color)),
        ])).style(style));

        current_row += 1;

        // Child items when expanded
        if is_expanded {
            for &item_idx in &cat.item_indices {
                let is_item_selected = app.selected_index == current_row;
                let is_checked = app.selected_items.get(item_idx).copied().unwrap_or(false);

                if let Some(item) = app.scan_results.items.get(item_idx) {
                    let item_checkbox = if is_checked { "[x]" } else { "[ ]" };
                    let file_name = item.path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| item.path.display().to_string());
                    let item_size = format!("{:>10}", ByteSize(item.size));

                    let item_style = if is_item_selected {
                        colors.selected
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    items.push(ListItem::new(Line::from(vec![
                        Span::raw(if is_item_selected { " >" } else { "  " }),
                        Span::raw(format!("     {} ", item_checkbox)),
                        Span::raw("    "),  // Indent for hierarchy
                        Span::styled(format!("{:<30}", truncate(&file_name, 30)), item_style),
                        Span::styled(item_size, colors.for_size(item.size)),
                    ])).style(item_style));

                    current_row += 1;
                }
            }
        }
    }

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
        Span::raw(" Nav  "),
        Span::styled("[Enter]", colors.header),
        Span::raw(" Expand  "),
        Span::styled("[Space]", colors.header),
        Span::raw(" Select  "),
        Span::styled("[A]", colors.header),
        Span::raw(" All  "),
        Span::styled("[F]", colors.header),
        Span::raw(" Finder  "),
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

/// Truncate a string to max length with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
    }
}
