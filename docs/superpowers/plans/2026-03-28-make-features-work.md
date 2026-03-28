# Resikno-Mac: Make All Features Work — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix all stub features and bugs so every existing command works end-to-end.

**Architecture:** Foundation fixes first (typo, config, progress, parallelism), then restore UX (RestoreMode enum + confirmation flow + actual backup/trash), then wire stub features (duplicates, old files, language files, custom path, restore command).

**Tech Stack:** Rust, ratatui, crossterm, rayon, indicatif, sha2, walkdir, serde/toml (new), trash crate (new)

---

## File Map

| File | Change |
|------|--------|
| `src/cleaner/backup.rs` | Fix typo `.resikno-mak` → `.resikno-mac` |
| `src/platform/mod.rs` | Fix typo in `config_dir` docstring |
| `Cargo.toml` | Remove `tokio`, add `toml = "1"`, add `trash = "3"` |
| `src/config.rs` | **NEW** — read/write `~/.resikno-mac/config.toml` |
| `src/main.rs` | Add `mod config;` |
| `src/cleaner/mod.rs` | Add `RestoreMode` enum, add to `CleanupOptions`, pass to `cleanup()` |
| `src/cleaner/prompt.rs` | **NEW** — shared restore confirmation flow (3 prompts) |
| `src/cleaner/delete.rs` | Use `trash` crate for Trash mode; copy files for Backup mode |
| `src/cleaner/backup.rs` | Actually copy files when `RestoreMode::Backup` |
| `src/cli/commands.rs` | Wire `config` command; wire `analyze --duplicates`/`--old`; implement `restore` |
| `src/ui/mod.rs` | Replace `handle_cleanup` prompts with shared prompt flow |
| `src/scanner/mod.rs` | Progress bars, parallel scanning, implement `custom_path`, add duplicates to full scan |
| `src/scanner/old_files.rs` | **NEW** — find files not accessed in N days |
| `src/scanner/language_files.rs` | **NEW** — find non-locale `.lproj` bundles in `/Applications/` |

---

## Task 1: Quick Fixes — Typo and Tokio

**Files:**
- Modify: `src/cleaner/backup.rs:36`
- Modify: `Cargo.toml`

- [ ] **Step 1: Fix the restore directory typo**

In `src/cleaner/backup.rs`, line 36, change:
```rust
Ok(home.join(".resikno-mak").join("restore"))
```
to:
```rust
Ok(home.join(".resikno-mac").join("restore"))
```

- [ ] **Step 2: Remove tokio from Cargo.toml**

In `Cargo.toml`, remove these lines entirely:
```toml
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 3: Verify it builds**

```bash
cargo build 2>&1
```
Expected: compiles without error. No `tokio` in dependency list.

- [ ] **Step 4: Commit**

```bash
git add src/cleaner/backup.rs Cargo.toml
git commit -m "fix: correct restore dir typo and remove unused tokio dep"
```

---

## Task 2: Config System

**Files:**
- Create: `src/config.rs`
- Modify: `src/main.rs`
- Modify: `Cargo.toml`
- Modify: `src/cli/commands.rs`

- [ ] **Step 1: Add toml dependency**

In `Cargo.toml` under `[dependencies]`, add:
```toml
toml = "1"
```

- [ ] **Step 2: Write the failing test for config load/save**

Create `src/config.rs` with this test at the bottom:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_default() {
        let cfg = Config::default();
        assert_eq!(cfg.default_restore_mode, "trash");
        assert_eq!(cfg.min_scan_size_mb, 0);
        assert!(!cfg.safe_only_by_default);
    }

    #[test]
    fn test_config_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let cfg = Config {
            default_restore_mode: "backup".to_string(),
            min_scan_size_mb: 10,
            safe_only_by_default: true,
        };

        save_to(&cfg, &path).unwrap();
        let loaded = load_from(&path).unwrap();

        assert_eq!(loaded.default_restore_mode, "backup");
        assert_eq!(loaded.min_scan_size_mb, 10);
        assert!(loaded.safe_only_by_default);
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

```bash
cargo test test_config 2>&1
```
Expected: FAIL — `config` module not found.

- [ ] **Step 4: Implement config.rs**

Write the full `src/config.rs`:
```rust
//! User configuration — reads/writes ~/.resikno-mac/config.toml

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Restore mode: "trash" | "backup" | "manifest"
    #[serde(default = "default_restore_mode")]
    pub default_restore_mode: String,
    /// Minimum file size in MB for scans (0 = no minimum)
    #[serde(default)]
    pub min_scan_size_mb: u64,
    /// Only clean Safe-rated items by default
    #[serde(default)]
    pub safe_only_by_default: bool,
}

fn default_restore_mode() -> String {
    "trash".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_restore_mode: default_restore_mode(),
            min_scan_size_mb: 0,
            safe_only_by_default: false,
        }
    }
}

/// Path to the config file
pub fn config_path() -> Result<PathBuf> {
    let home = directories::UserDirs::new()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .home_dir()
        .to_path_buf();
    Ok(home.join(".resikno-mac").join("config.toml"))
}

/// Load config from the default path, returning defaults if not found
pub fn load() -> Result<Config> {
    let path = config_path()?;
    load_from(&path)
}

/// Load config from a specific path
pub fn load_from(path: &Path) -> Result<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let contents = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&contents)?)
}

/// Save config to the default path
pub fn save(config: &Config) -> Result<()> {
    let path = config_path()?;
    save_to(config, &path)
}

