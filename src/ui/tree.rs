//! Interactive tree view component - Claude Code inspired design

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
    Frame,
};
use bytesize::ByteSize;

use super::{App, DialogState, ViewState, ScanProgress, MENU_ITEMS, colors::ColorScheme};

/// Loading animation frames (spinner)
const LOADING_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Render the main tree view with Claude Code aesthetic
pub fn render(frame: &mut Frame, app: &App) {
    let colors = ColorScheme::default();
    let area = frame.size();

    // Background fill
    frame.render_widget(
        Block::default().style(Style::default().bg(colors.bg_primary)),
        area,
    );

    // Render based on current view state
    match &app.view_state {
        ViewState::Menu { selected } => {
            render_menu_view(frame, app, &colors, area, *selected);
        }
        ViewState::Scanning { progress } => {
            render_scanning_view(frame, &colors, area, progress);
        }
        ViewState::Results => {
            // Main layout for results view
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(6),   // ASCII art header
                    Constraint::Length(1),   // Status line
                    Constraint::Min(5),      // Main content (scrollable)
                    Constraint::Length(3),   // Command palette
                ])
                .margin(1)
                .split(area);

            render_header(frame, main_layout[0], app, &colors);
            render_status_line(frame, main_layout[1], app, &colors);
            render_content(frame, main_layout[2], app, &colors);
            render_command_palette(frame, main_layout[3], &colors);
        }
    }
    
    // Render dialog on top if active
    render_dialog(frame, app, &colors, area);
}

/// Simple clean header for Resikno
fn render_header(frame: &mut Frame, area: Rect, _app: &App, colors: &ColorScheme) {
    let cyan = colors.accent_cyan;
    let secondary = colors.fg_secondary;
    let tertiary = colors.fg_tertiary;
    
    let header_text = vec![
        Line::from(vec![
            Span::styled("  ██████╗ ███████╗███████╗██╗██╗  ██╗███╗   ██╗ ██████╗ ", Style::default().fg(cyan)),
            Span::styled("     v", Style::default().fg(tertiary)),
            Span::styled(env!("CARGO_PKG_VERSION"), Style::default().fg(tertiary)),
        ]),
        Line::from(vec![
            Span::styled("  ██╔══██╗██╔════╝██╔════╝██║██║ ██╔╝████╗  ██║██╔═══██╗", Style::default().fg(cyan)),
            Span::styled("     Disk Cleanup for macOS", Style::default().fg(secondary)),
        ]),
        Line::from(vec![
            Span::styled("  ██████╔╝█████╗  ███████╗██║█████╔╝ ██╔██╗ ██║██║   ██║", Style::default().fg(cyan)),
        ]),
        Line::from(vec![
            Span::styled("  ██╔══██╗██╔══╝  ╚════██║██║██╔═██╗ ██║╚██╗██║██║   ██║", Style::default().fg(cyan)),
            Span::styled("     Safe • Fast • Reversible", Style::default().fg(tertiary)),
        ]),
        Line::from(vec![
            Span::styled("  ██║  ██║███████╗███████║██║██║  ██╗██║ ╚████║╚██████╔╝", Style::default().fg(cyan)),
        ]),
        Line::from(vec![
            Span::styled("  ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝ ", Style::default().fg(cyan)),
        ]),
    ];

    let header = Paragraph::new(header_text)
        .alignment(Alignment::Left);

    frame.render_widget(header, area);
}

