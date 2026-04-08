# Resikno TUI Design - Claude Code Inspired

## Design Philosophy

The new TUI follows Claude Code's aesthetic principles:
- **Dark, minimal theme** - Easy on the eyes for extended use
- **Subtle color accents** - Muted cyan as primary, not bright
- **Clear visual hierarchy** - Important elements stand out
- **Generous whitespace** - Breathing room between elements
- **Tool-like blocks** - Categories look like Claude's tool use indicators
- **Clean typography** - Consistent spacing and alignment

## Visual Changes

### Before (Original)
```
┌─ Disk Cleaner ─────────────────────────────────────────┐
│ RESIKNO  Found 11.9 GB (7.3 GB recoverable)            │
├────────────────────────────────────────────────────────┤
│  > [ ] ▶ 📦 System Caches     3.3 GB  SAFE  [██████░░] │
│    [ ] ▶ 📦 App Caches      650.4 MB  SAFE  [█░░░░░░░] │
├────────────────────────────────────────────────────────┤
│ [↑↓] Nav  [Enter] Expand  [Space] Select...            │
└────────────────────────────────────────────────────────┘
```

### After (Claude Code Style)
```

  ◉ Resikno
  
  › ◯ ▶ 📦 System Caches    3.3 GB   [SAFE]
    ◯ ▶ 📦 App Caches     650.4 MB   [SAFE]
    ◯ ▶ 🗑️ Temp Files      1.9 GB   [SAFE]
    
  1,567 items found  •  11.9 GB total  •  7.3 GB recoverable
  
  ─────────────────────────────────────────────────────────
  Navigation: ↑↓ move  Enter expand  Space select
  Actions: a all  c clean  f finder  q quit

```

## Key Improvements

### 1. Color Palette

| Element | Old | New (Claude-like) |
|---------|-----|-------------------|
| Background | Black | `rgb(15, 15, 15)` - subtle dark |
| Selection | `DarkGray` | `rgb(48, 48, 48)` - elevated gray |
| Primary accent | Bright Cyan | `rgb(95, 180, 180)` - muted cyan |
| Success | Bright Green | `rgb(130, 180, 130)` - soft green |
| Warning | Bright Yellow | `rgb(200, 180, 120)` - soft yellow |
| Error | Bright Red | `rgb(220, 130, 130)` - soft red |

### 2. Layout Structure

```
┌─────────────────────────────────────────────┐
│  ◉ Resikno                                   │  <- Clean header
│                                              │
│  › ◯ ▶ 📦 Category Name    Size   [STATUS]  │  <- Category rows
│    ◯ ▶ 📁 Item Name        Size             │  <- Child items
│                                              │
│  Status line with stats                      │  <- Info bar
│                                              │
│  ─────────────────────────────────────────   │  <- Subtle separator
│  Navigation: ↑↓ move  Enter expand...        │  <- Command palette
└─────────────────────────────────────────────┘
```

### 3. Selection Indicators

- **Cursor**: `›` (subtle arrow, not `>`)
- **Selected checkbox**: `✓` (checkmark)
- **Partial checkbox**: `◐` (half-filled)
- **Empty checkbox**: `◯` (circle, not `[ ]`)

### 4. Safety Level Pills

Instead of text labels, use pill-like backgrounds:

```
[SAFE]     - Green background, dark text
[REVIEW]   - Yellow background, dark text  
[CAREFUL]  - Orange background, dark text
[CAUTION]  - Red background, dark text
```

### 5. Welcome Screen

**Before:**
```
  ░█▀▀█ ░█▀▀▀ ...
  ░█▄▄▀ ░█▀▀▀ ...
  ░█─░█ ░█▄▄▄ ...
  
  Lightweight Disk Cleanup for macOS
  v0.2.0
  
  Tips:
  1. Type 'scan' to find cleanable files
  2. Type 'help' for all commands
  3. Press Ctrl+D or type 'exit' to quit
```

**After:**
```
  ◉ Resikno
  Disk Cleanup for macOS
  v0.2.0

  Quick start:

  scan                   Scan for cleanable files
  clean caches --execute Clean cache files
  help                   Show all commands

  Press Ctrl+D or type 'exit' to quit
```

## Files Modified

| File | Changes |
|------|---------|
| `src/ui/colors.rs` | New Claude-inspired dark color palette |
| `src/ui/tree.rs` | Complete redesign of layout and styling |
| `src/shell/welcome.rs` | Minimal, clean welcome screen |

## Keyboard Shortcuts (Unchanged)

| Key | Action |
|-----|--------|
| `↑↓` | Navigate |
| `Enter` | Expand/collapse |
| `Space` | Select/deselect |
| `a` | Select all |
| `f` | Reveal in Finder |
| `c` | Clean selected |
| `q` | Quit |

## Testing

Run the TUI:
```bash
cd resikno-mac
cargo run -- scan
```

Expected appearance:
- Dark background (not pure black)
- Muted cyan accents
- Clean, uncluttered layout
- Subtle selection highlighting
- Pills for safety levels
