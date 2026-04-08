//! Color scheme definitions - Claude Code inspired theme

use ratatui::style::{Color, Modifier, Style};

/// Claude Code inspired dark theme
pub struct ColorScheme {
    // Background colors
    pub bg_primary: Color,      // Main background
    pub bg_secondary: Color,    // Secondary/panel background
    pub bg_tertiary: Color,     // Tertiary/selected background
    pub bg_hover: Color,        // Hover state
    
    // Foreground colors
    pub fg_primary: Color,      // Primary text
    pub fg_secondary: Color,    // Secondary/muted text
    pub fg_tertiary: Color,     // Tertiary/hint text
    
    // Accent colors (Claude's subtle palette)
    pub accent_cyan: Color,     // Primary accent (like Claude)
    pub accent_blue: Color,     // Secondary accent
    pub accent_green: Color,    // Success/positive
    pub accent_yellow: Color,   // Warning/caution
    pub accent_orange: Color,   // Attention
    pub accent_red: Color,      // Error/danger
    pub accent_purple: Color,   // Special/interactive
    
    // Semantic colors
    pub safe: Style,
    pub review: Style,
    pub careful: Style,
    pub protected: Style,
    
    // UI states
    pub selected: Style,
    pub selected_bg: Color,
    pub hovered: Style,
    pub active: Style,
    pub inactive: Style,
    
    // Border styles
    pub border_active: Color,
    pub border_inactive: Color,
    pub border_subtle: Color,
}

impl Default for ColorScheme {
    fn default() -> Self {
        // Claude Code-like dark palette
        Self {
            // Backgrounds - subtle gray scale
            bg_primary: Color::Rgb(15, 15, 15),        // Almost black
            bg_secondary: Color::Rgb(28, 28, 28),      // Dark gray
            bg_tertiary: Color::Rgb(38, 38, 38),       // Medium gray
            bg_hover: Color::Rgb(48, 48, 48),          // Lighter gray for hover
            
            // Foregrounds
            fg_primary: Color::Rgb(230, 230, 230),     // Near white
            fg_secondary: Color::Rgb(150, 150, 150),   // Medium gray
            fg_tertiary: Color::Rgb(100, 100, 100),    // Dark gray
            
            // Accents - softer than default terminal colors
            accent_cyan: Color::Rgb(95, 180, 180),     // Muted cyan (Claude-like)
            accent_blue: Color::Rgb(120, 160, 200),    // Soft blue
            accent_green: Color::Rgb(130, 180, 130),   // Soft green
            accent_yellow: Color::Rgb(200, 180, 120),  // Soft yellow
            accent_orange: Color::Rgb(220, 160, 110),  // Soft orange
            accent_red: Color::Rgb(220, 130, 130),     // Soft red
            accent_purple: Color::Rgb(180, 140, 180),  // Soft purple
            
            // Semantic styles
            safe: Style::default()
                .fg(Color::Rgb(130, 180, 130))
                .add_modifier(Modifier::BOLD),
            review: Style::default()
                .fg(Color::Rgb(200, 180, 120))
                .add_modifier(Modifier::BOLD),
            careful: Style::default()
                .fg(Color::Rgb(220, 160, 110))
                .add_modifier(Modifier::BOLD),
            protected: Style::default()
                .fg(Color::Rgb(100, 100, 100)),
            
            // UI states
            selected: Style::default()
                .bg(Color::Rgb(48, 48, 48))
                .fg(Color::Rgb(230, 230, 230))
                .add_modifier(Modifier::BOLD),
            selected_bg: Color::Rgb(48, 48, 48),
            hovered: Style::default()
                .bg(Color::Rgb(38, 38, 38)),
            active: Style::default()
                .fg(Color::Rgb(95, 180, 180))
                .add_modifier(Modifier::BOLD),
            inactive: Style::default()
                .fg(Color::Rgb(100, 100, 100)),
            
            // Borders
            border_active: Color::Rgb(95, 180, 180),
            border_inactive: Color::Rgb(60, 60, 60),
            border_subtle: Color::Rgb(35, 35, 35),
        }
    }
}

impl ColorScheme {
    /// Get style based on file size
    pub fn for_size(&self, bytes: u64) -> Style {
        const GB: u64 = 1024 * 1024 * 1024;
        const MB: u64 = 1024 * 1024;

        if bytes > 10 * GB {
            Style::default().fg(self.accent_red).add_modifier(Modifier::BOLD)
        } else if bytes > GB {
            Style::default().fg(self.accent_orange)
        } else if bytes > 100 * MB {
            Style::default().fg(self.accent_yellow)
        } else {
            Style::default().fg(self.fg_secondary)
        }
    }

    /// Get subtle size color (for secondary display)
    pub fn for_size_subtle(&self, _bytes: u64) -> Style {
        Style::default().fg(self.fg_secondary)
    }

    /// Get accent color for percentage
    pub fn for_percentage(&self, percent: f64) -> Color {
        if percent > 90.0 {
            self.accent_red
        } else if percent > 70.0 {
            self.accent_orange
        } else if percent > 40.0 {
            self.accent_yellow
        } else {
            self.accent_green
        }
    }

    /// Claude-style tool use block background
    pub fn tool_block_style(&self) -> Style {
        Style::default().bg(self.bg_secondary)
    }

    /// Header style (like Claude's assistant label)
    pub fn header_style(&self) -> Style {
        Style::default()
            .fg(self.accent_cyan)
            .add_modifier(Modifier::BOLD)
    }

    /// User input style
    pub fn user_style(&self) -> Style {
        Style::default()
            .fg(self.fg_primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Muted/help text
    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.fg_tertiary)
    }

    /// Success message
    pub fn success_style(&self) -> Style {
        Style::default()
            .fg(self.accent_green)
            .add_modifier(Modifier::BOLD)
    }

    /// Error message
    pub fn error_style(&self) -> Style {
        Style::default()
            .fg(self.accent_red)
            .add_modifier(Modifier::BOLD)
    }
}

/// Create a modern progress bar (Claude-style)
pub fn progress_bar(percent: f64, width: usize, color: Color) -> Vec<Span<'static>> {
    let filled = (percent / 100.0 * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    vec![
        Span::styled(
            "█".repeat(filled),
            Style::default().fg(color),
        ),
        Span::styled(
            "░".repeat(empty),
            Style::default().fg(Color::Rgb(60, 60, 60)),
        ),
    ]
}

/// Create a subtle separator line
pub fn separator(color: Color) -> String {
    "─".repeat(100)
}

use ratatui::text::Span;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar() {
        let colors = ColorScheme::default();
        let bar = progress_bar(50.0, 10, colors.accent_cyan);
        assert_eq!(bar.len(), 2); // filled + empty spans
    }

    #[test]
    fn test_size_coloring() {
        let colors = ColorScheme::default();
        
        // Small file
        let small = colors.for_size(1024 * 1024); // 1 MB
        assert_eq!(small.fg, Some(colors.fg_secondary));
        
        // Large file
        let large = colors.for_size(15 * 1024 * 1024 * 1024); // 15 GB
        assert_eq!(large.fg, Some(colors.accent_red));
    }
}