/// Render the main menu view
fn render_menu_view(frame: &mut Frame, _app: &App, colors: &ColorScheme, area: Rect, selected: usize) {
    // Centered layout for menu
    let menu_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),   // Header with padding
            Constraint::Length(10),  // Menu items
            Constraint::Min(2),      // Spacer
            Constraint::Length(6),   // Navigation guide
        ])
        .margin(2)
        .split(area);

    // Render header
    let cyan = colors.accent_cyan;
    let secondary = colors.fg_secondary;
    let tertiary = colors.fg_tertiary;
    
    let header_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  ██████╗ ███████╗███████╗██╗██╗  ██╗███╗   ██╗ ██████╗ ", Style::default().fg(cyan)),
        ]),
        Line::from(vec![
            Span::styled("  ██╔══██╗██╔════╝██╔════╝██║██║ ██╔╝████╗  ██║██╔═══██╗", Style::default().fg(cyan)),
            Span::styled("  Disk Cleanup", Style::default().fg(secondary)),
        ]),
        Line::from(vec![
            Span::styled("  ██████╔╝█████╗  ███████╗██║█████╔╝ ██╔██╗ ██║██║   ██║", Style::default().fg(cyan)),
            Span::styled("  for macOS", Style::default().fg(secondary)),
        ]),
        Line::from(vec![
            Span::styled("  ██╔══██╗██╔══╝  ╚════██║██║██╔═██╗ ██║╚██╗██║██║   ██║", Style::default().fg(cyan)),
            Span::styled("  Safe • Fast • Reversible", Style::default().fg(tertiary)),
        ]),
        Line::from(vec![
            Span::styled("  ██║  ██║███████╗███████║██║██║  ██╗██║ ╚████║╚██████╔╝", Style::default().fg(cyan)),
        ]),
        Line::from(vec![
            Span::styled("  ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝ ", Style::default().fg(cyan)),
        ]),
    ];
    
    frame.render_widget(Paragraph::new(header_text), menu_layout[0]);

    // Render menu items
    let mut menu_lines: Vec<Line> = vec![];
    
    for (idx, (key, label, desc)) in MENU_ITEMS.iter().enumerate() {
        let is_selected = idx == selected;
        
        let key_style = if is_selected {
            Style::default()
                .fg(colors.bg_primary)
                .bg(colors.accent_cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors.accent_cyan)
        };
        
        let label_style = if is_selected {
            Style::default()
                .fg(colors.fg_primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors.fg_primary)
        };
        
        let desc_style = Style::default().fg(colors.fg_tertiary);
        
        // Selection indicator
        let indicator = if is_selected {
            Span::styled("› ", Style::default().fg(colors.accent_cyan))
        } else {
            Span::raw("  ")
        };
        
        menu_lines.push(Line::from(vec![
            indicator,
            Span::styled(format!(" {} ", key), key_style),
            Span::raw("  "),
            Span::styled(format!("{:<12}", label), label_style),
            Span::styled(format!("  {}", desc), desc_style),
        ]));
    }
    
    frame.render_widget(Paragraph::new(menu_lines), menu_layout[1]);

    // Navigation guide
    let guide_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  NAVIGATION", Style::default().fg(colors.accent_cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  ↑↓ or jk  ", Style::default().fg(colors.accent_cyan)),
            Span::styled("Navigate  •  ", Style::default().fg(colors.fg_tertiary)),
            Span::styled("Space", Style::default().fg(colors.accent_cyan)),
            Span::styled("  Select  •  ", Style::default().fg(colors.fg_tertiary)),
            Span::styled("Enter", Style::default().fg(colors.accent_cyan)),
            Span::styled("  Expand", Style::default().fg(colors.fg_tertiary)),
        ]),
        Line::from(vec![
            Span::styled("  c", Style::default().fg(colors.accent_cyan)),
            Span::styled("  Clean  •  ", Style::default().fg(colors.fg_tertiary)),
            Span::styled("a", Style::default().fg(colors.accent_cyan)),
            Span::styled("  Select All  •  ", Style::default().fg(colors.fg_tertiary)),
            Span::styled("f", Style::default().fg(colors.accent_cyan)),
            Span::styled("  Finder  •  ", Style::default().fg(colors.fg_tertiary)),
            Span::styled("m", Style::default().fg(colors.accent_cyan)),
            Span::styled("  Menu  •  ", Style::default().fg(colors.fg_tertiary)),
            Span::styled("q", Style::default().fg(colors.accent_cyan)),
            Span::styled("  Quit", Style::default().fg(colors.fg_tertiary)),
        ]),
    ];
    frame.render_widget(Paragraph::new(guide_lines), menu_layout[3]);
}

