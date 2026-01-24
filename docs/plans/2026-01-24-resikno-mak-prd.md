# Resikno-Mak: Product Requirements Document

> A lightweight, transparent, and safe disk cleanup CLI tool for macOS (and eventually cross-platform)

**Version:** 1.0
**Date:** 2026-01-24
**Status:** Draft

---

## Table of Contents

1. [Product Overview & Vision](#1-product-overview--vision)
2. [Core Features](#2-core-features)
3. [Technical Architecture](#3-technical-architecture)
4. [User Experience & Commands](#4-user-experience--commands)
5. [Safety & Restore System](#5-safety--restore-system)
6. [Success Metrics & Scope](#6-success-metrics--v10-scope)

---

## 1. Product Overview & Vision

### Product Name
**Resikno-Mak** (wordplay on "clean my Mac")

### Vision
A lightweight, transparent, and safe disk cleanup tool that respects users by being honest about what it does, requiring no subscription, and never surprising them with aggressive cleanups or data loss.

### Target Users
- Mac users frustrated with CleanMyMac's pricing and bloat
- Developers and power users who want CLI control
- Users who value transparency and safety over automated "magic"
- Eventually: Linux and Windows users seeking similar tools

### Key Differentiators from CleanMyMac

| CleanMyMac Problem | Resikno-Mak Solution |
|-------------------|----------------------|
| Expensive subscription ($47/year) | One-time purchase or free |
| Feature bloat | CLI-first, focused features |
| No restore points | Automatic restore before every cleanup |
| Hidden actions | Complete transparency - show everything |
| Aggressive notifications | Quiet, user-initiated only |
| GUI-only | CLI with beautiful TUI |
| macOS only | Cross-platform vision |

### Core Philosophy
> "Trust through transparency" - Users should understand and control every action the tool takes.

---

## 2. Core Features

### 2.1 Interactive Visual Disk Usage Analysis

**Goal:** Help users explore and understand what's consuming their disk space

#### Interactive Features
- **Selectable items** - Arrow keys to navigate, Enter to drill down
- **Breadcrumb navigation** - Always know where you are
- **Expandable tree view** - See what's inside each directory/category
- **Multiple view modes:**
  - Tree view (hierarchical)
  - Size-sorted list
  - Category view (group by file type)

#### Rich Color Scheme

| Color | Meaning |
|-------|---------|
| 🔴 **Red** | Critical space hogs (>10 GB) |
| 🟠 **Orange** | Large items (1-10 GB) |
| 🟡 **Yellow** | Medium items (100 MB - 1 GB) |
| 🟢 **Green** | Small items (<100 MB) |
| 🔵 **Blue** | Safe to clean (based on safety rating) |
| ⚪ **Gray** | System files (don't touch) |

#### Example Interactive Output

```
┌─ Disk Analysis ────────────────────────────────────┐
│ Total: 245.3 GB used | 12.7 GB available          │
├────────────────────────────────────────────────────┤
│ > 📦 System Caches        45.2 GB  🔵 SAFE        │
│   📦 App Caches           32.1 GB  🔵 SAFE        │
│   📦 Logs                 12.5 GB  🟡 REVIEW      │
│   📂 Downloads            98.7 GB  🟡 REVIEW      │
│   📂 Movies               67.3 GB  ⚪ KEEP        │
├────────────────────────────────────────────────────┤
│ ← Back | ↑↓ Navigate | ⏎ Expand | Space: Select  │
└────────────────────────────────────────────────────┘
```

### 2.2 Smart Safety Recommendations

Each cleanup category gets a **safety rating** and **recommendation**:

#### 🔵 SAFE (Auto-cleanable)
- System caches
- Application caches (non-active)
- Temporary files
- Old logs (>30 days)
- Trash contents
- *Recommendation: "Can be cleaned automatically with minimal risk"*

#### 🟢 MOSTLY SAFE (Review suggested)
- Duplicate files
- Old iOS backups (>90 days)
- Xcode derived data
- Browser downloads (>30 days)
- *Recommendation: "Review list before cleaning"*

#### 🟡 REVIEW CAREFULLY
- Large old files
- Downloads folder
- Language files
- *Recommendation: "Manual review required - may contain important files"*

#### 🔴 CAUTION
- Application support files
- User documents
- System files
- *Recommendation: "Not recommended for automatic cleanup"*

#### ⚪ DON'T TOUCH
- macOS system directories
- Active application data
- User-created content
- *Recommendation: "Protected - will not be offered for cleanup"*

### 2.3 Smart Cleanup Categories

| Category | Typical Recovery | Safety | Interactive Details |
|----------|------------------|--------|---------------------|
| System Caches | 10-50 GB | 🔵 SAFE | Expand to see per-app breakdown |
| App Caches | 5-30 GB | 🔵 SAFE | Select to see which apps |
| Logs | 1-10 GB | 🟢 MOSTLY SAFE | Choose retention period |
| Temp Files | 2-15 GB | 🔵 SAFE | Auto-detected safe temps |
| iOS Backups | 10-100+ GB | 🟢 MOSTLY SAFE | Select which backups to keep |
| Xcode Data | 20-100+ GB | 🟢 MOSTLY SAFE | Developers only |
| Duplicates | 5-50 GB | 🟡 REVIEW | Preview before delete |
| Large/Old Files | Variable | 🟡 REVIEW | Filter by size/age/type |
| Downloads | 10-50 GB | 🟡 REVIEW | Sort by age/size |
| Language Files | 2-5 GB | 🟢 MOSTLY SAFE | Choose languages to keep |

### 2.4 Cleanup Category Details

#### System Caches (🔵 SAFE)
- `/Library/Caches/*`
- `~/Library/Caches/*`
- Exclude actively-used caches
- Estimated recovery: 10-50 GB typically

#### Application Caches (🔵 SAFE)
- Browser caches (Safari, Chrome, Firefox, Edge)
- Slack, Discord, VS Code caches
- Per-app breakdown showing impact
- Estimated recovery: 5-30 GB typically

#### Log Files (🟢 MOSTLY SAFE)
- System logs `/var/log/*`
- Application logs `~/Library/Logs/*`
- Keep last 7 days by default (configurable)
- Estimated recovery: 1-10 GB typically

#### Temporary Files (🔵 SAFE)
- `/tmp/*` and `/var/tmp/*`
- `~/Library/Application Support/*/tmp`
- Download quarantine files
- Estimated recovery: 2-15 GB typically

#### Old iOS/iPadOS Backups (🟢 MOSTLY SAFE)
- `~/Library/Application Support/MobileSync/Backup/*`
- Show device names and backup dates
- Estimated recovery: 10-100+ GB per backup

#### Xcode Derived Data & Archives (🟢 MOSTLY SAFE)
- `~/Library/Developer/Xcode/DerivedData/*`
- `~/Library/Developer/Xcode/Archives/*`
- iOS DeviceSupport files
- Estimated recovery: 20-100+ GB for developers

#### Duplicate Files (🟡 REVIEW)
- Hash-based duplicate detection
- Show file size, path, and modification date
- Preview mode before deletion
- Estimated recovery: Variable, 5-50 GB

#### Large & Old Files (🟡 REVIEW)
- Files over X GB (user configurable, default 1GB)
- Not accessed in Y days (user configurable, default 180)
- Filter by file type
- Estimated recovery: Variable

#### Downloads Folder Cleanup (🟡 REVIEW)
- Show files by age and size
- Quick filters (images, videos, installers, archives)
- Estimated recovery: Variable, 10-50 GB

#### Unused Language Files (🟢 MOSTLY SAFE)
- .app bundles contain 20+ languages
- Keep system language + user preferences
- Estimated recovery: 2-5 GB

---

## 3. Technical Architecture

### 3.1 Technology Stack

**Language:** Rust
**Platform Priority:** macOS first, then Windows/Linux

### 3.2 Project Structure

```
resikno-mak/
├── Cargo.toml              # Dependencies
├── src/
│   ├── main.rs             # Entry point
│   ├── cli/                # Command-line interface
│   │   ├── commands.rs     # Command definitions
│   │   └── args.rs         # Argument parsing
│   ├── scanner/            # Disk scanning engine
│   │   ├── mod.rs
│   │   ├── cache.rs        # Cache detection
│   │   ├── duplicates.rs   # Duplicate finder
│   │   └── large_files.rs  # Large file detection
│   ├── cleaner/            # Cleanup operations
│   │   ├── mod.rs
│   │   ├── backup.rs       # Restore point creation
│   │   └── delete.rs       # Safe deletion
│   ├── ui/                 # Terminal UI
│   │   ├── mod.rs
│   │   ├── tree.rs         # Interactive tree view
│   │   ├── colors.rs       # Color schemes
│   │   └── charts.rs       # ASCII visualizations
│   └── platform/           # Platform-specific code
│       ├── mod.rs
│       ├── macos.rs        # macOS paths & APIs
│       ├── linux.rs        # Linux paths
│       └── windows.rs      # Windows paths
└── tests/
```

### 3.3 Key Dependencies (Cargo.toml)

```toml
[dependencies]
# CLI Framework
clap = { version = "4", features = ["derive"] }

# Interactive TUI
ratatui = "0.25"          # Terminal UI framework
crossterm = "0.27"        # Cross-platform terminal control

# File Operations
walkdir = "2"             # Fast directory traversal
sha2 = "0.10"             # Hashing for duplicates
fs_extra = "1.3"          # Enhanced file operations

# Visualization & Colors
indicatif = "0.17"        # Progress bars
console = "0.15"          # Colors and styling
tabled = "0.15"           # ASCII tables

# Async & Performance
rayon = "1.8"             # Parallel processing
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"          # JSON export

# Platform Detection
cfg-if = "1"              # Conditional compilation
directories = "5"         # Cross-platform paths
```

### 3.4 Core Architecture Principles

#### Safety First
```
User Action → Create Restore Point → Execute → Verify → Report
                    ↓
              Store in ~/.resikno-mak/restore/
              (Auto-cleanup after 7 days)
```

#### Scan Once, Display Many
- Single scan builds complete disk map
- Cached in memory for fast navigation
- Re-scan only when user requests or files change

#### Platform Abstraction
```rust
// Platform-specific paths handled via trait
trait PlatformPaths {
    fn cache_dirs(&self) -> Vec<PathBuf>;
    fn log_dirs(&self) -> Vec<PathBuf>;
    fn temp_dirs(&self) -> Vec<PathBuf>;
    // ... etc
}

// Implementations for each OS
struct MacOSPaths;
struct LinuxPaths;
struct WindowsPaths;
```

#### Non-Destructive by Default
- `--dry-run` is the default behavior
- Must explicitly use `--execute` or confirm interactively
- Preview everything before action

---

## 4. User Experience & Commands

### 4.1 Command Structure

```bash
# Main scan & interactive mode
resikno scan                  # Full scan, opens interactive TUI
resikno scan ~/Downloads      # Scan specific directory
resikno scan --json           # Output as JSON (for scripting)

# Quick actions
resikno clean caches          # Clean all caches (with confirmation)
resikno clean logs            # Clean old logs
resikno clean all --safe      # Clean all 🔵 SAFE items
resikno clean --dry-run       # Show what would be deleted

# Analysis
resikno analyze               # Show disk usage breakdown
resikno analyze --duplicates  # Find duplicate files
resikno analyze --large       # Find large files
resikno analyze --old 180     # Find files older than 180 days

# Restore
resikno restore               # List available restore points
resikno restore --latest      # Restore most recent cleanup
resikno restore 2026-01-24    # Restore specific date

# Config
resikno config                # Open config in editor
resikno config set retention 30    # Set log retention days
resikno config add-safe ~/temp     # Mark directory as safe to clean
```

### 4.2 Interactive TUI Flow

#### Scanning Screen
```
┌──────────────────────────────────────────────────────────────┐
│  RESIKNO-MAK v1.0                     💾 245.3 GB / 500 GB  │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ⏳ Scanning...  [████████████░░░░░░░░] 62%  ~/Library      │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

#### Main Dashboard
```
┌──────────────────────────────────────────────────────────────┐
│  RESIKNO-MAK                  🎯 Recoverable: 87.4 GB       │
├──────────────────────────────────────────────────────────────┤
│  [Tab] Categories  [Tab] All Files  [Tab] Duplicates        │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ☑ 📦 System Caches        45.2 GB  🔵 SAFE    [████████░░] │
│  ☑ 📦 App Caches           32.1 GB  🔵 SAFE    [██████░░░░] │
│  ☐ 📋 Logs                 12.5 GB  🟢 REVIEW  [██░░░░░░░░] │
│  ☐ 📂 Downloads            98.7 GB  🟡 REVIEW  [█████████░] │
│  ☐ 📀 iOS Backups          45.0 GB  🟢 REVIEW  [███████░░░] │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│  Selected: 77.3 GB                                           │
│                                                              │
│  [Space] Toggle  [Enter] Expand  [C] Clean Selected  [Q] Quit│
└──────────────────────────────────────────────────────────────┘
```

#### Drill-Down View
```
┌──────────────────────────────────────────────────────────────┐
│  📦 System Caches (45.2 GB)                    ← Back: Esc  │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ☑ com.apple.Safari        12.3 GB  Last: 2 hours ago       │
│  ☑ com.google.Chrome        8.7 GB  Last: 1 day ago         │
│  ☑ com.spotify.client       6.2 GB  Last: 3 days ago        │
│  ☑ com.microsoft.VSCode     4.1 GB  Last: 5 hours ago       │
│  ☑ com.slack.Slack          3.8 GB  Last: 1 hour ago        │
│    ... 23 more items                                         │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│  [A] Select All  [N] Select None  [Enter] View Files        │
└──────────────────────────────────────────────────────────────┘
```

#### Confirmation Dialog
```
┌──────────────────────────────────────────────────────────────┐
│  ⚠️  CONFIRM CLEANUP                                         │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  You are about to remove:                                    │
│                                                              │
│    • System Caches: 45.2 GB (28 items)                       │
│    • App Caches: 32.1 GB (45 items)                          │
│                                                              │
│  Total: 77.3 GB                                              │
│                                                              │
│  ✅ A restore point will be created automatically            │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│  [Y] Yes, Clean  [N] Cancel  [P] Preview Details            │
└──────────────────────────────────────────────────────────────┘
```

---

## 5. Safety & Restore System

### 5.1 Restore Point Architecture

**Storage Location:**
```
~/.resikno-mak/
├── config.toml             # User configuration
├── restore/
│   ├── 2026-01-24_143052/  # Timestamped restore point
│   │   ├── manifest.json   # What was deleted
│   │   ├── files/          # Actual backed up files (optional)
│   │   └── metadata.json   # File metadata for recreation
│   └── 2026-01-23_091522/
└── logs/
    └── cleanup.log         # Audit trail
```

**manifest.json Example:**
```json
{
  "timestamp": "2026-01-24T14:30:52Z",
  "total_size": "77.3 GB",
  "items": [
    {
      "path": "~/Library/Caches/com.apple.Safari",
      "size": "12.3 GB",
      "type": "cache",
      "backed_up": false,
      "reason": "Cache files - regenerated automatically"
    }
  ]
}
```

### 5.2 Smart Backup Decisions

| File Type | Backup Strategy | Reason |
|-----------|-----------------|--------|
| Caches | Manifest only | Regenerated by apps |
| Logs | Manifest only | Usually not needed |
| Duplicates | Keep one copy | Original preserved |
| User files | Full backup | May be important |
| iOS backups | Manifest + warn | Large but potentially valuable |

### 5.3 Restore Commands

```bash
# List available restore points
resikno restore
# Output:
# Available restore points:
#   1. 2026-01-24 14:30 - 77.3 GB cleaned (caches, logs)
#   2. 2026-01-23 09:15 - 23.1 GB cleaned (duplicates)

# Restore specific point
resikno restore 2026-01-24

# Restore latest
resikno restore --latest
```

---

## 6. Success Metrics & v1.0 Scope

### 6.1 What Success Looks Like

| Metric | Target |
|--------|--------|
| **Speed** | Full scan completes in <30 seconds for typical Mac |
| **Safety** | Zero data loss incidents (restore always works) |
| **UX** | Users understand exactly what's happening at every step |
| **Recovery** | Average user recovers 30-80 GB on first use |
| **Trust** | Users feel confident using it without fear |

### 6.2 v1.0 Feature Scope

#### Included in v1.0
- ✅ Interactive TUI with full navigation
- ✅ All cleanup categories (caches, logs, temps, duplicates, etc.)
- ✅ Restore point system
- ✅ Color-coded safety recommendations
- ✅ JSON export for scripting
- ✅ macOS full support

#### Deferred to v1.1+
- ⏳ Windows support
- ⏳ Linux support
- ⏳ Scheduled cleanups
- ⏳ Menu bar widget
- ⏳ Real-time monitoring

---

## 7. Implementation Phases

### Phase 1: Foundation
- [ ] Set up Rust project structure
- [ ] Implement basic CLI with clap
- [ ] Create platform abstraction layer
- [ ] Build macOS path detection

### Phase 2: Scanning Engine
- [ ] Directory traversal with walkdir
- [ ] Size calculation and caching
- [ ] Category classification
- [ ] Duplicate detection with hashing

### Phase 3: Terminal UI
- [ ] Basic ratatui setup
- [ ] Interactive tree view
- [ ] Color scheme implementation
- [ ] Progress bars and animations

### Phase 4: Cleanup & Safety
- [ ] Restore point creation
- [ ] Safe deletion with verification
- [ ] Manifest logging
- [ ] Restore functionality

### Phase 5: Polish & Release
- [ ] Error handling and edge cases
- [ ] Performance optimization
- [ ] Documentation
- [ ] Release binaries

---

## Appendix: Research Sources

- [CleanMyMac user complaints - Apple Community](https://discussions.apple.com/thread/255036185)
- [Why users switch to alternatives - AlternativeTo](https://alternativeto.net/software/cleanmymac/)
- [CleanMyMac reviews and feedback - Cybernews](https://cybernews.com/best-antivirus-software/cleanmymac-review/)
- [Reddit community preferences - G2](https://www.g2.com/products/cleanmymac/competitors/alternatives)