/// Save config to a specific path
pub fn save_to(config: &Config, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let contents = toml::to_string_pretty(config)?;
    std::fs::write(path, contents)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_default() {
        let cfg = Config::default();
        assert_eq!(cfg.default_restore_mode, "trash");
        assert_eq!(cfg.min_scan_size_mb, 0);
        assert!(!cfg.safe_only_by_default);
    }

    #[test]
    fn test_config_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let cfg = Config {
            default_restore_mode: "backup".to_string(),
            min_scan_size_mb: 10,
            safe_only_by_default: true,
        };

        save_to(&cfg, &path).unwrap();
        let loaded = load_from(&path).unwrap();

        assert_eq!(loaded.default_restore_mode, "backup");
        assert_eq!(loaded.min_scan_size_mb, 10);
        assert!(loaded.safe_only_by_default);
    }

    #[test]
    fn test_config_missing_file_returns_defaults() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nonexistent.toml");
        let cfg = load_from(&path).unwrap();
        assert_eq!(cfg.default_restore_mode, "trash");
    }
}
```

- [ ] **Step 5: Add mod config to main.rs**

In `src/main.rs`, after `mod cli;`, add:
```rust
mod config;
```

- [ ] **Step 6: Wire config command in commands.rs**

In `src/cli/commands.rs`, replace the `Commands::Config` arm (currently just prints):
```rust
Commands::Config { key, value } => {
    let mut cfg = crate::config::load()?;
    match (key, value) {
        (Some(k), Some(v)) => {
            match k.as_str() {
                "default_restore_mode" => {
                    if !["trash", "backup", "manifest"].contains(&v.as_str()) {
                        anyhow::bail!("Invalid restore mode. Use: trash, backup, or manifest");
                    }
                    cfg.default_restore_mode = v.clone();
                }
                "min_scan_size_mb" => {
                    cfg.min_scan_size_mb = v.parse::<u64>()
                        .map_err(|_| anyhow::anyhow!("min_scan_size_mb must be a number"))?;
                }
                "safe_only_by_default" => {
                    cfg.safe_only_by_default = matches!(v.as_str(), "true" | "1" | "yes");
                }
                _ => anyhow::bail!("Unknown config key: {}. Valid keys: default_restore_mode, min_scan_size_mb, safe_only_by_default", k),
            }
            crate::config::save(&cfg)?;
            println!("Set {} = {}", k, v);
        }
        (Some(k), None) => {
            match k.as_str() {
                "default_restore_mode" => println!("{}", cfg.default_restore_mode),
                "min_scan_size_mb" => println!("{}", cfg.min_scan_size_mb),
                "safe_only_by_default" => println!("{}", cfg.safe_only_by_default),
                _ => anyhow::bail!("Unknown config key: {}", k),
            }
        }
        _ => {
            println!("Config file: {}", crate::config::config_path()?.display());
            println!();
            println!("default_restore_mode = {}", cfg.default_restore_mode);
            println!("min_scan_size_mb     = {}", cfg.min_scan_size_mb);
            println!("safe_only_by_default = {}", cfg.safe_only_by_default);
            println!();
            println!("Set a value with: resikno config <key> <value>");
        }
    }
}
```

- [ ] **Step 7: Run tests**

```bash
cargo test test_config 2>&1
```
Expected: 3 tests pass.

- [ ] **Step 8: Commit**

```bash
git add src/config.rs src/main.rs src/cli/commands.rs Cargo.toml Cargo.lock
git commit -m "feat: add persistent config system with get/set commands"
```

---

## Task 3: RestoreMode Enum + CleanupOptions

**Files:**
- Modify: `src/cleaner/mod.rs`

- [ ] **Step 1: Write failing test**

Add to the bottom of `src/cleaner/mod.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restore_mode_from_str() {
        assert!(matches!(RestoreMode::from_str("trash"), RestoreMode::Trash));
        assert!(matches!(RestoreMode::from_str("backup"), RestoreMode::Backup));
        assert!(matches!(RestoreMode::from_str("manifest"), RestoreMode::ManifestOnly));
        assert!(matches!(RestoreMode::from_str("unknown"), RestoreMode::Trash)); // default
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test test_restore_mode 2>&1
```
Expected: FAIL — `RestoreMode` not found.

- [ ] **Step 3: Add RestoreMode enum and update CleanupOptions**

In `src/cleaner/mod.rs`, add after the imports:
```rust
/// How to handle files before deletion
#[derive(Debug, Clone, PartialEq)]
pub enum RestoreMode {
    /// Move files to macOS Trash (recoverable, recommended default)
    Trash,
    /// Copy files to ~/.resikno-mac/restore/<id>/ before deleting
    Backup,
    /// Only write a manifest log — files are not recoverable through this tool
    ManifestOnly,
}

impl RestoreMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "backup" => RestoreMode::Backup,
            "manifest" => RestoreMode::ManifestOnly,
            _ => RestoreMode::Trash,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            RestoreMode::Trash => "Move to Trash",
            RestoreMode::Backup => "Backup files to ~/.resikno-mac/restore/",
            RestoreMode::ManifestOnly => "Manifest only (no file recovery)",
        }
    }
}
```

Replace `CleanupOptions` struct with:
```rust
/// Options for cleanup operations
#[derive(Debug, Clone)]
pub struct CleanupOptions {
    /// Actually delete files (false = dry run)
    pub execute: bool,
    /// How to handle files before deletion
    pub restore_mode: RestoreMode,
    /// Only process items marked as SAFE
    pub safe_only: bool,
    /// Skip confirmation prompts
    pub force: bool,
}

impl Default for CleanupOptions {
    fn default() -> Self {
        Self {
            execute: false,
            restore_mode: RestoreMode::Trash,
            safe_only: false,
            force: false,
        }
    }
}
```

Update the `cleanup()` function signature and body to pass `restore_mode` to the backup/delete logic:
```rust
pub fn cleanup(paths: &[PathBuf], options: &CleanupOptions) -> Result<CleanupResult> {
    let mut result = CleanupResult {
        items_deleted: 0,
        bytes_freed: 0,
        restore_point: None,
        errors: Vec::new(),
    };

    if options.execute {
        // Create restore point / backup depending on mode
        result.restore_point = Some(backup::create_restore_point(paths, &options.restore_mode)?);

        for path in paths {
            match delete::delete_by_mode(path, &options.restore_mode) {
                Ok(size) => {
                    result.items_deleted += 1;
                    result.bytes_freed += size;
                }
                Err(e) => {
                    result.errors.push(CleanupError {
                        path: path.clone(),
                        message: e.to_string(),
                    });
                }
            }
        }
    }

    Ok(result)
}
```

- [ ] **Step 4: Run test**

```bash
cargo test test_restore_mode 2>&1
```
Expected: 1 test passes. (Build will fail on callers of `CleanupOptions` — fix those next.)

- [ ] **Step 5: Fix all CleanupOptions callers**

Search for all construction sites of `CleanupOptions`:
```bash
grep -rn "CleanupOptions {" src/
```

For each found location, update to use `restore_mode` instead of `create_restore_point`. Example in `src/cli/commands.rs`:

Change:
```rust
let options = CleanupOptions {
    execute: true,
    create_restore_point: true,
    safe_only,
    force: false,
};
```
To:
```rust
let cfg = crate::config::load().unwrap_or_default();
let options = CleanupOptions {
    execute: true,
    restore_mode: RestoreMode::from_str(&cfg.default_restore_mode),
    safe_only,
    force: false,
};
```

Do the same for all other `CleanupOptions { ... }` constructions in `src/ui/mod.rs`.

- [ ] **Step 6: Verify build**

```bash
cargo build 2>&1
```
Expected: compiles. (Some functions like `delete_by_mode` and updated `create_restore_point` don't exist yet — stubs are fine, add placeholder `todo!()` if needed to get it to compile.)

- [ ] **Step 7: Commit**

```bash
git add src/cleaner/mod.rs src/cli/commands.rs src/ui/mod.rs
git commit -m "feat: add RestoreMode enum and update CleanupOptions"
```

---

## Task 4: Restore Confirmation Prompt Flow

**Files:**
- Create: `src/cleaner/prompt.rs`
- Modify: `src/cleaner/mod.rs` (add `pub mod prompt;`)
- Modify: `src/ui/mod.rs` (replace `handle_cleanup` prompt logic)
- Modify: `src/cli/commands.rs` (use prompt in `clean --execute`)

- [ ] **Step 1: Create src/cleaner/prompt.rs**

```rust
//! Interactive restore-mode confirmation prompts

