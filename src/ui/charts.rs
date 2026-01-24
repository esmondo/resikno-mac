//! ASCII chart visualizations

use bytesize::ByteSize;

/// Create a horizontal bar chart
pub fn horizontal_bar(label: &str, size: u64, max_size: u64, width: usize) -> String {
    let percent = if max_size > 0 {
        (size as f64 / max_size as f64 * 100.0).min(100.0)
    } else {
        0.0
    };

    let bar_width = width.saturating_sub(30); // Leave room for label and size
    let filled = (percent / 100.0 * bar_width as f64).round() as usize;
    let empty = bar_width.saturating_sub(filled);

    format!(
        "{:<20} [{}{}] {:>8}",
        truncate(label, 20),
        "█".repeat(filled),
        "░".repeat(empty),
        ByteSize(size).to_string()
    )
}

/// Create a vertical bar chart (for terminal display)
pub fn vertical_bars(data: &[(String, u64)], height: usize) -> Vec<String> {
    if data.is_empty() {
        return vec![];
    }

    let max_value = data.iter().map(|(_, v)| *v).max().unwrap_or(1);
    let bar_width = 8;
    let mut lines = vec![String::new(); height + 2]; // +2 for label and value

    for (label, value) in data {
        let bar_height = if max_value > 0 {
            (*value as f64 / max_value as f64 * height as f64).round() as usize
        } else {
            0
        };

        // Build the bar from bottom to top
        for (i, line) in lines.iter_mut().enumerate().take(height) {
            let row_from_bottom = height - 1 - i;
            if row_from_bottom < bar_height {
                line.push_str(&format!("{:^width$}", "███", width = bar_width));
            } else {
                line.push_str(&format!("{:^width$}", "", width = bar_width));
            }
        }

        // Add value
        lines[height].push_str(&format!("{:^width$}", ByteSize(*value).to_string(), width = bar_width));

        // Add label
        lines[height + 1].push_str(&format!("{:^width$}", truncate(label, bar_width), width = bar_width));
    }

    lines
}

/// Create a simple pie chart representation (ASCII)
pub fn pie_chart_legend(data: &[(String, u64)]) -> Vec<String> {
    let total: u64 = data.iter().map(|(_, v)| *v).sum();
    let mut lines = Vec::new();

    let symbols = ['●', '○', '◐', '◑', '◒', '◓'];

    for (i, (label, value)) in data.iter().enumerate() {
        let percent = if total > 0 {
            *value as f64 / total as f64 * 100.0
        } else {
            0.0
        };

        let symbol = symbols[i % symbols.len()];
        lines.push(format!(
            "{} {:>5.1}%  {}  {}",
            symbol,
            percent,
            ByteSize(*value),
            label
        ));
    }

    lines
}

/// Create a treemap-style ASCII visualization
pub fn treemap(data: &[(String, u64)], width: usize, height: usize) -> Vec<String> {
    if data.is_empty() {
        return vec![" ".repeat(width); height];
    }

    let total: u64 = data.iter().map(|(_, v)| *v).sum();
    let mut lines = vec![String::new(); height];

    // Simple horizontal layout
    let mut current_x = 0;

    for (label, value) in data {
        let percent = if total > 0 {
            *value as f64 / total as f64
        } else {
            0.0
        };

        let item_width = ((percent * width as f64).round() as usize).max(1);

        if current_x + item_width > width {
            break;
        }

        // Fill the rectangle
        for (row, line) in lines.iter_mut().enumerate() {
            if row == height / 2 {
                // Middle row: show label
                let display = truncate(label, item_width.saturating_sub(2));
                line.push_str(&format!("[{:^width$}]", display, width = item_width.saturating_sub(2)));
            } else if row == 0 || row == height - 1 {
                // Top/bottom border
                line.push_str(&"─".repeat(item_width));
            } else {
                // Side borders
                line.push_str(&format!("│{}│", " ".repeat(item_width.saturating_sub(2))));
            }
        }

        current_x += item_width;
    }

    // Fill remaining space
    for line in &mut lines {
        while line.chars().count() < width {
            line.push(' ');
        }
    }

    lines
}

/// Truncate a string to fit a maximum width
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horizontal_bar() {
        let bar = horizontal_bar("Test", 500, 1000, 50);
        assert!(bar.contains("Test"));
        assert!(bar.contains("█"));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Hello", 10), "Hello");
        assert_eq!(truncate("Hello World", 8), "Hello...");
    }
}