/// Render the scanning progress view
fn render_scanning_view(frame: &mut Frame, colors: &ColorScheme, area: Rect, progress: &ScanProgress) {
    // Centered layout
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(35),  // Top spacer
            Constraint::Length(10),      // Content
            Constraint::Percentage(35),  // Bottom spacer
        ])
        .split(area);

    let content_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(layout[1]);

    // Get current spinner frame
    let spinner_frame = LOADING_FRAMES[progress.animation_frame % LOADING_FRAMES.len()];
    
    // Title with spinner
    let title = Line::from(vec![
        Span::styled(format!("  {} ", spinner_frame), 
            Style::default().fg(colors.accent_cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Scanning System...", 
            Style::default().fg(colors.accent_cyan).add_modifier(Modifier::BOLD)),
    ]);
    frame.render_widget(Paragraph::new(title), content_area[0]);

    // Progress bar
    let bar_width = 40;
    let filled = (progress.percent_complete as usize * bar_width / 100).min(bar_width);
    let empty = bar_width - filled;
    
    let progress_bar = Line::from(vec![
        Span::styled("  [", Style::default().fg(colors.fg_secondary)),
        Span::styled("█".repeat(filled), Style::default().fg(colors.accent_cyan)),
        Span::styled("░".repeat(empty), Style::default().fg(colors.fg_tertiary)),
        Span::styled(format!("] {:>3}%", progress.percent_complete), 
            Style::default().fg(colors.fg_secondary)),
    ]);
    frame.render_widget(Paragraph::new(progress_bar), content_area[2]);

    // Current category with animation
    let category = Line::from(vec![
        Span::styled("  Scanning: ", Style::default().fg(colors.fg_secondary)),
        Span::styled(&progress.current_category, Style::default().fg(colors.fg_primary)),
    ]);
    frame.render_widget(Paragraph::new(category), content_area[3]);

    // Stats
    let stats = Line::from(vec![
        Span::styled(format!("  Items found: {}  •  Space: {}", 
            progress.items_found,
            ByteSize(progress.bytes_found)),
            Style::default().fg(colors.fg_tertiary)),
    ]);
    frame.render_widget(Paragraph::new(stats), content_area[4]);

    // Navigation guide
    let guide = Line::from(vec![
        Span::styled("  ↑↓ Navigate • Space Select • Enter Expand • c Clean • m Menu • q Quit", 
            Style::default().fg(colors.fg_tertiary)),
    ]);
    frame.render_widget(Paragraph::new(guide), content_area[7]);

    // Cancel hint
    let hint = Line::from(vec![
        Span::styled("  Press 'q' to cancel scan", Style::default().fg(colors.fg_secondary)),
    ]);
    frame.render_widget(Paragraph::new(hint), content_area[8]);
}

/// Status line showing current operation
fn render_status_line(frame: &mut Frame, area: Rect, app: &App, colors: &ColorScheme) {
    let total_items = app.scan_results.items.len();
    let total_size = ByteSize(app.scan_results.total_size);
    let recoverable = ByteSize(app.scan_results.total_recoverable);
    
    let selected_count = app.selected_items.iter().filter(|&&s| s).count() +
                        app.selected_children.values().filter(|&&s| s).count();

    let status = if selected_count > 0 {
        format!("{} items selected  •  {} to recover", 
            selected_count, 
            ByteSize(app.get_selected_size()))
    } else {
        format!("{} items found  •  {} total  •  {} recoverable", 
            total_items, total_size, recoverable)
    };

    let status_widget = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(status, Style::default().fg(colors.fg_tertiary)),
    ]));

    frame.render_widget(status_widget, area);
}

/// Main content area with tool-like blocks for each category
fn render_content(frame: &mut Frame, area: Rect, app: &App, colors: &ColorScheme) {
    let mut items: Vec<ListItem> = Vec::new();
    let mut current_row = 0;

    for (cat_idx, cat) in app.categories.iter().enumerate() {
        let is_selected = app.selected_index == current_row;
        let is_expanded = app.expanded.get(cat_idx).copied().unwrap_or(false);

        // Check selection state
        let all_items_selected = cat.item_indices.iter()
            .all(|&i| app.selected_items.get(i).copied().unwrap_or(false));
        let some_items_selected = cat.item_indices.iter()
            .any(|&i| app.selected_items.get(i).copied().unwrap_or(false));

        // Build category row - Claude Code "tool use" style block
        let category_item = build_category_row(
            cat, 
            is_selected, 
            is_expanded,
            all_items_selected,
            some_items_selected,
            colors,
        );
        items.push(category_item);
        current_row += 1;

        // Child items when expanded
        if is_expanded {
            for &item_idx in &cat.item_indices {
                let is_item_selected = app.selected_index == current_row;
                let is_checked = app.selected_items.get(item_idx).copied().unwrap_or(false);
                let is_item_expanded = app.is_item_expanded(item_idx);

                if let Some(item) = app.scan_results.items.get(item_idx) {
                    let child_item = build_child_row(
                        item,
                        is_item_selected,
                        is_checked,
                        is_item_expanded,
                        colors,
                    );
                    items.push(child_item);
                    current_row += 1;

                    // Grandchildren if expanded
                    if is_item_expanded {
                        if let Some(children) = app.expanded_items.get(&item_idx) {
                            for child in children {
                                let is_grandchild_selected = app.selected_index == current_row;
                                let is_grandchild_checked = app.selected_children
                                    .get(&child.path)
                                    .copied()
                                    .unwrap_or(false);

                                let grandchild_item = build_grandchild_row(
                                    child,
                                    is_grandchild_selected,
                                    is_grandchild_checked,
                                    colors,
                                );
                                items.push(grandchild_item);
                                current_row += 1;
                            }
                        }
                    }
                }
            }
            
            // Add spacing after expanded category
            items.push(ListItem::new(""));
        }
    }

    // Apply scroll offset - only render visible items
    let viewport_height = area.height as usize;
    let visible_items: Vec<ListItem> = items
        .into_iter()
        .skip(app.scroll_offset)
        .take(viewport_height)
        .collect();

    let list = List::new(visible_items)
        .style(Style::default().bg(colors.bg_primary));

    frame.render_widget(list, area);
}

