# Resikno-Mac User Guide

> Complete step-by-step documentation for the safe disk cleanup tool

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [Interactive Shell Mode](#interactive-shell-mode)
4. [Command Reference](#command-reference)
5. [Safety Features](#safety-features)
6. [TUI (Terminal UI) Guide](#tui-terminal-ui-guide)
7. [Scenarios & Examples](#scenarios--examples)
8. [Troubleshooting](#troubleshooting)

---

## Installation

### Prerequisites

- macOS 10.14 or later
- Rust 1.75+ (for building from source)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/esmondo/resikno-mac.git
cd resikno-mac

# Build release version
cargo build --release

# Install globally (optional)
ln -sf "$(pwd)/target/release/resikno" ~/.cargo/bin/resikno
```

### Verify Installation

```bash
resikno --version
# Output: resikno 0.2.0
```

---

## Quick Start

### 1. First Scan (Safe - Dry Run)

```bash
resikno scan --no-interactive
```

This shows what can be cleaned **without deleting anything**.

Example output:
```
Scanning system for cleanable files (no size filter)...

Found 1567 items totaling 11.9 GB
Safely recoverable: 7.3 GB

📦 System Caches - 3.3 GB
📦 App Caches - 650.4 MB
🗑️ Temp Files - 1.9 GB
⬇️ Large Re-download - 1.4 GB
📋 Logs - 10.6 MB
📂 Downloads - 4.6 GB
```

### 2. Launch Interactive TUI

```bash
resikno scan
```

This opens the interactive Terminal UI where you can select items to clean.

### 3. Quick Clean (Caches Only)

```bash
# Dry run first (see what will be deleted)
resikno clean caches

# Actually clean (requires --execute)
resikno clean caches --execute
```

---

## Interactive Shell Mode

Launch the interactive shell by running `resikno` without arguments:

```bash
$ resikno

  ░█▀▀█ ░█▀▀▀ ░█▀▀▀█ ▀█▀ ░█─▄▀ ░█▄─░█ ░█▀▀▀█
  ░█▄▄▀ ░█▀▀▀ ─▀▀▀▄▄ ░█─ ░█▀▄─ ░█░█░█ ░█──░█
  ░█─░█ ░█▄▄▄ ░█▄▄▄█ ▄█▄ ░█─░█ ░█──▀█ ░█▄▄▄█

  Lightweight Disk Cleanup for macOS
  v0.2.0

resikno > 
```

### Shell Commands

| Command | Description | Example |
|---------|-------------|---------|
| `scan` | Scan and open TUI | `resikno > scan` |
| `scan -m 100` | Scan with min size filter (MB) | `resikno > scan -m 100` |
| `review` | Review last scan in TUI | `resikno > review` |
| `status` | Show last scan summary | `resikno > status` |
| `clean <category>` | Dry-run clean category | `resikno > clean caches` |
| `clean <cat> --execute` | Actually clean | `resikno > clean caches --execute` |
| `clean --safe-only` | Clean only SAFE items | `resikno > clean all --safe-only` |
| `clean --force` | Skip confirmation | `resikno > clean caches --execute --force` |
| `analyze --large 500` | Find files > 500MB | `resikno > analyze --large 500` |
| `analyze --duplicates` | Find duplicate files | `resikno > analyze --duplicates` |
| `restore --list` | List restore points | `resikno > restore --list` |
| `restore --latest` | Restore latest cleanup | `resikno > restore --latest` |
| `update` | Check for updates | `resikno > update` |
| `version` | Show version | `resikno > version` |
| `help` | Show help | `resikno > help` |
| `exit` | Quit shell | `resikno > exit` |

### Categories

- `caches` - System and app caches
- `logs` - Log files
- `temp` - Temporary files
- `downloads` - Downloads folder
- `all` - All categories (default)

---

## Command Reference

### Global Options

```bash
resikno [OPTIONS] <COMMAND>

Options:
  -v, --verbose    Enable verbose output
      --json       Output as JSON (for scripting)
  -h, --help       Print help
  -V, --version    Print version
```

### Scan Command

```bash
resikno scan [OPTIONS] [PATH]

Arguments:
  [PATH]  Directory to scan (default: ~)

Options:
      --no-interactive    Skip TUI, just show results
  -m, --min-size <MB>     Minimum file size in MB
  -M, --max-size <MB>     Maximum file size in MB
```

**Examples:**

```bash
# Basic scan with TUI
resikno scan

# Scan with size filter (files > 100MB only)
resikno scan --min-size 100

# Scan Downloads only
resikno scan ~/Downloads

# Text output only (no TUI)
resikno scan --no-interactive
```

### Clean Command

```bash
resikno clean [OPTIONS] [CATEGORY]

Arguments:
  [CATEGORY]  Category to clean: caches, logs, temp, downloads, all [default: all]

Options:
      --execute      Actually perform deletion (default is dry-run)
      --safe-only    Only clean items marked as SAFE
  -f, --force        Skip confirmation prompts
```

**Examples:**

```bash
# Dry run (see what would be deleted)
resikno clean caches

# Clean only SAFE caches
resikno clean caches --safe-only

# Clean all SAFE items
resikno clean all --safe-only --execute

# Clean without confirmation (use with caution!)
resikno clean caches --execute --force

# Clean with specific size considerations
resikno clean logs --execute
```

### Analyze Command

```bash
resikno analyze [OPTIONS]

Options:
      --duplicates       Find duplicate files
      --large <MB>       Find files larger than MB
      --old <DAYS>       Find files older than DAYS (not implemented)
```

**Examples:**

```bash
# Find duplicate files
resikno analyze --duplicates

# Find files larger than 500MB
resikno analyze --large 500

# Combined (future feature)
resikno analyze --duplicates --large 100
```

### Restore Command

```bash
resikno restore [OPTIONS] [DATE]

Arguments:
  [DATE]  Restore point date (YYYY-MM-DD) or partial match

Options:
      --latest    Restore the most recent cleanup
      --list      List available restore points
```

**Examples:**

```bash
# List all restore points
resikno restore --list

# Restore most recent cleanup
resikno restore --latest

# Restore specific date
resikno restore 2026-03-31

# Restore by partial date match
resikno restore 2026-03
```

### Update Command

```bash
resikno update [OPTIONS]

Options:
      --check    Check for updates without installing
```

**Examples:**

```bash
# Check for updates
resikno update --check

# Update to latest version
resikno update
```

---

## Safety Features

### Safety Levels

Every item has a safety rating:

| Level | Emoji | Description | Auto-clean? |
|-------|-------|-------------|-------------|
| **SAFE** | 🔵 | Cache files, temp files | ✅ Yes |
| **REVIEW** | 🟢 | Logs, iOS backups, Xcode data | ⚠️ Suggested |
| **CAREFUL** | 🟡 | Downloads, large files | ❌ No |
| **PROTECTED** | ⚪ | System files, Documents, etc. | 🚫 Never |

### Protected Paths (Never Deleted)

The following are **never** offered for cleanup:

- System directories (`/System`, `/usr`, `/bin`, etc.)
- User documents (`~/Documents`, `~/Desktop`, `~/Pictures`, etc.)
- Credentials (`~/.ssh`, `~/.aws`, `~/.kube`, etc.)
- Git repositories (`~/.git/objects`)
- Application data

### Dry-Run by Default

**Important:** By default, all clean commands show what **would** be deleted without actually deleting anything.

```bash
# Dry run (safe)
resikno clean caches

# Actually delete (requires --execute)
resikno clean caches --execute
```

### Trash Recovery (macOS)

Deleted files are moved to **Trash**, not permanently deleted. You can recover them from Trash in Finder.

### Restore Points

Every cleanup creates a restore point with metadata:

```bash
# List restore points
resikno restore --list

# Restore from a specific point
resikno restore --latest
```

**Note:** Files must still exist in Trash to be restored. If you empty Trash, restoration is not possible.

---

## TUI (Terminal UI) Guide

The TUI opens when you run `resikno scan` or `resikno review`.

### Layout

```
┌─ Disk Cleaner ─────────────────────────────────────────┐
│ RESIKNO  Found 11.9 GB (7.3 GB recoverable)            │  <- Header with stats
├────────────────────────────────────────────────────────┤
│  > [ ] ▶ 📦 System Caches     3.3 GB  SAFE  [██████░░] │  <- Category row
│    [ ] ▶ 📦 App Caches      650.4 MB  SAFE  [█░░░░░░░] │
│    [ ] ▶ 🗑️ Temp Files        1.9 GB  SAFE  [███░░░░░] │
├────────────────────────────────────────────────────────┤
│ [↑↓] Nav [Enter] Expand [Space] Select [A] All [Q]Quit │  <- Footer help
└────────────────────────────────────────────────────────┘
```

### Keyboard Controls

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate up/down |
| `k` / `j` | Navigate (vim style) |
| `Enter` | Expand/collapse category or folder |
| `Space` | Select/deselect item |
| `a` | Select/deselect all |
| `f` | Reveal selected item in Finder |
| `c` | Clean selected items (exits to confirmation) |
| `q` / `Esc` | Quit TUI |

### Selection Indicators

```
[ ]  - Not selected
[x]  - Selected
[-]  - Partially selected (some items in category selected)
```

### Usage Flow

1. **Launch TUI:**
   ```bash
   resikno scan
   ```

2. **Navigate categories:**
   - Use `↑↓` to move
   - Press `Enter` to expand a category

3. **Select items:**
   - Press `Space` to select/deselect
   - Or press `a` to select all SAFE items

4. **Drill down into folders:**
   - Navigate to a folder
   - Press `Enter` to see its contents
   - Select individual files

5. **Clean:**
   - Press `c` to clean selected items
   - TUI exits and shows confirmation prompt

6. **Confirm:**
   - Type `y` to proceed
   - Type `n` to cancel

---

## Scenarios & Examples

### Scenario 1: Quick Cache Cleanup

**Goal:** Free up space quickly and safely

```bash
# 1. Check what's in caches (dry run)
resikno clean caches

# 2. If looks good, clean only SAFE caches
resikno clean caches --safe-only --execute
```

**Expected:** Frees 3-5 GB typically

### Scenario 2: Find and Clean Large Files

**Goal:** Find files larger than 1GB

```bash
# 1. Analyze large files
resikno analyze --large 1000

# 2. If needed, scan Downloads specifically
resikno scan ~/Downloads --no-interactive

# 3. Use TUI to selectively clean
resikno scan ~/Downloads
```

### Scenario 3: Find Duplicate Downloads

**Goal:** Find duplicate files in Downloads

```bash
# Find duplicates
resikno analyze --duplicates

# Example output:
# Found 3 duplicate groups, 15.9 MB recoverable:
#   Group 1 (2.9 MB each):
#     📄 ~/Downloads/file (1).pdf
#     📄 ~/Downloads/file.pdf
```

Manually delete duplicates in Finder, or:
```bash
# Open Downloads in Finder
open ~/Downloads
```

### Scenario 4: Full System Cleanup

**Goal:** Comprehensive cleanup with safety

```bash
# Step 1: Scan everything
resikno scan --no-interactive

# Step 2: Clean only SAFE items first
resikno clean all --safe-only --execute

# Step 3: Review REVIEW items
resikno clean logs --execute

# Step 4: Check Downloads manually
resikno scan ~/Downloads
```

### Scenario 5: Automated/Scripted Cleanup

**Goal:** Run in a script without prompts

```bash
#!/bin/bash
# nightly-cleanup.sh

# Clean only SAFE caches without confirmation
resikno clean caches --safe-only --execute --force

# Log the result
echo "$(date): Cache cleanup completed" >> ~/.resikno-mac/logs/cleanup.log
```

### Scenario 6: Recover Deleted Files

**Goal:** Restore accidentally deleted files

```bash
# List restore points
resikno restore --list

# Restore latest
resikno restore --latest

# Or restore specific date
resikno restore 2026-03-31
```

**Note:** Files are restored from Trash. If Trash was emptied, files cannot be recovered.

---

## Troubleshooting

### Permission Denied Errors

```
Error: Permission denied: /path/to/file

Hint: Try running with sudo or check file permissions.
```

**Solution:**
- System files need `sudo` (but generally don't clean these)
- User files should work without sudo
- Some app caches may need app to be closed

### No Items Found

```
No items found to clean.
```

**Possible causes:**
- Recently cleaned already
- Using `--safe-only` with no SAFE items in category
- Size filters too restrictive

**Solution:**
```bash
# Check without filters
resikno scan --no-interactive

# Try different category
resikno clean logs
```

### TUI Display Issues

**Problem:** TUI looks garbled or colors don't show

**Solution:**
- Ensure terminal supports Unicode and 256 colors
- Try different terminal (iTerm2, Terminal.app, Alacritty)
- Check `TERM` environment variable

### Restore Failed

```
Failed: X items (may have been permanently deleted)
```

**Cause:** Files were deleted from Trash in Finder

**Prevention:**
- Don't empty Trash immediately after cleanup
- Restore soon after accidental deletion

### Build Errors

**Problem:** `cargo build` fails

**Solutions:**
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release

# Check dependencies
cargo check
```

---

## Tips & Best Practices

### 1. Always Dry-Run First

```bash
# See what will be deleted
resikno clean caches

# Then execute
resikno clean caches --execute
```

### 2. Use `--safe-only` for Automation

```bash
# Safe for cron jobs
resikno clean all --safe-only --execute --force
```

### 3. Review Before Major Cleanups

```bash
# Check REVIEW items individually
resikno clean logs --execute
resikno clean temp --execute
```

### 4. Keep Restore Points

Don't empty Trash immediately if you might need to restore.

### 5. Regular Maintenance

```bash
# Weekly SAFE cleanup
0 9 * * 1 /usr/local/bin/resikno clean all --safe-only --execute --force
```

### 6. Monitor Space Savings

```bash
# Before
df -h ~

# Clean
resikno clean caches --safe-only --execute

# After
df -h ~
```

---

## FAQ

**Q: Is it safe to use?**  
A: Yes. Protected paths are never touched, and dry-run is default.

**Q: Can I recover deleted files?**  
A: Yes, if they haven't been emptied from Trash. Use `resikno restore`.

**Q: What if I accidentally clean something important?**  
A: Files go to Trash first. Restore from Trash or use `resikno restore`.

**Q: Does it work on Apple Silicon (M1/M2/M3)?**  
A: Yes, native support for both Intel and Apple Silicon Macs.

**Q: Can I schedule automatic cleanups?**  
A: Yes, use cron with `--safe-only --force` flags.

**Q: What's the difference between SAFE and REVIEW?**  
A: SAFE items are caches/temp that apps regenerate. REVIEW items (logs, backups) might have value.

---

## Support

- **Issues:** https://github.com/esmondo/resikno-mac/issues
- **Source:** https://github.com/esmondo/resikno-mac

---

*Last updated: 2026-04-07*