use std::io::{self, Write};
use anyhow::Result;
use bytesize::ByteSize;
use super::RestoreMode;

/// Ask the user to choose a restore mode and confirm cleanup.
/// Returns `None` if the user cancels, `Some(RestoreMode)` if they confirm.
pub fn confirm_cleanup(item_count: usize, total_size: u64, default_mode: &RestoreMode) -> Result<Option<RestoreMode>> {
    let default_choice = match default_mode {
        RestoreMode::Trash => 1,
        RestoreMode::Backup => 2,
        RestoreMode::ManifestOnly => 3,
    };

    // Prompt 1: mode selection
    println!();
    println!("Before deleting {} items ({}), choose restore behavior:", item_count, ByteSize(total_size));
    println!();
    println!("  [1] Move to Trash      — recoverable via macOS Trash (recommended)");
    println!("  [2] Backup files       — copy to ~/.resikno-mac/restore/ before delete");
    println!("  [3] Manifest only      — log what was deleted, no file recovery");
    println!("  [4] Cancel");
    println!();
    print!("Choice [{}]: ", default_choice);
    io::stdout().flush()?;

    let mode = loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();
        match trimmed {
            "" => break default_mode.clone(),
            "1" => break RestoreMode::Trash,
            "2" => break RestoreMode::Backup,
            "3" => break RestoreMode::ManifestOnly,
            "4" | "q" | "Q" => return Ok(None),
            _ => {
                print!("Please enter 1, 2, 3, or 4: ");
                io::stdout().flush()?;
            }
        }
    };

    // Prompt 2: extra confirmation for Backup mode (disk space warning)
    if mode == RestoreMode::Backup {
        println!();
        println!("Backing up {} to ~/.resikno-mac/restore/ requires free disk space.", ByteSize(total_size));
        print!("Continue? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            return Ok(None);
        }
    }

    // Prompt 3: final confirmation (always shown)
    println!();
    println!("Ready to delete {} items ({}).", item_count, ByteSize(total_size));
    println!("Restore mode: {}", mode.label());
    print!("Proceed? [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if !input.trim().eq_ignore_ascii_case("y") {
        return Ok(None);
    }

    Ok(Some(mode))
}
```

- [ ] **Step 2: Add pub mod prompt to cleaner/mod.rs**

In `src/cleaner/mod.rs`, after `pub mod delete;`, add:
```rust
pub mod prompt;
```

- [ ] **Step 3: Update handle_cleanup in src/ui/mod.rs**

Replace the confirmation block in `handle_cleanup` (the section that prints the box and reads `y/N`) with:
```rust
fn handle_cleanup(paths: &[PathBuf], total_size: u64) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }

    let cfg = crate::config::load().unwrap_or_default();
    let default_mode = cleaner::RestoreMode::from_str(&cfg.default_restore_mode);

    let mode = match cleaner::prompt::confirm_cleanup(paths.len(), total_size, &default_mode)? {
        Some(m) => m,
        None => {
            println!("Cleanup cancelled.");
            return Ok(());
        }
    };

    println!();
    println!("Creating restore point...");

    let options = cleaner::CleanupOptions {
        execute: true,
        restore_mode: mode,
        safe_only: false,
        force: false,
    };

    match cleaner::cleanup(paths, &options) {
        Ok(result) => {
            // Verify which files were actually deleted
            let mut actually_deleted: Vec<&PathBuf> = Vec::new();
            let mut still_exists: Vec<&PathBuf> = Vec::new();
            for path in paths {
                if path.exists() { still_exists.push(path); } else { actually_deleted.push(path); }
            }

            println!();
            println!("Cleanup Complete");
            println!("Deleted: {} items", result.items_deleted);
            println!("Freed:   {}", ByteSize(result.bytes_freed));
            if let Some(restore) = &result.restore_point {
                println!("Restore: {}", restore.id);
            }

            for path in actually_deleted.iter().take(10) {
                let name = path.file_name().map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.display().to_string());
                println!("  ✓ {}", name);
            }
            if actually_deleted.len() > 10 {
                println!("  ... and {} more", actually_deleted.len() - 10);
            }
            if !still_exists.is_empty() {
                println!("Warning: {} files could not be verified as deleted", still_exists.len());
            }
            if !result.errors.is_empty() {
                println!("Errors ({}): ", result.errors.len());
                for err in result.errors.iter().take(5) {
                    let name = err.path.file_name().map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| err.path.display().to_string());
                    println!("   {}: {}", name, err.message);
                }
            }
        }
        Err(e) => {
            println!();
            println!("Cleanup failed: {}", e);
        }
    }

    Ok(())
}
```

- [ ] **Step 4: Update clean --execute in commands.rs**

In `Commands::Clean { ... }` arm, when `execute` is true, replace the hardcoded prompt with:
```rust
let cfg = crate::config::load().unwrap_or_default();
let default_mode = cleaner::RestoreMode::from_str(&cfg.default_restore_mode);
let total_size: u64 = items_to_clean.iter().map(|i| i.size).sum();
let paths: Vec<_> = items_to_clean.iter().map(|i| i.path.clone()).collect();

