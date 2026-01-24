# Resikno

> A lightweight, transparent, and safe disk cleanup CLI for macOS

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Why Resikno?

**Resikno** (from Esperanto *"resiknigi"* - to resign/let go) helps you let go of disk clutter safely.

Unlike other cleanup tools, Resikno is:
- **Transparent** - See exactly what will be deleted before any action
- **Safe** - Creates restore points before every cleanup
- **Fast** - Built in Rust with parallel scanning
- **Simple** - CLI-first with a beautiful TUI, no bloat

## Installation

```bash
# From source (requires Rust 1.75+)
cargo install --git https://github.com/esmondo/resikno-mac.git

# Or clone and build
git clone https://github.com/esmondo/resikno-mac.git
cd resikno-mac
cargo install --path .
```

## Usage

```bash
# Scan and open interactive TUI
resikno scan

# Scan with custom minimum file size (default: 50MB)
resikno scan --min-size 100    # Only show files >= 100MB
resikno scan -m 10             # Show files >= 10MB

# Non-interactive scan (just show results)
resikno scan --no-interactive

# Clean specific categories
resikno clean caches           # Clean cache files
resikno clean --safe-only      # Only clean SAFE items
resikno clean --execute        # Actually delete (default is dry-run)

# Analyze disk
resikno analyze --large 500    # Find files > 500MB
resikno analyze --duplicates   # Find duplicate files

# Manage restore points
resikno restore --list         # List restore points
resikno restore --latest       # Restore most recent cleanup

# Update to latest version
resikno update
resikno update --check         # Check without installing
```

## TUI Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑↓` / `jk` | Navigate |
| `Enter` | Expand/collapse category |
| `Space` | Select/deselect item |
| `A` | Select all |
| `F` | Reveal in Finder |
| `C` | Clean selected items |
| `Q` / `Esc` | Quit |

## What It Scans

| Category | Safety | Description |
|----------|--------|-------------|
| System Caches | SAFE | OS-level cache files |
| App Caches | SAFE | Application cache data |
| Temp Files | SAFE | Temporary files in /tmp, /var/tmp |
| Logs | REVIEW | System and app logs |
| iOS Backups | REVIEW | iPhone/iPad backups |
| Xcode Data | REVIEW | DerivedData, device support |
| Downloads | CAREFUL | Large files in ~/Downloads |

## Safety Features

1. **Dry-run by default** - Use `--execute` to actually delete
2. **Restore points** - Every cleanup creates a restore point
3. **Protected paths** - Critical system files are never touched
4. **Confirmation required** - Interactive confirmation for risky items

## Screenshots

```
┌─ Disk Cleaner ──────────────────────────────────────────────────────┐
│ RESIKNO  Found 22.1 GB (18.8 GB recoverable) in 33 items           │
├─────────────────────────────────────────────────────────────────────┤
│ > [ ] ▶ 📦 System Caches     11.0 GB  SAFE     [████████████░░░]   │
│   [ ] ▶ 📦 App Caches         4.8 GB  SAFE     [█████░░░░░░░░░░]   │
│   [ ] ▶ 📋 Logs             502.9 MB  REVIEW   [█░░░░░░░░░░░░░░]   │
│   [ ] ▶ 🗑️ Temp Files         2.3 GB  SAFE     [███░░░░░░░░░░░░]   │
│   [ ] ▶ 📂 Downloads          3.3 GB  CAREFUL  [████░░░░░░░░░░░]   │
├─────────────────────────────────────────────────────────────────────┤
│ [↑↓] Nav  [Enter] Expand  [Space] Select  [A] All  [F] Finder  [Q] │
└─────────────────────────────────────────────────────────────────────┘
```

## Contributing

Contributions welcome! Please read the contributing guidelines first.

```bash
# Development
cargo build
cargo test
cargo clippy -- -W clippy::all

# Run with debug logging
RUST_LOG=debug cargo run -- scan
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for the TUI
- Inspired by the need for a transparent, trustworthy disk cleaner
