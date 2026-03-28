# Resikno-Mac: Future Plans

Features not in the current implementation scope. Prioritized roughly by user impact.

---

## High Priority

### Per-app cache breakdown
Instead of showing `~/Library/Caches` as a single item, break it down per app (e.g., Slack: 1.2GB, Chrome: 800MB). Requires walking one level deeper and grouping by bundle identifier. High UX value — users can see exactly which app is using space.

### Scheduled / automatic scans
Add a background daemon (`launchd` plist on macOS) that runs `resikno scan` on a schedule and notifies the user if space is recoverable above a threshold. Config key: `auto_scan_interval_days`.

### Disk usage visualization
A treemap or bar chart showing disk usage by category and by folder. Could be done in-TUI with ratatui or as a separate `resikno viz` command that opens an HTML report in the browser.

---

## Medium Priority

### Privacy cleanup
Clean browser history, recent files list, clipboard, and app-specific recent documents. Requires per-app knowledge of where these are stored. Categories:
- Safari history: `~/Library/Safari/History.db`
- Chrome history: `~/Library/Application Support/Google/Chrome/Default/History`
- Recent items: `~/Library/Application Support/com.apple.sharedfilelist/`
- Clipboard: programmatic clear via `pbcopy < /dev/null`

### Mail attachments cleanup
Scan `~/Library/Mail/` for downloaded attachments that are duplicated elsewhere or in large mail threads. Safety level: Caution (user confirmation required per attachment).

### Login items management
List and disable macOS login items (apps that launch at startup). Read from `SMLoginItemSetEnabled` / `LaunchAgents` plists. Display in TUI with enable/disable toggle. No deletion — just management.

### App uninstaller
Given an app name or path, find and remove all associated files:
- The `.app` bundle in `/Applications/`
- `~/Library/Application Support/<bundle-id>/`
- `~/Library/Preferences/<bundle-id>.plist`
- `~/Library/Caches/<bundle-id>/`
- `~/Library/Logs/<bundle-id>/`
Creates a restore point (Backup mode) before removal.

---

## Low Priority

### Windows / Linux support
Platform stubs exist (`platform/windows.rs`, `platform/linux.rs`). Fill in actual paths for each platform. Linux: `~/.cache/`, `~/.local/share/Trash/`. Windows: `%LOCALAPPDATA%`, `%TEMP%`.

### GUI wrapper
A native macOS SwiftUI app that shells out to `resikno` CLI commands and displays results. Resikno stays the engine; the GUI is a front-end. Keeps the CLI as the source of truth.

### Cloud storage cleanup
Detect Dropbox / iCloud Drive / Google Drive local caches and offline files that haven't been accessed recently. Offer to evict them from local storage (keep in cloud only).

### Homebrew cleanup
Run `brew cleanup --prune=all` and `brew autoremove` as part of a `resikno clean --brew` command. Show how much space would be freed before executing.

### RAM pressure relief
On macOS, run `sudo purge` to flush disk cache from RAM. Cosmetic effect but users expect this from CleanMyMac. Requires `sudo`. Should be gated behind an explicit `resikno ram` command.