let mode = match crate::cleaner::prompt::confirm_cleanup(paths.len(), total_size, &default_mode)? {
    Some(m) => m,
    None => {
        println!("Cleanup cancelled.");
        return Ok(());
    }
};

let options = CleanupOptions {
    execute: true,
    restore_mode: mode,
    safe_only,
    force: false,
};

let result = cleaner::cleanup(&paths, &options)?;
// ... rest of result display unchanged
```

- [ ] **Step 5: Build and manual test**

```bash
cargo build 2>&1
```
Expected: compiles cleanly.

Manual test (dry run, no files deleted):
```bash
cargo run -- clean caches
```
Expected: shows prompt flow asking for restore mode.

- [ ] **Step 6: Commit**

```bash
git add src/cleaner/prompt.rs src/cleaner/mod.rs src/ui/mod.rs src/cli/commands.rs
git commit -m "feat: add interactive restore-mode confirmation flow"
```

---

## Task 5: Delete by Mode (Trash Crate + Actual Backup)

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/cleaner/delete.rs`
- Modify: `src/cleaner/backup.rs`

- [ ] **Step 1: Add trash dependency**

In `Cargo.toml` under `[dependencies]`, add:
```toml
trash = "3"
```

- [ ] **Step 2: Add delete_by_mode to delete.rs**

Add this function to `src/cleaner/delete.rs` (after the existing `safe_delete` function):
```rust
use crate::cleaner::RestoreMode;

/// Delete a path using the chosen restore mode.
/// For Trash: moves to macOS Trash.
/// For Backup: files are already copied by create_restore_point — just delete.
/// For ManifestOnly: permanently deletes after manifest is written.
pub fn delete_by_mode(path: &Path, mode: &RestoreMode) -> Result<u64> {
    // Safety checks apply regardless of mode
    match check_deletion_safety(path)? {
        SafetyCheckResult::Blocked(reason) => anyhow::bail!("{}", reason),
        SafetyCheckResult::RequiresConfirmation(reason) => eprintln!("Warning: {}", reason),
        SafetyCheckResult::Safe => {}
    }

    let size = calculate_dir_size(path)?;

    match mode {
        RestoreMode::Trash => {
            trash::delete(path)
                .with_context(|| format!("Failed to move to Trash: {}", path.display()))?;
        }
        RestoreMode::Backup | RestoreMode::ManifestOnly => {
            // Files already backed up (or backup skipped). Permanently delete.
            if path.is_file() {
                fs::remove_file(path)
                    .with_context(|| format!("Failed to delete: {}", path.display()))?;
            } else {
                fs::remove_dir_all(path)
                    .with_context(|| format!("Failed to delete directory: {}", path.display()))?;
            }
        }
    }

    Ok(size)
}
```

- [ ] **Step 3: Update backup.rs to accept RestoreMode**

Update `create_restore_point` signature in `src/cleaner/backup.rs` to accept mode:
```rust
use crate::cleaner::RestoreMode;

/// Create a restore point before deletion.
/// For Backup mode: actually copies files.
/// For Trash/ManifestOnly: writes manifest only.
pub fn create_restore_point(paths: &[PathBuf], mode: &RestoreMode) -> Result<RestorePoint> {
    let timestamp = Utc::now();
    let id = timestamp.format("%Y-%m-%d_%H%M%S").to_string();

    let restore_path = restore_dir()?.join(&id);
    fs::create_dir_all(&restore_path)?;

    let mut manifest = Manifest {
        timestamp,
        total_size: 0,
        items: Vec::new(),
    };

    for path in paths {
        if let Ok(metadata) = path.metadata() {
            let size = if metadata.is_file() {
                metadata.len()
            } else {
                calculate_dir_size(path)?
            };

            // For Backup mode: actually copy the file/dir
            let backed_up = if matches!(mode, RestoreMode::Backup) {
                copy_to_restore(&restore_path, path).is_ok()
            } else {
                false
            };

            manifest.total_size += size;
            manifest.items.push(ManifestEntry {
                path: path.clone(),
                size,
                category: categorize_path(path),
                backed_up,
                reason: get_deletion_reason(path),
            });
        }
    }

    let manifest_path = restore_path.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, manifest_json)?;

    Ok(RestorePoint {
        id,
        timestamp,
        manifest_path,
        total_size: manifest.total_size,
        item_count: manifest.items.len(),
    })
}

/// Copy a file or directory into the restore point directory.
/// Preserves the original filename.
fn copy_to_restore(restore_path: &PathBuf, source: &PathBuf) -> Result<()> {
    let filename = source.file_name()
        .ok_or_else(|| anyhow::anyhow!("Path has no filename: {}", source.display()))?;
    let dest = restore_path.join(filename);

    if source.is_file() {
        fs::copy(source, &dest)?;
    } else {
        let options = fs_extra::dir::CopyOptions::new();
        fs_extra::dir::copy(source, restore_path, &options)?;
    }
    Ok(())
}
```

Also add the `calculate_dir_size` helper that's called above (copy from the existing one in the file if already present, otherwise):
```rust
fn calculate_dir_size(path: &PathBuf) -> Result<u64> {
    let mut total = 0;
    for entry in walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            total += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    Ok(total)
}
```

- [ ] **Step 4: Build**

```bash
cargo build 2>&1
```
Expected: compiles. Fix any import/type errors.

- [ ] **Step 5: Run existing tests**

