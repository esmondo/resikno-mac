# Resikno

> A lightweight, transparent, and safe disk cleanup CLI tool for macOS

## Features

- **Transparency**: Always shows what will be deleted before acting
- **Safety**: Creates restore points before every cleanup
- **Simplicity**: CLI-first with beautiful TUI, no bloat
- **Cross-platform**: macOS first, Windows/Linux planned

## Installation

### Homebrew (Recommended)

```bash
brew install --formula https://raw.githubusercontent.com/esmondo/resikno-mac/main/Formula/resikno.rb
```

### Quick Install Script

```bash
curl -sSL https://raw.githubusercontent.com/esmondo/resikno-mac/main/install.sh | bash
```

### From Source

```bash
# Requires Rust (install from https://rustup.rs)
cargo install --git https://github.com/esmondo/resikno-mac.git
```

### Manual Build

```bash
git clone https://github.com/esmondo/resikno-mac.git
cd resikno-mac
cargo install --path .
```

After installation, verify it works:

```bash
resikno --help
```

## Usage

```bash
# Scan your system for cleanable files
resikno scan

# Clean specific category (dry-run by default)
resikno clean caches

# Actually delete files (requires --execute flag)
resikno clean caches --execute

# Interactive TUI mode
resikno
```

## Safety First

Resikno is designed to never delete anything important:

| Category | Safety Level | Action |
|----------|--------------|--------|
| System caches | SAFE | Auto-cleanable |
| App caches | SAFE | Auto-cleanable |
| Logs (>30 days) | MOSTLY SAFE | Review suggested |
| Duplicates | REVIEW | Manual confirmation |
| User files | CAUTION | Never auto-clean |
| System files | PROTECTED | Never touch |

### Protected Paths (Never Touched)

- System directories: `/System`, `/usr`, `/bin`, `/sbin`
- User data: `~/Documents`, `~/Desktop`, `~/Pictures`, `~/Music`
- Credentials: `~/.ssh`, `~/.gnupg`, `~/.aws`, `~/.config`
- Application data: `~/Library/Application Support`

## Project Structure

```
src/
├── main.rs             # Entry point, CLI setup
├── cli/                # Command-line interface
│   ├── commands.rs     # Command implementations
│   └── args.rs         # Argument definitions (clap)
├── scanner/            # Disk scanning engine
│   ├── cache.rs        # Cache directory detection
│   ├── duplicates.rs   # Duplicate file finder
│   └── large_files.rs  # Large/old file detection
├── cleaner/            # Cleanup operations
│   ├── backup.rs       # Restore point creation
│   └── delete.rs       # Safe deletion logic
├── ui/                 # Terminal UI
│   ├── tree.rs         # Interactive tree view
│   ├── colors.rs       # Color scheme definitions
│   └── charts.rs       # ASCII visualizations
└── platform/           # Platform-specific code
    ├── macos.rs        # macOS paths & APIs
    ├── linux.rs        # Linux paths (future)
    └── windows.rs      # Windows paths (future)
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run -- scan

# Build release
cargo build --release
```

## Data Locations

```
~/.resikno-mac/
├── config.toml         # User configuration
├── restore/            # Restore points
│   └── YYYY-MM-DD_HHMMSS/
│       ├── manifest.json
│       └── metadata.json
└── logs/
    └── cleanup.log     # Audit trail
```

## Why "Resikno"?

From Javanese: *resik* means "clean", *resikno* is the imperative form - "clean it up!" A fitting name for a disk cleanup tool.

## License

MIT