/// Build category row with tool-like styling
fn build_category_row<'a>(
    cat: &'a super::CategoryView,
    is_selected: bool,
    is_expanded: bool,
    all_selected: bool,
    some_selected: bool,
    colors: &'a ColorScheme,
) -> ListItem<'a> {
    use crate::scanner::SafetyLevel;
    
    // Expansion indicator
    let expand = if is_expanded {
        Span::styled("▼ ", Style::default().fg(colors.fg_tertiary))
    } else {
        Span::styled("▶ ", Style::default().fg(colors.fg_tertiary))
    };

    // Checkbox state
    let checkbox = if all_selected {
        Span::styled("✓ ", Style::default().fg(colors.accent_cyan))
    } else if some_selected {
        Span::styled("◐ ", Style::default().fg(colors.accent_cyan))
    } else {
        Span::styled("◯ ", Style::default().fg(colors.fg_tertiary))
    };

    // Category icon
    let icon = Span::styled(
        format!("{} ", cat.category.icon()),
        Style::default().fg(colors.accent_cyan),
    );

    // Category name
    let name_style = if is_selected {
        colors.selected
    } else {
        Style::default().fg(colors.fg_primary)
    };
    let name = Span::styled(cat.category.name(), name_style);

    // Size info
    let size_str = format_size_aligned(cat.total_size);
    let size = Span::styled(size_str, colors.for_size_subtle(cat.total_size));

    // Safety pill (Claude tool-like)
    let safety = cat.category.safety_level();
    let safety_pill = safety_badge(safety, colors);

    let mut spans = vec![
        checkbox,
        expand,
        icon,
        name,
        Span::raw("  "),
        size,
        Span::raw("  "),
        safety_pill,
    ];

    // Selection indicator (›)
    if is_selected {
        spans.insert(0, Span::styled(" › ", 
            Style::default().fg(colors.accent_cyan)));
    } else {
        spans.insert(0, Span::raw("   "));
    }

    ListItem::new(Line::from(spans))
        .style(if is_selected { 
            Style::default().bg(colors.selected_bg) 
        } else { 
            Style::default() 
        })
}

/// Build child row (individual items)
fn build_child_row<'a>(
    item: &'a crate::scanner::ScannedItem,
    is_selected: bool,
    is_checked: bool,
    is_expanded: bool,
    colors: &'a ColorScheme,
) -> ListItem<'a> {
    let checkbox = if is_checked {
        Span::styled("✓ ", Style::default().fg(colors.accent_cyan))
    } else {
        Span::styled("◯ ", Style::default().fg(colors.fg_tertiary))
    };

    // Expansion indicator for directories
    let expand = if true {
        if is_expanded {
            Span::styled("▼ ", Style::default().fg(colors.fg_tertiary))
        } else {
            Span::styled("▶ ", Style::default().fg(colors.fg_tertiary))
        }
    } else {
        Span::raw("  ")
    };

    // File name
    let file_name = item.path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| item.path.display().to_string());

    let name_style = if is_selected {
        colors.selected
    } else {
        Style::default().fg(colors.fg_secondary)
    };

    // Size
    let size_str = format_size_aligned(item.size);
    let size = Span::styled(size_str, colors.for_size_subtle(item.size));

    let mut spans = vec![
        checkbox,
        expand,
        Span::styled(truncate(&file_name, 40), name_style),
        Span::raw("  "),
        size,
    ];

    // Selection indicator
    if is_selected {
        spans.insert(0, Span::styled("  › ", 
            Style::default().fg(colors.accent_cyan)));
    } else {
        spans.insert(0, Span::raw("    "));
    }

    ListItem::new(Line::from(spans))
        .style(if is_selected { 
            Style::default().bg(colors.selected_bg) 
        } else { 
            Style::default() 
        })
}