```bash
cargo test 2>&1
```
Expected: all existing tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/cleaner/delete.rs src/cleaner/backup.rs Cargo.toml Cargo.lock
git commit -m "feat: implement delete by restore mode (Trash/Backup/ManifestOnly)"
```

---

## Task 6: Restore Command Implementation

**Files:**
- Modify: `src/cli/commands.rs`

- [ ] **Step 1: Implement restore for Backup mode in commands.rs**

Replace the `Commands::Restore` arm with:
```rust
Commands::Restore { date, latest, list } => {
    let restore_points = backup::list_restore_points()?;

    if list || (date.is_none() && !latest) {
        if restore_points.is_empty() {
            println!("No restore points found.");
            println!("\nRestore points are created when you run 'resikno clean --execute'");
        } else {
            println!("Available restore points:\n");
            for point in &restore_points {
                // Detect if it has actual backed-up files (Backup mode) or is manifest-only
                let has_files = point.manifest_path.parent()
                    .map(|p| p.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false))
                    .unwrap_or(false);
                let mode_label = if has_files { "backup" } else { "manifest" };
                println!("  {} — {} items, {} [{}]",
                    point.id,
                    point.item_count,
                    ByteSize(point.total_size),
                    mode_label);
            }
            println!("\nTo restore: resikno restore <date> or resikno restore --latest");
        }
        return Ok(());
    }

    let target = if latest {
        restore_points.first()
    } else if let Some(ref d) = date {
        restore_points.iter().find(|p| p.id.starts_with(d))
    } else {
        None
    };

    let point = match target {
        Some(p) => p,
        None => {
            println!("Restore point not found.");
            if !restore_points.is_empty() {
                println!("\nAvailable restore points:");
                for p in restore_points.iter().take(5) {
                    println!("  - {}", p.id);
                }
            }
            return Ok(());
        }
    };

    // Read manifest
    let manifest_json = std::fs::read_to_string(&point.manifest_path)?;
    let manifest: backup::Manifest = serde_json::from_str(&manifest_json)?;

    // Check if files were actually backed up
    let restore_dir = point.manifest_path.parent().unwrap().to_path_buf();
    let backed_up_entries: Vec<_> = manifest.items.iter().filter(|e| e.backed_up).collect();

    if backed_up_entries.is_empty() {
        println!("This restore point is manifest-only — files were not backed up.");
        println!("\nDeleted files ({}):", manifest.items.len());
        for entry in manifest.items.iter().take(20) {
            println!("  {} ({})", entry.path.display(), ByteSize(entry.size));
        }
        if manifest.items.len() > 20 {
            println!("  ... and {} more", manifest.items.len() - 20);
        }
        println!("\nTo recover these files, check your macOS Trash or Time Machine.");
        return Ok(());
    }

    println!("Restoring {} files from {}...\n", backed_up_entries.len(), point.id);

    let mut restored = 0;
    let mut skipped = 0;
    let mut errors = 0;

    for entry in &backed_up_entries {
        let filename = entry.path.file_name()
            .ok_or_else(|| anyhow::anyhow!("No filename in path"))?;
        let backed_up_path = restore_dir.join(filename);

        if !backed_up_path.exists() {
            println!("  MISSING: {} (backup file not found)", entry.path.display());
            errors += 1;
            continue;
        }

        if entry.path.exists() {
            print!("  EXISTS: {} already exists. [o]verwrite / [s]kip? [s]: ", entry.path.display());
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("o") {
                skipped += 1;
                continue;
            }
        }

        // Restore: move back to original path
        if let Some(parent) = entry.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        match std::fs::rename(&backed_up_path, &entry.path) {
            Ok(_) => {
                println!("  RESTORED: {}", entry.path.display());
                restored += 1;
            }
            Err(e) => {
                println!("  ERROR: {} — {}", entry.path.display(), e);
                errors += 1;
            }
        }
    }

    println!("\nDone: {} restored, {} skipped, {} errors", restored, skipped, errors);

    // Clean up restore dir if fully restored
    if errors == 0 && skipped == 0 {
        let _ = std::fs::remove_dir_all(&restore_dir);
        println!("Restore point {} removed.", point.id);
    }
}
```

- [ ] **Step 2: Build**

```bash
cargo build 2>&1
```
Expected: compiles cleanly.

- [ ] **Step 3: Commit**

```bash
git add src/cli/commands.rs
git commit -m "feat: implement restore command for Backup mode restore points"
```

---

## Task 7: Progress Bars + Parallel Scanning

**Files:**
- Modify: `src/scanner/mod.rs`

- [ ] **Step 1: Write test for parallel scan producing same results**

Add to the `#[cfg(test)]` block in `src/scanner/mod.rs`:
```rust
#[test]
fn test_run_full_scan_returns_results() {
    // This is an integration-style test — just verify it runs without panic
    // and returns a valid (possibly empty) result on a temp directory.
    use tempfile::tempdir;
    use crate::platform::macos::MacOSPaths;

    let platform = MacOSPaths::new();
    // Use a size filter so large real scans don't slow CI
    let result = run_full_scan(&platform, None, 9999, 0);
    assert!(result.is_ok(), "scan should not error: {:?}", result.err());
}
```

- [ ] **Step 2: Run test to verify it passes already (baseline)**

```bash
cargo test test_run_full_scan 2>&1
```
Expected: PASS (baseline before refactor).

- [ ] **Step 3: Refactor run_full_scan with progress bars and parallel scanning**

