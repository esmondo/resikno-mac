//! Welcome screen - Claude Code inspired minimal design

use console::Style;

/// Display the welcome banner - clean, minimal, Claude Code style
pub fn display() {
    let dim = Style::new().dim();
    let cyan = Style::new().cyan();  // Use standard cyan
    let white = Style::new().white();
    let gray = Style::new().color256(244); // Gray

    // Clean, minimal header like Claude Code
    println!();
    println!("  {} {}", 
        cyan.apply_to("◉"),
        white.apply_to("Resikno")
    );
    println!("  {}", dim.apply_to("Disk Cleanup for macOS"));
    println!("  {}", gray.apply_to(format!("v{}", env!("CARGO_PKG_VERSION"))));
    println!();

    // Quick start tips - organized like Claude's suggestions
    println!("  {}", gray.apply_to("Quick start:"));
    println!();
    println!("  {}  {}", 
        cyan.apply_to("scan"),
        dim.apply_to("Scan for cleanable files")
    );
    println!("  {} {}", 
        cyan.apply_to("clean caches --execute"),
        dim.apply_to("Clean cache files")
    );
    println!("  {}  {}", 
        cyan.apply_to("help"),
        dim.apply_to("Show all commands")
    );
    println!();
    println!("  {} {} {}", 
        dim.apply_to("Press"),
        cyan.apply_to("Ctrl+D"),
        dim.apply_to("or type 'exit' to quit")
    );
    println!();
}

/// Display compact welcome for returning users
pub fn display_compact() {
    let dim = Style::new().dim();
    let cyan = Style::new().cyan();
    
    println!();
    println!("  {} {}  {}", 
        cyan.apply_to("◉"),
        "Resikno",
        dim.apply_to(format!("v{}", env!("CARGO_PKG_VERSION")))
    );
    println!("  {} {} {}", 
        dim.apply_to("Type"),
        cyan.apply_to("help"),
        dim.apply_to("for commands, or")
    );
    println!("  {} {}", 
        cyan.apply_to("scan"),
        dim.apply_to("to find cleanable files")
    );
    println!();
}
