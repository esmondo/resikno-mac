# Resikno

> A lightweight, transparent, and safe disk cleanup TUI for macOS

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Why Resikno?

**Resikno** (from Javanese *"resik"* - clean) helps you clean up disk clutter safely.

Unlike other cleanup tools, Resikno is:
- **Beautiful TUI** - Clean, intuitive terminal interface
- **Transparent** - See exactly what will be deleted before any action
- **Safe** - Creates restore points before every cleanup
- **Fast** - Built in Rust with parallel scanning
- **Simple** - Launch and clean, no bloat

## Installation

### Prerequisites
- macOS 10.15+ (Catalina or later)
- Rust 1.75+ (for building from source)

### From Source

```bash
# Clone the repository
git clone https://github.com/esmondo/resikno-mac.git
cd resikno-mac

# Build release binary
cargo build --release

# Install to cargo bin directory (recommended)
cargo install --path .

# Or manually link
ln -sf "$(pwd)/target/release/resikno" ~/.cargo/bin/resikno
```

After installation, run `resikno` from any directory.

## Usage

### Launch the TUI

```bash
resikno
```

This opens the main menu:

```
  ██████╗ ███████╗███████╗██╗██╗  ██╗███╗   ██╗ ██████╗     v0.2.0
  ██╔══██╗██╔════╝██╔════╝██║██║ ██╔╝████╗  ██║██╔═══██╗    Disk Cleanup for macOS
  ██████╔╝█████╗  ███████╗██║█████╔╝ ██╔██╗ ██║██║   ██║
  ██╔══██╗██╔══╝  ╚════██║██║██╔═██╗ ██║╚██╗██║██║   ██║    Safe • Fast • Reversible
  ██║  ██║███████╗███████║██║██║  ██╗██║ ╚████║╚██████╔╝
  ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝

  [1] Scan System       Full scan for all cleanable files
  [2] Quick Scan        Safe items only (recommended)
› [3] Review Results    View previous scan results
  [4] Restore Files     Restore files from trash
  [5] Help              Keyboard shortcuts & guide
  [q] Quit              Exit Resikno

  NAVIGATION
  ↑↓ or jk  Navigate  •  Space  Select  •  Enter  Expand
  c  Clean  •  a  Select All  •  f  Finder  •  m  Menu  •  q  Quit
```

### Navigation

| Key | Action |
|-----|--------|
| `↑↓` or `jk` | Navigate items |
| `Enter` | Expand/collapse category |
| `Space` | Select/deselect item |
| `a` | Select all items |
| `c` | Clean selected items |
| `f` | Reveal in Finder |
| `m` | Return to menu |
| `q` or `Esc` | Quit |

### One-Shot Commands

For scripting or quick operations:

```bash
# Scan and show results
resikno scan

# Scan only safe items
resikno scan --safe-only

# Clean specific category (dry-run by default)
resikno clean caches
resikno clean logs
resikno clean temp

# Actually delete files (--execute required)
resikno clean --execute
resikno clean --execute --force  # Skip confirmation

# Find large files
resikno analyze --large 500    # Files > 500MB

# Find duplicates
resikno analyze --duplicates

# Restore files from trash
resikno restore --list         # List restore points
resikno restore --latest       # Restore most recent

# Update to latest version
resikno update
```

## What It Scans

| Category | Safety | Description |
|----------|--------|-------------|
| System Caches | SAFE | OS-level cache files |
| App Caches | SAFE | Application cache data |
| Temp Files | SAFE | Temporary files in /tmp, /var/tmp |
| Logs | MOSTLY SAFE | System and app logs |
| iOS Backups | MOSTLY SAFE | iPhone/iPad backups |
| Xcode Data | MOSTLY SAFE | DerivedData, device support |
| Downloads | REVIEW | Large files in ~/Downloads |
| Large Files | REVIEW | Files over size threshold |
| Duplicates | REVIEW | Duplicate file detection |

## Safety Features

1. **Dry-run by default** - Use `--execute` or `c` + `Y` to actually delete
2. **Trash on macOS** - Files are moved to Trash (recoverable), not permanently deleted
3. **Restore points** - Every cleanup creates a restore point with metadata
4. **Protected paths** - Critical system files are never touched
5. **Safety levels** - Items categorized as SAFE, MOSTLY SAFE, REVIEW, CAUTION, or PROTECTED
6. **Confirmation required** - Interactive confirmation unless using `--force`

## Screenshots

### Main Menu
```
  ██████╗ ███████╗███████╗██╗██╗  ██╗███╗   ██╗ ██████╗
  ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝

  [1] Scan System       Full scan for all cleanable files
› [2] Quick Scan        Safe items only (recommended)
  [3] Review Results    View previous scan results
  [4] Restore Files     Restore files from trash
  [5] Help              Keyboard shortcuts & guide
  [q] Quit              Exit Resikno

  NAVIGATION
  ↑↓ or jk  Navigate  •  Space  Select  •  Enter  Expand
```

### Scan Results
```
  RESIKNO  v0.2.0
  33 items found  •  22.1 GB total  •  18.8 GB recoverable

  ◯ ▶ 📦 System Caches     11.0 GB  SAFE
  ◯ ▶ 📦 App Caches         4.8 GB  SAFE
  ◯ ▶ 📋 Logs             502.9 MB  MOSTLY SAFE
  ◯ ▶ 🗑️ Temp Files         2.3 GB  SAFE
  ◯ ▶ 📂 Downloads          3.3 GB  REVIEW

  ↑↓ Navigate • Space Select • Enter Expand • c Clean • m Menu • q Quit
```

### Cleanup Confirmation
```
┌─ Cleanup Confirmation ─────────────────────────────┐
│                                                    │
│  Items:  12 files/folders                          │
│  Size:   2.4 GB                                    │
│                                                    │
│  A restore point will be created                   │
│  before deletion.                                  │
│                                                    │
│  Proceed?   [ Yes ]     [ No ]                     │
│                                                    │
│  ← → to select, Enter to confirm                   │
└────────────────────────────────────────────────────┘
```

## Documentation

- **[User Guide](USER_GUIDE.md)** - Complete step-by-step documentation
- **[Cheatsheet](CHEATSHEET.md)** - Quick reference for commands

## Contributing

Contributions welcome! Please ensure:

```bash
# Code compiles without warnings
cargo build

# All tests pass
cargo test

# Code follows style guidelines
cargo clippy -- -W clippy::all

# Format code
cargo fmt
```

### Development

```bash
# Run in development mode
cargo run

# Run with debug logging
RUST_LOG=debug cargo run

# Build and test release
cargo build --release
./target/release/resikno
```

## Safety Notice

Resikno is designed to be safe, but:
- Always review what will be deleted before confirming
- Files are moved to Trash, not permanently deleted
- Restore points are created automatically
- Review the safety levels (SAFE, MOSTLY SAFE, REVIEW, CAUTION)

**Use at your own risk.** While we take every precaution to prevent data loss, you are responsible for your data.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for the beautiful TUI
- Built with [Rust](https://www.rust-lang.org/) for safety and performance
- Inspired by the need for a transparent, trustworthy disk cleaner

---

Made with ❤️ in Indonesia