/// Build grandchild row (deepest level)
fn build_grandchild_row<'a>(
    child: &'a crate::scanner::ChildEntry,
    is_selected: bool,
    is_checked: bool,
    colors: &'a ColorScheme,
) -> ListItem<'a> {
    let checkbox = if is_checked {
        Span::styled("✓ ", Style::default().fg(colors.accent_cyan))
    } else {
        Span::styled("◯ ", Style::default().fg(colors.fg_tertiary))
    };

    // File name with deeper indent
    let file_name = child.path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| child.path.display().to_string());

    let name_style = if is_selected {
        colors.selected
    } else {
        Style::default().fg(colors.fg_tertiary)
    };

    let size_str = format_size_aligned(child.size);
    let size = Span::styled(size_str, colors.for_size_subtle(child.size));

    let mut spans = vec![
        checkbox,
        Span::raw("    "),  // Deep indent
        Span::styled(truncate(&file_name, 35), name_style),
        Span::raw("  "),
        size,
    ];

    if is_selected {
        spans.insert(0, Span::styled("  › ", 
            Style::default().fg(colors.accent_cyan)));
    } else {
        spans.insert(0, Span::raw("    "));
    }

    ListItem::new(Line::from(spans))
        .style(if is_selected { 
            Style::default().bg(colors.selected_bg) 
        } else { 
            Style::default() 
        })
}

/// Command palette footer - Claude Code style
fn render_command_palette(frame: &mut Frame, area: Rect, colors: &ColorScheme) {
    let commands = vec![
        ("↑↓/jk", "Navigate"),
        ("Space", "Select"),
        ("Enter", "Expand"),
        ("c", "Clean"),
        ("a", "All"),
        ("f", "Finder"),
        ("m", "Menu"),
        ("q", "Quit"),
    ];

    let mut spans = vec![Span::raw("  ")];
    
    for (i, (key, desc)) in commands.iter().enumerate() {
        spans.push(Span::styled(
            *key,
            Style::default()
                .fg(colors.accent_cyan)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {}", desc),
            Style::default().fg(colors.fg_tertiary),
        ));
        
        if i < commands.len() - 1 {
            spans.push(Span::styled("  ·  ", Style::default().fg(colors.fg_tertiary)));
        }
    }

    let palette = Paragraph::new(Line::from(spans));
    frame.render_widget(palette, area);
}

/// Render dialog modal on top of content
fn render_dialog(frame: &mut Frame, app: &App, colors: &ColorScheme, area: Rect) {
    match &app.dialog {
        DialogState::ConfirmCleanup { items, total_size } => {
            render_confirm_dialog(frame, colors, area, items.len(), *total_size, app.dialog_button_selected);
        }
        DialogState::CleanupRunning => {
            render_running_dialog(frame, colors, area);
        }
        DialogState::CleanupResult { success, message } => {
            render_result_dialog(frame, colors, area, *success, message);
        }
        DialogState::None => {}
    }
}

/// Render cleanup confirmation dialog
fn render_confirm_dialog(
    frame: &mut Frame, 
    colors: &ColorScheme, 
    area: Rect,
    item_count: usize,
    total_size: u64,
    button_selected: usize,
) {
    let dialog_width = 50;
    let dialog_height = 13;
    
    // Center the dialog
    let dialog_area = centered_rect(dialog_width, dialog_height, area);
    
    // Clear background
    frame.render_widget(Clear, dialog_area);
    
    // Dialog border
    let block = Block::default()
        .title(" Cleanup Confirmation ")
        .title_style(Style::default().fg(colors.accent_cyan).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.accent_cyan))
        .style(Style::default().bg(colors.bg_secondary));
    
    frame.render_widget(block.clone(), dialog_area);
    
    // Inner content area
    let inner = block.inner(dialog_area);
    
    // Button styles based on selection
    let yes_style = if button_selected == 0 {
        Style::default()
            .fg(colors.bg_primary)
            .bg(colors.accent_green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(colors.accent_green)
    };
    
    let no_style = if button_selected == 1 {
        Style::default()
            .fg(colors.bg_primary)
            .bg(colors.accent_red)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(colors.accent_red)
    };
    
    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Items:  ", Style::default().fg(colors.fg_secondary)),
            Span::styled(format!("{} files/folders", item_count), Style::default().fg(colors.fg_primary)),
        ]),
        Line::from(vec![
            Span::styled("  Size:   ", Style::default().fg(colors.fg_secondary)),
            Span::styled(ByteSize(total_size).to_string(), Style::default().fg(colors.fg_primary)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  A restore point will be created", 
            Style::default().fg(colors.fg_tertiary)
        )),
        Line::from("  before deletion."),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Proceed?  ", Style::default().fg(colors.fg_primary)),
            Span::styled(" Yes ", yes_style),
            Span::raw("     "),
            Span::styled(" No ", no_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ← → to select, Enter to confirm", Style::default().fg(colors.fg_tertiary)),
        ]),
    ];
    
    let paragraph = Paragraph::new(content);
    frame.render_widget(paragraph, inner);
}

