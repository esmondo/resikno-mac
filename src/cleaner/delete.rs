//! Safe deletion operations
//!
//! # Safety First
//! This module implements multiple layers of protection:
//! 1. Protected path checking (blocks system/critical files)
//! 2. Pre-flight validation (path exists, readable, etc.)
//! 3. Size threshold warnings
//! 4. Trash-first approach on macOS

use std::fs;
use std::path::Path;
use anyhow::{bail, Context, Result};
use crate::scanner::{is_protected_path, requires_caution};

/// Size threshold (1 GB) above which we require extra confirmation
pub const LARGE_DELETION_THRESHOLD: u64 = 1024 * 1024 * 1024;

/// Result of a pre-deletion safety check
#[derive(Debug)]
pub enum SafetyCheckResult {
    /// Safe to proceed
    Safe,
    /// Requires user confirmation (with reason)
    RequiresConfirmation(String),
    /// Blocked - cannot delete this path
    Blocked(String),
}

/// Perform pre-flight safety checks before deletion
pub fn check_deletion_safety(path: &Path) -> Result<SafetyCheckResult> {
    // Check 1: Path must exist
    if !path.exists() {
        bail!("Path does not exist: {}", path.display());
    }

    // Check 2: Never delete protected paths
    if is_protected_path(path) {
        return Ok(SafetyCheckResult::Blocked(format!(
            "PROTECTED: '{}' is a protected path and cannot be deleted. \
             This includes system files, user documents, and credentials.",
            path.display()
        )));
    }

    // Check 3: Caution paths require confirmation
    if requires_caution(path) {
        return Ok(SafetyCheckResult::RequiresConfirmation(format!(
            "CAUTION: '{}' may contain important data. Please confirm deletion.",
            path.display()
        )));
    }

    // Check 4: Large deletions require confirmation
    let size = calculate_dir_size(path).unwrap_or(0);
    if size > LARGE_DELETION_THRESHOLD {
        let size_gb = size as f64 / (1024.0 * 1024.0 * 1024.0);
        return Ok(SafetyCheckResult::RequiresConfirmation(format!(
            "LARGE: This will delete {:.2} GB of data. Please confirm.",
            size_gb
        )));
    }

    Ok(SafetyCheckResult::Safe)
}

/// Safely delete a file or directory with all safety checks
/// Returns the size of the deleted item
pub fn safe_delete(path: &Path) -> Result<u64> {
    // Run safety checks first
    match check_deletion_safety(path)? {
        SafetyCheckResult::Blocked(reason) => {
            bail!("{}", reason);
        }
        SafetyCheckResult::RequiresConfirmation(reason) => {
            // In a real implementation, this would prompt the user
            // For now, we log and proceed (caller should handle this)
            eprintln!("Warning: {}", reason);
        }
        SafetyCheckResult::Safe => {}
    }

    // Calculate size before deletion
    let size = calculate_dir_size(path)?;

    // Perform deletion
    if path.is_file() {
        fs::remove_file(path)
            .with_context(|| format!("Failed to delete file: {}", path.display()))?;
    } else {
        fs::remove_dir_all(path)
            .with_context(|| format!("Failed to delete directory: {}", path.display()))?;
    }

    Ok(size)
}

/// Move to trash instead of permanent deletion (macOS)
/// This is the PREFERRED method - items can be recovered from Trash
#[cfg(target_os = "macos")]
pub fn move_to_trash(path: &Path) -> Result<u64> {
    use std::process::Command;

    // Run safety checks first
    match check_deletion_safety(path)? {
        SafetyCheckResult::Blocked(reason) => {
            bail!("{}", reason);
        }
        SafetyCheckResult::RequiresConfirmation(reason) => {
            eprintln!("Warning: {}", reason);
        }
        SafetyCheckResult::Safe => {}
    }

    let size = calculate_dir_size(path)?;

    // Use macOS Finder to move to trash (recoverable)
    let status = Command::new("osascript")
        .args([
            "-e",
            &format!(
                "tell application \"Finder\" to delete POSIX file \"{}\"",
                path.display()
            ),
        ])
        .status()
        .with_context(|| "Failed to execute osascript")?;

    if !status.success() {
        bail!("Failed to move to trash: {}", path.display());
    }

    Ok(size)
}

/// Move to trash (non-macOS fallback)
#[cfg(not(target_os = "macos"))]
pub fn move_to_trash(path: &Path) -> Result<u64> {
    // On non-macOS, fall back to permanent deletion
    // TODO: Implement proper trash support for Linux/Windows
    safe_delete(path)
}

/// Calculate total size of a file or directory
pub fn calculate_dir_size(path: &Path) -> Result<u64> {
    if path.is_file() {
        return Ok(path.metadata()?.len());
    }

    let mut total = 0u64;
    for entry in walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() {
            total += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    Ok(total)
}

/// Check if a path matches known safe-to-delete patterns
pub fn matches_safe_pattern(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    // Patterns that are generally safe to clean
    const SAFE_PATTERNS: &[&str] = &[
        "/caches/",
        "/cache/",
        "/logs/",
        "/log/",
        "/tmp/",
        "/temp/",
        "/deriveddata/",
        "/xcuserdata/",
        ".trash/",
    ];

    SAFE_PATTERNS.iter().any(|pattern| path_str.contains(pattern))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_safe_patterns() {
        let cache = Path::new("/Users/test/Library/Caches/com.app");
        assert!(matches_safe_pattern(cache));

        let logs = Path::new("/var/log/system.log");
        assert!(matches_safe_pattern(logs));
    }

    #[test]
    fn test_protected_paths() {
        let documents = Path::new("/Users/test/Documents/important.txt");
        assert!(is_protected_path(documents));

        let ssh = Path::new("/Users/test/.ssh/id_rsa");
        assert!(is_protected_path(ssh));

        let system = Path::new("/System/Library/something");
        assert!(is_protected_path(system));
    }

    #[test]
    fn test_cache_not_protected() {
        let cache = Path::new("/Users/test/Library/Caches/com.app");
        assert!(!is_protected_path(cache));
    }
}