Replace the body of `run_full_scan` in `src/scanner/mod.rs` with:
```rust
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;

pub fn run_full_scan<P: PlatformPaths>(
    platform: &P,
    custom_path: Option<&Path>,
    min_size_mb: u64,
    max_size_mb: u64,
) -> Result<ScanResults> {
    let min_bytes = min_size_mb * 1024 * 1024;
    let max_bytes = max_size_mb * 1024 * 1024;

    // If a custom path is given, scan only that path
    if let Some(root) = custom_path {
        return scan_custom_path(root, min_bytes, max_bytes);
    }

    // Phase 1: collect directory lists (fast, single-threaded, needs &platform)
    let cache_items_input = cache::scan_caches(platform).unwrap_or_default();
    let app_cache_dirs: Vec<PathBuf> = platform.app_cache_dirs().into_iter().filter(|d| d.exists()).collect();
    let log_dirs: Vec<PathBuf> = platform.log_dirs().into_iter().filter(|d| d.exists()).collect();
    let temp_dirs: Vec<PathBuf> = platform.temp_dirs().into_iter().filter(|d| d.exists()).collect();
    let mobile_dirs: Vec<PathBuf> = platform.mobile_backup_dirs().into_iter().filter(|d| d.exists()).collect();
    let xcode_dirs: Vec<PathBuf> = platform.xcode_dirs().into_iter().filter(|d| d.exists()).collect();
    let downloads_dir = platform.downloads_dir();

    // Phase 2: define scan jobs
    struct ScanJob {
        name: &'static str,
        dirs: Vec<PathBuf>,
        category: CleanupCategory,
    }

    let jobs = vec![
        ScanJob { name: "App caches",   dirs: app_cache_dirs, category: CleanupCategory::AppCaches },
        ScanJob { name: "Logs",         dirs: log_dirs,       category: CleanupCategory::Logs },
        ScanJob { name: "Temp files",   dirs: temp_dirs,      category: CleanupCategory::TempFiles },
        ScanJob { name: "iOS backups",  dirs: mobile_dirs,    category: CleanupCategory::IOSBackups },
        ScanJob { name: "Xcode data",   dirs: xcode_dirs,     category: CleanupCategory::XcodeData },
    ];

    // Phase 3: parallel scan with progress bars
    let mp = MultiProgress::new();
    let style = ProgressStyle::with_template("{spinner:.cyan} {msg}").unwrap();

    let mut all_items: Vec<ScannedItem> = Vec::new();

    // System caches: already have sizes, just filter
    let cache_items: Vec<ScannedItem> = cache_items_input
        .into_iter()
        .filter(|(_, size)| meets_size_filter(*size, min_bytes, max_bytes))
        .map(|(path, size)| ScannedItem {
            path, size,
            category: CleanupCategory::SystemCaches,
            last_accessed: None,
            last_modified: None,
        })
        .collect();
    all_items.extend(cache_items);

    // Category jobs: parallel
    let parallel_results: Vec<Vec<ScannedItem>> = jobs
        .into_par_iter()
        .map(|job| {
            let spinner = mp.add(ProgressBar::new_spinner());
            spinner.set_style(style.clone());
            spinner.set_message(format!("Scanning {}...", job.name));
            spinner.enable_steady_tick(std::time::Duration::from_millis(80));

            let mut items = Vec::new();
            for dir in &job.dirs {
                if let Ok(size) = calculate_dir_size(dir) {
                    if size > 0 && meets_size_filter(size, min_bytes, max_bytes) {
                        items.push(ScannedItem {
                            path: dir.clone(),
                            size,
                            category: job.category.clone(),
                            last_accessed: None,
                            last_modified: None,
                        });
                    }
                }
            }

            let total: u64 = items.iter().map(|i| i.size).sum();
            spinner.finish_with_message(format!("{}: {}", job.name, ByteSize(total)));
            items
        })
        .collect();

    for batch in parallel_results {
        all_items.extend(batch);
    }

    // Downloads: large files (sequential, separate logic)
    if let Some(dl) = downloads_dir {
        if dl.exists() {
            let spinner = mp.add(ProgressBar::new_spinner());
            spinner.set_style(style.clone());
            spinner.set_message("Scanning Downloads...");
            spinner.enable_steady_tick(std::time::Duration::from_millis(80));

            let large = large_files::find_large_files(&[dl], min_bytes, max_bytes).unwrap_or_default();
            let total: u64 = large.iter().map(|f| f.size).sum();
            spinner.finish_with_message(format!("Downloads: {}", ByteSize(total)));

            for file in large {
                all_items.push(ScannedItem {
                    path: file.path,
                    size: file.size,
                    category: CleanupCategory::Downloads,
                    last_accessed: file.last_accessed,
                    last_modified: file.last_modified,
                });
            }
        }
    }

    // Calculate totals
    let total_size = all_items.iter().map(|i| i.size).sum();
    let total_recoverable = all_items
        .iter()
        .filter(|i| matches!(i.category.safety_level(), SafetyLevel::Safe | SafetyLevel::MostlySafe))
        .map(|i| i.size)
        .sum();

    Ok(ScanResults { items: all_items, total_size, total_recoverable })
}

/// Scan a custom path, classifying items by path patterns
fn scan_custom_path(root: &Path, min_bytes: u64, max_bytes: u64) -> Result<ScanResults> {
    let mut items = Vec::new();

    for entry in walkdir::WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.depth() == 0 { continue; }
        let path = entry.path().to_path_buf();
        if let Ok(size) = calculate_dir_size(&path) {
            if size > 0 && meets_size_filter(size, min_bytes, max_bytes) {
                let path_str = path.to_string_lossy().to_lowercase();
                let category = if path_str.contains("cache") {
                    CleanupCategory::AppCaches
                } else if path_str.contains("log") {
                    CleanupCategory::Logs
                } else if path_str.contains("tmp") || path_str.contains("temp") {
                    CleanupCategory::TempFiles
                } else {
                    CleanupCategory::LargeFiles
                };
                items.push(ScannedItem { path, size, category, last_accessed: None, last_modified: None });
            }
        }
    }

    let total_size = items.iter().map(|i| i.size).sum();
    let total_recoverable = items.iter()
        .filter(|i| matches!(i.category.safety_level(), SafetyLevel::Safe | SafetyLevel::MostlySafe))
        .map(|i| i.size)
        .sum();

    Ok(ScanResults { items, total_size, total_recoverable })
}
```

- [ ] **Step 4: Run test again**

```bash
cargo test test_run_full_scan 2>&1
```
Expected: PASS.

- [ ] **Step 5: Full test suite**