/// Render cleanup running dialog
fn render_running_dialog(frame: &mut Frame, colors: &ColorScheme, area: Rect) {
    let dialog_width = 40;
    let dialog_height = 5;
    
    let dialog_area = centered_rect(dialog_width, dialog_height, area);
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.accent_cyan))
        .style(Style::default().bg(colors.bg_secondary));
    
    frame.render_widget(block.clone(), dialog_area);
    
    let inner = block.inner(dialog_area);
    
    let content = Line::from(vec![
        Span::styled("  Creating restore point...", Style::default().fg(colors.fg_primary)),
    ]);
    
    let paragraph = Paragraph::new(content).alignment(Alignment::Center);
    frame.render_widget(paragraph, inner);
}

/// Render cleanup result dialog
fn render_result_dialog(
    frame: &mut Frame, 
    colors: &ColorScheme, 
    area: Rect,
    success: bool,
    message: &str,
) {
    let dialog_width = 50;
    let dialog_height = 10;
    
    let dialog_area = centered_rect(dialog_width, dialog_height, area);
    frame.render_widget(Clear, dialog_area);
    
    let (title, title_color) = if success {
        (" ✅ Cleanup Complete ", colors.accent_green)
    } else {
        (" ❌ Cleanup Failed ", colors.accent_red)
    };
    
    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(title_color).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(title_color))
        .style(Style::default().bg(colors.bg_secondary));
    
    frame.render_widget(block.clone(), dialog_area);
    
    let inner = block.inner(dialog_area);
    
    // Parse message into lines
    let lines: Vec<Line> = message
        .lines()
        .map(|line| {
            Line::from(vec![
                Span::raw("  "),
                Span::styled(line.to_string(), Style::default().fg(colors.fg_primary)),
            ])
        })
        .chain(std::iter::once(Line::from("")))
        .chain(std::iter::once(Line::from(vec![
            Span::raw("  Press "),
            Span::styled("any key", Style::default().fg(colors.accent_cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" to continue"),
        ])))
        .collect();
    
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Create a centered rectangle
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    Rect::new(
        area.x + x,
        area.y + y,
        width.min(area.width),
        height.min(area.height),
    )
}

// Helper functions

/// Format size with alignment
fn format_size_aligned(size: u64) -> String {
    let bs = ByteSize(size);
    format!("{:>8}", bs)
}

/// Create safety level badge
fn safety_badge(level: crate::scanner::SafetyLevel, colors: &ColorScheme) -> Span {
    use crate::scanner::SafetyLevel;
    
    let (text, style) = match level {
        SafetyLevel::Safe => (
            " SAFE ",
            Style::default()
                .fg(colors.bg_primary)
                .bg(Color::Rgb(130, 180, 130))
                .add_modifier(Modifier::BOLD),
        ),
        SafetyLevel::MostlySafe => (
            " MOSTLY SAFE ",
            Style::default()
                .fg(colors.bg_primary)
                .bg(Color::Rgb(200, 180, 120)),
        ),
        SafetyLevel::ReviewCarefully => (
            " REVIEW ",
            Style::default()
                .fg(colors.bg_primary)
                .bg(Color::Rgb(220, 160, 110)),
        ),
        SafetyLevel::Caution => (
            " CAUTION ",
            Style::default()
                .fg(colors.bg_primary)
                .bg(Color::Rgb(220, 130, 130)),
        ),
        SafetyLevel::Protected => (
            " PROTECTED ",
            Style::default()
                .fg(colors.fg_primary)
                .bg(colors.bg_tertiary),
        ),
    };
    
    Span::styled(text, style)
}

/// Truncate string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else if max_len > 3 {
        let truncated: String = s.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    } else {
        s.chars().take(max_len).collect()
    }
}
