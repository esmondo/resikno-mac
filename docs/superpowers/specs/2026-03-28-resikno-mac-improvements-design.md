# Resikno-Mac: Make All Features Work — Design Spec

**Date:** 2026-03-28
**Scope:** Fix foundation issues, implement restore UX, wire all stub features
**Approach:** Option B — foundations first, then feature wiring

---

## Context

Resikno-Mac is at v0.2.0. The CLI skeleton, TUI, scanner, safety system, and restore manifest exist. However, several features are stubs ("coming soon") and a few critical bugs exist. This spec covers everything needed to make the existing feature surface fully functional.

---

## Section 1: Foundation Fixes

### 1.1 Fix restore directory typo
**File:** `src/cleaner/backup.rs:36`
Change `~/.resikno-mak` → `~/.resikno-mac`. All restore points currently write to the wrong directory.

### 1.2 Config persistence
**New file:** `src/config.rs`
Add `toml` crate dependency. Implement read/write for `~/.resikno-mac/config.toml`.

Supported keys:
- `default_restore_mode` — `"trash" | "backup" | "manifest"` (default: `"trash"`)
- `min_scan_size_mb` — u64, default `0`
- `safe_only_by_default` — bool, default `false`

The `config` command (`config <key> [value]`) reads/writes this file. All scan/clean commands load config as defaults before applying CLI flags.

### 1.3 Progress bars during scan
**File:** `src/scanner/mod.rs`
Use the existing `indicatif` dependency. Add a `MultiProgress` with one spinner per scan category in `run_full_scan`. Each category spinner shows the category name and updates to a checkmark with size found when complete.

### 1.4 Parallel scanning
**File:** `src/scanner/mod.rs`
The 6 scan categories (caches, app caches, logs, temp, iOS backups, Xcode, downloads) are fully independent. Collect results from each into a `Vec` using `rayon::par_iter` / `rayon::scope`. Merge results after all threads complete. Uses the existing `rayon` dependency.

### 1.5 Remove unused tokio dependency
**File:** `Cargo.toml`
Remove `tokio`. Nothing in the codebase is async. Cuts compile time and binary size.

---

## Section 2: Restore UX

### 2.1 RestoreMode enum
**File:** `src/cleaner/mod.rs`

```rust
pub enum RestoreMode {
    Trash,         // Move to macOS Trash (recommended default)
    Backup,        // Copy files to ~/.resikno-mac/restore/<id>/ before delete
    ManifestOnly,  // Log what was deleted, no file recovery
}
```

Add `restore_mode: RestoreMode` to `CleanupOptions`.

### 2.2 Interactive confirmation flow
**File:** `src/ui/mod.rs` (in `handle_cleanup`) and `src/cli/commands.rs` (in `clean --execute`)

The flow has up to 3 prompts:

**Prompt 1 — Mode selection (always shown):**
```
Before deleting N items (X GB), choose restore behavior:

  [1] Move to Trash      — recoverable via macOS Trash (recommended)
  [2] Backup files       — copy to ~/.resikno-mac/restore/ before delete
  [3] Manifest only      — log what was deleted, no file recovery
  [4] Cancel

Choice [1]:
```
Default config key `default_restore_mode` pre-selects the default option.

**Prompt 2 — Only for Backup mode:**
```
Backing up N items (X GB) requires free disk space.
Destination: ~/.resikno-mac/restore/2026-03-28_143022/
Continue? [y/N]:
```

**Prompt 3 — Final confirmation (always shown):**
```
Ready to delete N items (X GB).
Restore mode: Move to Trash
Proceed? [y/N]:
```

### 2.3 Delete implementation by mode
**File:** `src/cleaner/delete.rs`

- `Trash`: use the `trash` crate (new dependency, wraps macOS `NSFileManager trashItemAtURL`). Add `trash = "3"` to `Cargo.toml`.
- `Backup`: copy each file/dir into `~/.resikno-mac/restore/<id>/<original_path_hash>/` preserving the filename, then delete. Use `fs::copy` for files and `fs_extra::dir::copy` (already a dep) for directories. Write manifest.
- `ManifestOnly`: current behavior — write manifest, then delete.

### 2.4 Restore command implementation
**File:** `src/cli/commands.rs` (`Commands::Restore`)

- For `Backup` mode restore points: read manifest, `fs::rename` files from restore dir back to original paths. If original path exists, prompt: overwrite / skip / abort.
- For `Trash` / `ManifestOnly` restore points: display manifest contents, explain that files are not recoverable through the tool (point to Trash or Time Machine).

---

## Section 3: Feature Wiring

### 3.1 Duplicate detection
**Files:** `src/cli/commands.rs`, `src/ui/mod.rs`

`duplicates.rs` is complete. Wire it up:
- `analyze --duplicates`: call `find_duplicates(&[home_dir], min_size: 1_000_000)` (default 1MB minimum to skip tiny files). Display groups sorted by recoverable space descending. Show file count, total recoverable, and up to 5 groups with their file paths.
- Add `CleanupCategory::Duplicates` items to `run_full_scan` results so TUI can show them. Heuristic for "keep": oldest file by `created` time = original.

### 3.2 Old file detection
**New file:** `src/scanner/old_files.rs`

Walk `~/Downloads` and `~/Desktop`. Filter files where `last_accessed < now - days`. Return `Vec<ScannedItem>` with `CleanupCategory::Downloads` (for Downloads dir) or a new `CleanupCategory::OldFiles` (for Desktop — caution level).

Wire to:
- `analyze --old <DAYS>`: run the scan, display results grouped by directory.
- `run_full_scan`: add as optional pass (only when a future config flag `include_old_files` is set, or via CLI flag).

### 3.3 Language files scanner
**New file:** `src/scanner/language_files.rs`

Walk `/Applications/` for `*.app` bundles. For each app, find `Contents/Resources/*.lproj` directories. Collect all `.lproj` dirs that are NOT the user's current locale (read from `LANG` or `LC_ALL` env var, default `en`). Return size and path for each.

Wire into `run_full_scan` as `CleanupCategory::LanguageFiles` with `SafetyLevel::MostlySafe`.

### 3.4 Custom path scanning
**File:** `src/scanner/mod.rs`

`_custom_path` is currently ignored. When `Some(path)`:
- Skip all platform-default directories
- Walk only the given path
- Classify items by category based on path patterns (cache/log/temp/other)
- Apply size filters as normal

### 3.5 Actual restore
**File:** `src/cli/commands.rs` (`Commands::Restore`)

For `Backup` mode restore points (detected by presence of actual files in restore dir alongside `manifest.json`):
1. Read manifest
2. For each entry, check if file exists in restore dir
3. `fs::rename` back to original path (prompt on conflict)
4. Remove restore dir after successful restore
5. Print summary

---

## Bug Fixes

| Location | Bug | Fix |
|----------|-----|-----|
| `backup.rs:36` | Typo: `.resikno-mak` | Change to `.resikno-mac` |
| `Cargo.toml` | `tokio` unused | Remove dependency |
| `scanner/mod.rs` | `_custom_path` ignored | Implement custom path logic |
| `cleaner/backup.rs` | `backed_up: false` hardcoded | Set based on actual backup status |

---

## New Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `toml` | `1` | Config file parsing |
| `trash` | `3` | macOS Trash integration |

---

## Future Plans

See `docs/future-plans.md` for planned features not in this spec.
