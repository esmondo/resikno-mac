//! Welcome screen with gradient banner for Resikno
//!
//! Displays a clean, modern welcome banner with gradient colors.

use console::Style;

/// Gradient color palette: Cyan → Blue → Purple
const GRADIENT: &[(u8, u8, u8)] = &[
    (0, 255, 255),   // Cyan
    (50, 220, 255),  // Light cyan
    (100, 180, 255), // Light blue
    (150, 140, 255), // Blue-purple
    (200, 100, 255), // Purple
];

/// The ASCII art banner for RESIKNO
const BANNER_LINES: &[&str] = &[
    "░█▀▀█ ░█▀▀▀ ░█▀▀▀█ ▀█▀ ░█─▄▀ ░█▄─░█ ░█▀▀▀█",
    "░█▄▄▀ ░█▀▀▀ ─▀▀▀▄▄ ░█─ ░█▀▄─ ░█░█░█ ░█──░█",
    "░█─░█ ░█▄▄▄ ░█▄▄▄█ ▄█▄ ░█─░█ ░█──▀█ ░█▄▄▄█",
];

/// Interpolate between two colors
fn lerp_color(c1: (u8, u8, u8), c2: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let r = (f32::from(c1.0) + (f32::from(c2.0) - f32::from(c1.0)) * t) as u8;
    let g = (f32::from(c1.1) + (f32::from(c2.1) - f32::from(c1.1)) * t) as u8;
    let b = (f32::from(c1.2) + (f32::from(c2.2) - f32::from(c1.2)) * t) as u8;
    (r, g, b)
}

/// Get gradient color for a position (0.0 to 1.0)
fn gradient_color(position: f32) -> (u8, u8, u8) {
    let position = position.clamp(0.0, 1.0);
    let segment_count = GRADIENT.len() - 1;
    let scaled = position * segment_count as f32;
    let index = (scaled as usize).min(segment_count - 1);
    let t = scaled - index as f32;
    lerp_color(GRADIENT[index], GRADIENT[index + 1], t)
}

/// Convert RGB to closest ANSI 256 color
fn to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    // Use the 216 color cube (colors 16-231)
    // Each channel has 6 levels: 0, 95, 135, 175, 215, 255
    fn to_cube(v: u8) -> u8 {
        if v < 48 {
            0
        } else if v < 115 {
            1
        } else if v < 155 {
            2
        } else if v < 195 {
            3
        } else if v < 235 {
            4
        } else {
            5
        }
    }

    16 + 36 * to_cube(r) + 6 * to_cube(g) + to_cube(b)
}

/// Display the welcome banner
pub fn display() {
    let gray = Style::new().color256(243);
    let dim = Style::new().dim();
    let cyan = Style::new().cyan();
    let white = Style::new().white();

    // Top border
    println!();
    println!(
        "    {}",
        gray.apply_to("╭──────────────────────────────────────────────────────────╮")
    );
    println!("    {}                                                          {}", gray.apply_to("│"), gray.apply_to("│"));

    // Banner lines with gradient
    for line in BANNER_LINES {
        print!("    {}               ", gray.apply_to("│"));
        print_gradient_inline(line);
        println!(" {}", gray.apply_to("│"));
    }

    println!("    {}                                                          {}", gray.apply_to("│"), gray.apply_to("│"));

    // Tagline - center manually
    let tagline = "Lightweight Disk Cleanup for macOS";
    let tag_pad = (58 - tagline.len()) / 2;
    let tag_pad_right = 58 - tag_pad - tagline.len();
    print!("    {}", gray.apply_to("│"));
    print!("{:width$}", "", width = tag_pad);
    print!("{}", dim.apply_to(tagline));
    print!("{:width$}", "", width = tag_pad_right);
    println!("{}", gray.apply_to("│"));

    // Version - center manually
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    let ver_pad = (58 - version.len()) / 2;
    let ver_pad_right = 58 - ver_pad - version.len();
    print!("    {}", gray.apply_to("│"));
    print!("{:width$}", "", width = ver_pad);
    print!("{}", dim.apply_to(&version));
    print!("{:width$}", "", width = ver_pad_right);
    println!("{}", gray.apply_to("│"));

    println!("    {}                                                          {}", gray.apply_to("│"), gray.apply_to("│"));

    // Bottom border
    println!(
        "    {}",
        gray.apply_to("╰──────────────────────────────────────────────────────────╯")
    );
    println!();

    // Tips section
    println!("  {}:", white.apply_to("Tips"));
    println!(
        "  {}. Type '{}' to find cleanable files",
        cyan.apply_to("1"),
        cyan.apply_to("scan")
    );
    println!(
        "  {}. Type '{}' for all commands",
        cyan.apply_to("2"),
        cyan.apply_to("help")
    );
    println!(
        "  {}. Press {} or type '{}' to quit",
        cyan.apply_to("3"),
        dim.apply_to("Ctrl+D"),
        cyan.apply_to("exit")
    );
    println!();
}

/// Print gradient text inline (no newline)
fn print_gradient_inline(text: &str) {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    if len == 0 {
        return;
    }

    for (i, ch) in chars.iter().enumerate() {
        let position = i as f32 / (len - 1).max(1) as f32;
        let (r, g, b) = gradient_color(position);
        let style = Style::new().color256(to_ansi256(r, g, b));
        print!("{}", style.apply_to(ch));
    }
}
