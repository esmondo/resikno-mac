# Resikno-Mac

> A lightweight, transparent, and safe disk cleanup CLI tool for macOS

## Project Overview

Resikno-Mac is a CleanMyMac alternative built in Rust. It focuses on:
- **Transparency**: Always show what will be deleted before acting
- **Safety**: Create restore points before every cleanup
- **Simplicity**: CLI-first with beautiful TUI, no bloat
- **Cross-platform**: macOS first, then Windows/Linux

## Quick Commands

```bash
# Build and run
cargo build
cargo run -- scan

# Run tests
cargo test

# Build release
cargo build --release

# Run with logging
RUST_LOG=debug cargo run -- scan
```

## Project Structure

```
src/
├── main.rs             # Entry point, CLI setup
├── cli/                # Command-line interface
│   ├── mod.rs          # CLI module
│   ├── commands.rs     # Command implementations
│   └── args.rs         # Argument definitions (clap)
├── scanner/            # Disk scanning engine
│   ├── mod.rs          # Scanner module
│   ├── cache.rs        # Cache directory detection
│   ├── duplicates.rs   # Duplicate file finder
│   └── large_files.rs  # Large/old file detection
├── cleaner/            # Cleanup operations
│   ├── mod.rs          # Cleaner module
│   ├── backup.rs       # Restore point creation
│   └── delete.rs       # Safe deletion logic
├── ui/                 # Terminal UI
│   ├── mod.rs          # UI module
│   ├── tree.rs         # Interactive tree view
│   ├── colors.rs       # Color scheme definitions
│   └── charts.rs       # ASCII visualizations
└── platform/           # Platform-specific code
    ├── mod.rs          # Platform detection
    ├── macos.rs        # macOS paths & APIs
    ├── linux.rs        # Linux paths (future)
    └── windows.rs      # Windows paths (future)
```

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `ratatui` | Terminal UI framework |
| `crossterm` | Cross-platform terminal control |
| `walkdir` | Fast directory traversal |
| `sha2` | File hashing for duplicates |
| `rayon` | Parallel processing |
| `serde` | Serialization |
| `anyhow` | Error handling |
| `bytesize` | Human-readable file sizes |

## Coding Guidelines

### Rust Style
- Use `rustfmt` for formatting (default settings)
- Use `clippy` for linting: `cargo clippy -- -W clippy::all`
- Prefer `anyhow::Result` for error handling in binaries
- Use `thiserror` for custom error types in library code

### Safety Principles
1. **Never delete without confirmation** - Always require explicit user action
2. **Create restore points first** - Before any destructive operation
3. **Dry-run by default** - `--execute` flag required for actual deletion
4. **Log everything** - Maintain audit trail in `~/.resikno-mac/logs/`

### Platform Abstraction
```rust
// Use the PlatformPaths trait for cross-platform code
use crate::platform::PlatformPaths;

fn get_cache_dirs() -> Vec<PathBuf> {
    platform::current().cache_dirs()
}
```

### TUI Guidelines
- Use consistent color scheme from `ui/colors.rs`
- All interactive elements must have keyboard shortcuts
- Show loading states for long operations
- Always provide escape/back navigation

## File Safety Categories

| Category | Safety Level | Action |
|----------|--------------|--------|
| System caches | 🔵 SAFE | Auto-cleanable |
| App caches | 🔵 SAFE | Auto-cleanable |
| Logs (>30 days) | 🟢 MOSTLY SAFE | Review suggested |
| Duplicates | 🟡 REVIEW | Manual confirmation |
| User files | 🔴 CAUTION | Never auto-clean |
| System files | ⚪ PROTECTED | Never touch |

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

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_cache_detection

# Integration tests
cargo test --test integration
```

## Current Implementation Status

### Phase 1: Foundation ⬜ In Progress
- [x] Project structure
- [ ] Basic CLI with clap
- [ ] Platform abstraction layer
- [ ] macOS path detection

### Phase 2: Scanning Engine ⬜ Not Started
- [ ] Directory traversal
- [ ] Size calculation
- [ ] Category classification
- [ ] Duplicate detection

### Phase 3: Terminal UI ⬜ Not Started
- [ ] Basic ratatui setup
- [ ] Interactive tree view
- [ ] Color scheme
- [ ] Progress bars

### Phase 4: Cleanup & Safety ⬜ Not Started
- [ ] Restore point creation
- [ ] Safe deletion
- [ ] Manifest logging
- [ ] Restore functionality

### Phase 5: Polish ⬜ Not Started
- [ ] Error handling
- [ ] Performance optimization
- [ ] Documentation
- [ ] Release binaries

## PRD Reference

Full PRD is available at: `~/.claude/plans/snug-popping-plum.md`
