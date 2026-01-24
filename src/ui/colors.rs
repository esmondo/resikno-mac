//! Color scheme definitions

use ratatui::style::{Color, Modifier, Style};

/// Color scheme for the application
pub struct ColorScheme {
    /// Critical space hogs (>10 GB)
    pub critical: Style,
    /// Large items (1-10 GB)
    pub large: Style,
    /// Medium items (100 MB - 1 GB)
    pub medium: Style,
    /// Small items (<100 MB)
    pub small: Style,
    /// Safe to clean
    pub safe: Style,
    /// Protected/system files
    pub protected: Style,
    /// Selected item
    pub selected: Style,
    /// Header text
    pub header: Style,
    /// Normal text
    pub normal: Style,
    /// Muted/secondary text
    pub muted: Style,
    /// Success messages
    pub success: Style,
    /// Warning messages
    pub warning: Style,
    /// Error messages
    pub error: Style,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            critical: Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
            large: Style::default()
                .fg(Color::LightRed),
            medium: Style::default()
                .fg(Color::Yellow),
            small: Style::default()
                .fg(Color::Green),
            safe: Style::default()
                .fg(Color::Blue),
            protected: Style::default()
                .fg(Color::DarkGray),
            selected: Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            header: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            normal: Style::default()
                .fg(Color::White),
            muted: Style::default()
                .fg(Color::DarkGray),
            success: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            warning: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            error: Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        }
    }
}

impl ColorScheme {
    /// Get style based on file size in bytes
    pub fn for_size(&self, bytes: u64) -> Style {
        const GB: u64 = 1024 * 1024 * 1024;
        const MB: u64 = 1024 * 1024;

        if bytes > 10 * GB {
            self.critical
        } else if bytes > GB {
            self.large
        } else if bytes > 100 * MB {
            self.medium
        } else {
            self.small
        }
    }

    /// Get color for progress bar based on percentage
    pub fn for_percentage(&self, percent: f64) -> Color {
        if percent > 90.0 {
            Color::Red
        } else if percent > 75.0 {
            Color::LightRed
        } else if percent > 50.0 {
            Color::Yellow
        } else {
            Color::Green
        }
    }
}

/// Create a progress bar string
pub fn progress_bar(percent: f64, width: usize) -> String {
    let filled = (percent / 100.0 * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    format!(
        "[{}{}]",
        "█".repeat(filled),
        "░".repeat(empty)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar() {
        assert_eq!(progress_bar(50.0, 10), "[█████░░░░░]");
        assert_eq!(progress_bar(0.0, 10), "[░░░░░░░░░░]");
        assert_eq!(progress_bar(100.0, 10), "[██████████]");
    }
}