```bash
cargo test 2>&1
```
Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/scanner/mod.rs
git commit -m "feat: add progress bars and parallel scanning with rayon"
```

---

## Task 8: Wire Duplicate Detection

**Files:**
- Modify: `src/cli/commands.rs`
- Modify: `src/scanner/mod.rs`

- [ ] **Step 1: Implement analyze --duplicates in commands.rs**

Replace the `if duplicates {` block in `Commands::Analyze` with:
```rust
if duplicates {
    println!("Scanning for duplicate files...\n");
    let platform = crate::platform::current();
    let home = crate::platform::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    let groups = crate::scanner::duplicates::find_duplicates(&[home], 1_000_000)?; // 1MB minimum

    if groups.is_empty() {
        println!("No duplicate files found (1 MB+ files only).");
    } else {
        let total_recoverable: u64 = groups.iter().map(|g| g.recoverable_space()).sum();
        println!("Found {} duplicate groups — {} recoverable\n", groups.len(), ByteSize(total_recoverable));

        // Sort by recoverable space descending
        let mut sorted = groups;
        sorted.sort_by(|a, b| b.recoverable_space().cmp(&a.recoverable_space()));

        for (i, group) in sorted.iter().take(10).enumerate() {
            println!("Group {} — {} copies, {} each, {} recoverable",
                i + 1, group.files.len(), ByteSize(group.size), ByteSize(group.recoverable_space()));
            for (j, file) in group.files.iter().enumerate() {
                let marker = if j == 0 { "KEEP" } else { "DUPE" };
                println!("  [{}] {}", marker, file.display());
            }
            println!();
        }

        if sorted.len() > 10 {
            println!("... and {} more groups", sorted.len() - 10);
        }

        println!("To clean duplicates, run with TUI: resikno scan");
    }
    let _ = platform; // suppress unused warning
}
```

- [ ] **Step 2: Add duplicates to run_full_scan**

In `run_full_scan` in `src/scanner/mod.rs`, after the downloads block (before calculating totals), add:
```rust
// Duplicates: scan home dir (slow — only run when no custom path)
if let Some(home) = crate::platform::home_dir() {
    let spinner = mp.add(ProgressBar::new_spinner());
    spinner.set_style(style.clone());
    spinner.set_message("Scanning for duplicates...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    if let Ok(groups) = duplicates::find_duplicates(&[home], 5_000_000) { // 5MB min for full scan
        let total: u64 = groups.iter().map(|g| g.recoverable_space()).sum();
        spinner.finish_with_message(format!("Duplicates: {} recoverable", ByteSize(total)));

        for group in groups {
            // Skip the first file (keep it), add the rest as duplicates
            for file in group.files.into_iter().skip(1) {
                if meets_size_filter(group.size, min_bytes, max_bytes) {
                    all_items.push(ScannedItem {
                        path: file,
                        size: group.size,
                        category: CleanupCategory::Duplicates,
                        last_accessed: None,
                        last_modified: None,
                    });
                }
            }
        }
    } else {
        spinner.finish_with_message("Duplicates: skipped");
    }
}
```

- [ ] **Step 3: Build and verify**

```bash
cargo build 2>&1
```
Expected: compiles.

- [ ] **Step 4: Commit**

```bash
git add src/cli/commands.rs src/scanner/mod.rs
git commit -m "feat: wire duplicate detection into analyze and full scan"
```

---

## Task 9: Old File Scanner

**Files:**
- Create: `src/scanner/old_files.rs`
- Modify: `src/scanner/mod.rs`
- Modify: `src/cli/commands.rs`

- [ ] **Step 1: Write failing test**

Create `src/scanner/old_files.rs` with only this test:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::time::{SystemTime, Duration};

    #[test]
    fn test_find_old_files_empty_when_recent() {
        let dir = tempdir().unwrap();
        // Create a file
        let file = dir.path().join("recent.txt");
        std::fs::write(&file, b"hello").unwrap();
        // File was just created — not old at all
        let results = find_old_files(&[dir.path().to_path_buf()], 1).unwrap();
        assert!(results.is_empty(), "freshly created file should not be 'old'");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test test_find_old_files 2>&1
```
Expected: FAIL — module not found.

- [ ] **Step 3: Implement old_files.rs**

```rust
//! Find files not accessed within N days

use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use anyhow::Result;

pub struct OldFile {
    pub path: PathBuf,
    pub size: u64,
    pub days_since_access: u64,
}

/// Find files in `dirs` not accessed within `older_than_days` days.
pub fn find_old_files(dirs: &[PathBuf], older_than_days: u64) -> Result<Vec<OldFile>> {
    let threshold = SystemTime::now()
        .checked_sub(Duration::from_secs(older_than_days * 86400))
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let mut results = Vec::new();

    for dir in dirs {
        if !dir.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let last_accessed = metadata.accessed().unwrap_or(SystemTime::UNIX_EPOCH);

            if last_accessed < threshold {
                let elapsed = last_accessed
                    .elapsed()
                    .unwrap_or(Duration::from_secs(0));
                let days = elapsed.as_secs() / 86400;

                results.push(OldFile {
                    path: entry.path().to_path_buf(),
                    size: metadata.len(),
                    days_since_access: days,
                });
            }
        }
    }

    // Sort by largest first
    results.sort_by(|a, b| b.size.cmp(&a.size));
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_find_old_files_empty_when_recent() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("recent.txt");
        std::fs::write(&file, b"hello").unwrap();
        let results = find_old_files(&[dir.path().to_path_buf()], 1).unwrap();
        assert!(results.is_empty(), "freshly created file should not be 'old'");
    }

    #[test]
    fn test_find_old_files_nonexistent_dir() {
        let results = find_old_files(&[PathBuf::from("/nonexistent/path/xyz")], 30).unwrap();
        assert!(results.is_empty());
    }
}
```

- [ ] **Step 4: Add pub mod old_files to scanner/mod.rs**

In `src/scanner/mod.rs`, after `pub mod large_files;`, add:
```rust
pub mod old_files;
```

- [ ] **Step 5: Wire to analyze --old in commands.rs**

Replace the `if let Some(days) = old {` block with:
```rust
if let Some(days) = old {
    println!("Scanning for files older than {} days...\n", days);
    let platform = crate::platform::current();

    let mut dirs = Vec::new();
    if let Some(dl) = platform.downloads_dir() { dirs.push(dl); }
    // Desktop: use home dir + "Desktop"
    if let Some(home) = crate::platform::home_dir() {
        let desktop = home.join("Desktop");
        if desktop.exists() { dirs.push(desktop); }
    }

    let old_files = crate::scanner::old_files::find_old_files(&dirs, days)?;

    if old_files.is_empty() {
        println!("No files found older than {} days in Downloads/Desktop.", days);
    } else {
        let total: u64 = old_files.iter().map(|f| f.size).sum();
        println!("Found {} files not accessed in {} days ({} total):\n",
            old_files.len(), days, ByteSize(total));

        for file in old_files.iter().take(20) {
            println!("  {} days  {}  ({})",
                file.days_since_access,
                file.path.display(),
                ByteSize(file.size));
        }
        if old_files.len() > 20 {
            println!("  ... and {} more", old_files.len() - 20);
        }
        println!("\nReview these files and delete manually, or use the TUI: resikno scan");
    }
    let _ = platform;
}
```

- [ ] **Step 6: Run tests**

```bash
cargo test test_find_old_files 2>&1
```
Expected: 2 tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/scanner/old_files.rs src/scanner/mod.rs src/cli/commands.rs
git commit -m "feat: add old file scanner and wire analyze --old command"
```

---

## Task 10: Language Files Scanner

**Files:**
- Create: `src/scanner/language_files.rs`
- Modify: `src/scanner/mod.rs`

- [ ] **Step 1: Write failing test**

Create `src/scanner/language_files.rs` with:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_find_foreign_lproj_dirs() {
        let dir = tempdir().unwrap();
        // Simulate an app bundle structure
        let resources = dir.path().join("MyApp.app/Contents/Resources");
        std::fs::create_dir_all(&resources).unwrap();

        // Create lproj dirs: en (user's locale = keep) + fr, de (foreign = remove)
        std::fs::create_dir(resources.join("en.lproj")).unwrap();
        std::fs::create_dir(resources.join("fr.lproj")).unwrap();
        std::fs::create_dir(resources.join("de.lproj")).unwrap();

        let results = find_foreign_lproj_dirs(&[dir.path().to_path_buf()], "en").unwrap();
        let names: Vec<_> = results.iter()
            .map(|r| r.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(names.contains(&"fr.lproj".to_string()));
        assert!(names.contains(&"de.lproj".to_string()));
        assert!(!names.contains(&"en.lproj".to_string()), "user locale should be kept");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test test_find_foreign_lproj 2>&1
```
Expected: FAIL.

- [ ] **Step 3: Implement language_files.rs**

```rust
//! Scan for non-locale .lproj language bundles in app directories

use std::path::PathBuf;
use anyhow::Result;

pub struct LanguageFile {
    pub path: PathBuf,
    pub size: u64,
    pub locale: String,
}

/// Find .lproj directories that don't match the user's locale.
/// `search_dirs` should be `/Applications/` or similar app dirs.
/// `user_locale` is the base locale to keep (e.g. "en", "fr").
pub fn find_foreign_lproj_dirs(search_dirs: &[PathBuf], user_locale: &str) -> Result<Vec<LanguageFile>> {
    // Normalize user locale to base (e.g. "en_US" -> "en")
    let base_locale = user_locale.split('_').next().unwrap_or("en").to_lowercase();

    let mut results = Vec::new();

    for dir in search_dirs {
        if !dir.exists() { continue; }
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_dir() { continue; }

            let name = match path.file_name() {
                Some(n) => n.to_string_lossy().to_lowercase(),
                None => continue,
            };

            if !name.ends_with(".lproj") { continue; }

            let locale = name.trim_end_matches(".lproj");
            let locale_base = locale.split('_').next().unwrap_or(locale);

            // Keep if it matches user locale (exact or base)
            if locale_base == base_locale || locale == base_locale {
                continue;
            }

            // Also keep "Base" lproj — it's required by apps
            if locale == "base" { continue; }

            let size = super::calculate_dir_size(path).unwrap_or(0);
            if size == 0 { continue; }

            results.push(LanguageFile {
                path: path.to_path_buf(),
                size,
                locale: locale.to_string(),
            });
        }
    }

    results.sort_by(|a, b| b.size.cmp(&a.size));
    Ok(results)
}

/// Get the user's current locale from environment variables
pub fn user_locale() -> String {
    std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .unwrap_or_else(|_| "en_US".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_find_foreign_lproj_dirs() {
        let dir = tempdir().unwrap();
        let resources = dir.path().join("MyApp.app/Contents/Resources");
        std::fs::create_dir_all(&resources).unwrap();

        std::fs::create_dir(resources.join("en.lproj")).unwrap();
        std::fs::create_dir(resources.join("fr.lproj")).unwrap();
        std::fs::create_dir(resources.join("de.lproj")).unwrap();
        // Write a file so size > 0
        std::fs::write(resources.join("fr.lproj/Localizable.strings"), b"test").unwrap();
        std::fs::write(resources.join("de.lproj/Localizable.strings"), b"test").unwrap();

        let results = find_foreign_lproj_dirs(&[dir.path().to_path_buf()], "en").unwrap();
        let names: Vec<_> = results.iter()
            .map(|r| r.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(names.contains(&"fr.lproj".to_string()));
        assert!(names.contains(&"de.lproj".to_string()));
        assert!(!names.contains(&"en.lproj".to_string()));
    }

    #[test]
    fn test_user_locale_fallback() {
        // Should not panic even if env vars are missing
        let locale = user_locale();
        assert!(!locale.is_empty());
    }
}
```

- [ ] **Step 4: Add pub mod language_files to scanner/mod.rs**

```rust
pub mod language_files;
```

- [ ] **Step 5: Wire into run_full_scan**

In `run_full_scan`, after the downloads block and before the duplicates block, add:
```rust
// Language files: scan /Applications/ for non-locale .lproj dirs
{
    let spinner = mp.add(ProgressBar::new_spinner());
    spinner.set_style(style.clone());
    spinner.set_message("Scanning language files...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let locale = language_files::user_locale();
    let app_dirs = vec![PathBuf::from("/Applications")];
    if let Ok(lang_files) = language_files::find_foreign_lproj_dirs(&app_dirs, &locale) {
        let total: u64 = lang_files.iter().map(|f| f.size).sum();
        spinner.finish_with_message(format!("Language files: {}", ByteSize(total)));

        for lf in lang_files {
            if meets_size_filter(lf.size, min_bytes, max_bytes) {
                all_items.push(ScannedItem {
                    path: lf.path,
                    size: lf.size,
                    category: CleanupCategory::LanguageFiles,
                    last_accessed: None,
                    last_modified: None,
                });
            }
        }
    } else {
        spinner.finish_with_message("Language files: skipped");
    }
}
```

- [ ] **Step 6: Run tests**

```bash
cargo test test_find_foreign_lproj 2>&1
```
Expected: 2 tests pass.

- [ ] **Step 7: Full test suite**

```bash
cargo test 2>&1
```
Expected: all tests pass.

- [ ] **Step 8: Commit**

```bash
git add src/scanner/language_files.rs src/scanner/mod.rs
git commit -m "feat: add language files scanner for non-locale .lproj bundles"
```

---

## Task 11: Final Build, Test, and Push

- [ ] **Step 1: Run full test suite**

```bash
cargo test 2>&1
```
Expected: all tests pass with no warnings about unused deps.

- [ ] **Step 2: Build release binary**

```bash
cargo build --release 2>&1
```
Expected: compiles. Binary at `target/release/resikno`.

- [ ] **Step 3: Smoke test key commands**

```bash
# Should show real config values
./target/release/resikno config

# Should work end-to-end (shows prompt, dry run if no --execute)
./target/release/resikno scan --no-interactive

# Should now run the duplicate engine
./target/release/resikno analyze --duplicates

# Should show old files
./target/release/resikno analyze --old 90
```

- [ ] **Step 4: Final commit and push**

```bash
git add -A
git status  # Review — make sure no .env or secrets
git commit -m "chore: final build verification" --allow-empty
git push origin main
```
