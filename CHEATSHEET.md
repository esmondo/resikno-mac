# Resikno-Mac Quick Reference

## 🚀 Quick Start

```bash
# First scan (dry-run, safe)
resikno scan --no-interactive

# Interactive TUI
resikno scan

# Clean caches (dry-run)
resikno clean caches

# Clean caches (actually delete)
resikno clean caches --execute
```

---

## 📋 Command Cheat Sheet

### Scan Commands
```bash
resikno scan                           # Scan + open TUI
resikno scan --no-interactive          # Text output only
resikno scan -m 100                    # Min size 100MB
resikno scan ~/Downloads               # Scan specific path
```

### Clean Commands
```bash
resikno clean caches                   # Dry-run caches
resikno clean all --safe-only          # Only SAFE items
resikno clean caches --execute         # Actually delete
resikno clean all --execute --force    # No confirmation
```

### Analyze Commands
```bash
resikno analyze --duplicates           # Find duplicates
resikno analyze --large 500            # Files > 500MB
```

### Restore Commands
```bash
resikno restore --list                 # List restore points
resikno restore --latest               # Restore latest
resikno restore 2026-03-31             # Restore specific date
```

---

## 🎮 TUI Controls

| Key | Action |
|-----|--------|
| `↑↓` or `jk` | Navigate |
| `Enter` | Expand/collapse |
| `Space` | Select/deselect |
| `a` | Select all/none |
| `f` | Reveal in Finder |
| `c` | Clean selected |
| `q` | Quit |

---

## 🛡️ Safety Levels

| Level | Emoji | Auto-Clean? | Examples |
|-------|-------|-------------|----------|
| SAFE | 🔵 | ✅ Yes | Caches, temp files |
| REVIEW | 🟢 | ⚠️ Suggest | Logs, iOS backups |
| CAREFUL | 🟡 | ❌ No | Downloads |
| PROTECTED | ⚪ | 🚫 Never | Documents, SSH keys |

---

## 💾 Categories

- `caches` - System & app caches
- `logs` - Log files
- `temp` - Temporary files
- `downloads` - Downloads folder
- `all` - Everything

---

## 🔧 Options

| Option | Description |
|--------|-------------|
| `--execute` | Actually delete (default: dry-run) |
| `--safe-only` | Only SAFE items |
| `--force` / `-f` | Skip confirmation |
| `-m <MB>` | Minimum size filter |
| `-M <MB>` | Maximum size filter |

---

## 📝 Common Workflows

### Quick Safe Cleanup
```bash
resikno clean all --safe-only --execute
```

### Check Before Clean
```bash
resikno clean caches                    # Dry-run
resikno clean caches --execute          # Execute
```

### Find Large Files
```bash
resikno analyze --large 1000
resikno scan ~/Downloads -m 1000
```

### Full Cleanup Session
```bash
resikno scan                            # Review in TUI
resikno clean caches --execute          # Clean caches
resikno clean logs --execute            # Clean logs
resikno analyze --duplicates            # Check duplicates
```

---

## 🔄 Restore Flow

```bash
resikno restore --list                  # See available
resikno restore --latest                # Restore latest
# or
resikno restore 2026-03-31              # Specific date
```

---

## ⚡ Pro Tips

1. **Always dry-run first** - Don't use `--execute` immediately
2. **Use `--safe-only` for scripts** - Won't delete important items
3. **Files go to Trash** - Can recover if needed
4. **Check Downloads manually** - Use TUI to review
5. **Use `--force` carefully** - Skips all confirmations

---

## 🐛 Troubleshooting

```bash
# Permission issues
sudo resikno clean caches --execute     # Rarely needed

# No items found
resikno scan --no-interactive           # Check what's available

# Build issues
cargo clean && cargo build --release
```

---

*For full documentation, see USER_GUIDE.md*
