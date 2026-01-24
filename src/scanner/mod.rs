//! Scanner module - Disk scanning and analysis engine
//!
//! # Safety
//! This module includes a protected paths list to prevent accidental deletion
//! of critical system files and user data.

pub mod cache;
pub mod duplicates;
pub mod large_files;

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use bytesize::ByteSize;
use anyhow::Result;
use crate::platform::PlatformPaths;

// ============================================================================
// PROTECTED PATHS - NEVER DELETE THESE
// ============================================================================
// This is our primary defense against catastrophic data loss.
// Any path containing these patterns will be BLOCKED from deletion.
// ============================================================================

/// Paths that are NEVER safe to delete, regardless of user intent.
/// These patterns are matched case-insensitively against full paths.
pub const PROTECTED_PATH_PATTERNS: &[&str] = &[
    // macOS System directories
    "/system",
    "/usr",
    "/bin",
    "/sbin",
    "/private/var/db",
    "/library/frameworks",
    "/applications",

    // User critical data
    "/documents",
    "/desktop",
    "/pictures",
    "/movies",
    "/music",

    // Credentials & security (CRITICAL)
    "/.ssh",
    "/.gnupg",
    "/.gpg",
    "/.aws",
    "/.azure",
    "/.kube",
    "/.docker/config",
    "/keychain",

    // Development (potential data loss)
    "/.git/objects",  // Git object database
    "/node_modules",  // Can be reinstalled but takes time

    // Application data that shouldn't be auto-deleted
    "/application support/",
    "/preferences/",
];

/// Paths that require extra confirmation (large impact)
pub const CAUTION_PATH_PATTERNS: &[&str] = &[
    "/library/developer",  // Xcode data - large but regeneratable
    "/mobileSync/backup",  // iOS backups - user should decide
    "/deriveddata",        // Build artifacts
];

/// Check if a path is protected from deletion
pub fn is_protected_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    PROTECTED_PATH_PATTERNS.iter().any(|pattern| {
        path_str.contains(&pattern.to_lowercase())
    })
}

/// Check if a path requires extra caution/confirmation
pub fn requires_caution(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    CAUTION_PATH_PATTERNS.iter().any(|pattern| {
        path_str.contains(&pattern.to_lowercase())
    })
}

/// Safety level for cleanup categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyLevel {
    /// Can be cleaned automatically with minimal risk
    Safe,
    /// Review suggested before cleaning
    MostlySafe,
    /// Manual review required
    ReviewCarefully,
    /// Not recommended for automatic cleanup
    Caution,
    /// Protected - will not be offered for cleanup
    Protected,
}

impl SafetyLevel {
    pub fn emoji(&self) -> &'static str {
        match self {
            SafetyLevel::Safe => "🔵",
            SafetyLevel::MostlySafe => "🟢",
            SafetyLevel::ReviewCarefully => "🟡",
            SafetyLevel::Caution => "🔴",
            SafetyLevel::Protected => "⚪",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SafetyLevel::Safe => "SAFE",
            SafetyLevel::MostlySafe => "REVIEW",
            SafetyLevel::ReviewCarefully => "CAREFUL",
            SafetyLevel::Caution => "CAUTION",
            SafetyLevel::Protected => "KEEP",
        }
    }
}

/// Category of cleanable items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupCategory {
    SystemCaches,
    AppCaches,
    Logs,
    TempFiles,
    IOSBackups,
    XcodeData,
    Duplicates,
    LargeFiles,
    Downloads,
    LanguageFiles,
}

impl CleanupCategory {
    pub fn safety_level(&self) -> SafetyLevel {
        match self {
            CleanupCategory::SystemCaches => SafetyLevel::Safe,
            CleanupCategory::AppCaches => SafetyLevel::Safe,
            CleanupCategory::TempFiles => SafetyLevel::Safe,
            CleanupCategory::Logs => SafetyLevel::MostlySafe,
            CleanupCategory::IOSBackups => SafetyLevel::MostlySafe,
            CleanupCategory::XcodeData => SafetyLevel::MostlySafe,
            CleanupCategory::LanguageFiles => SafetyLevel::MostlySafe,
            CleanupCategory::Duplicates => SafetyLevel::ReviewCarefully,
            CleanupCategory::LargeFiles => SafetyLevel::ReviewCarefully,
            CleanupCategory::Downloads => SafetyLevel::ReviewCarefully,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            CleanupCategory::SystemCaches => "System Caches",
            CleanupCategory::AppCaches => "App Caches",
            CleanupCategory::Logs => "Logs",
            CleanupCategory::TempFiles => "Temp Files",
            CleanupCategory::IOSBackups => "iOS Backups",
            CleanupCategory::XcodeData => "Xcode Data",
            CleanupCategory::Duplicates => "Duplicates",
            CleanupCategory::LargeFiles => "Large Files",
            CleanupCategory::Downloads => "Downloads",
            CleanupCategory::LanguageFiles => "Language Files",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            CleanupCategory::SystemCaches => "📦",
            CleanupCategory::AppCaches => "📦",
            CleanupCategory::Logs => "📋",
            CleanupCategory::TempFiles => "🗑️",
            CleanupCategory::IOSBackups => "📱",
            CleanupCategory::XcodeData => "🔨",
            CleanupCategory::Duplicates => "📄",
            CleanupCategory::LargeFiles => "📁",
            CleanupCategory::Downloads => "📂",
            CleanupCategory::LanguageFiles => "🌐",
        }
    }
}

/// A single scanned item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedItem {
    pub path: PathBuf,
    pub size: u64,
    pub category: CleanupCategory,
    pub last_accessed: Option<chrono::DateTime<chrono::Utc>>,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
}

impl ScannedItem {
    pub fn human_size(&self) -> String {
        ByteSize(self.size).to_string()
    }
}

/// Results from a scan operation
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ScanResults {
    pub items: Vec<ScannedItem>,
    pub total_size: u64,
    pub total_recoverable: u64,
}

impl ScanResults {
    pub fn human_total_size(&self) -> String {
        ByteSize(self.total_size).to_string()
    }

    pub fn human_recoverable(&self) -> String {
        ByteSize(self.total_recoverable).to_string()
    }
}

/// Run a full system scan for cleanable files
///
/// # Arguments
/// * `platform` - Platform-specific path provider
/// * `_custom_path` - Optional custom path to scan (not yet implemented)
/// * `min_size_mb` - Minimum file size in MB for Downloads/LargeFiles (0 = no minimum)
/// * `max_size_mb` - Maximum file size in MB for Downloads/LargeFiles (0 = no maximum)
pub fn run_full_scan<P: PlatformPaths>(
    platform: &P,
    _custom_path: Option<&Path>,
    min_size_mb: u64,
    max_size_mb: u64,
) -> Result<ScanResults> {
    let mut results = ScanResults::default();
    let min_bytes = min_size_mb * 1024 * 1024;
    let max_bytes = max_size_mb * 1024 * 1024;

    // 1. System Caches
    let caches = cache::scan_caches(platform)?;
    for (path, size) in caches {
        results.items.push(ScannedItem {
            path,
            size,
            category: CleanupCategory::SystemCaches,
            last_accessed: None,
            last_modified: None,
        });
    }

    // 2. App Caches
    for dir in platform.app_cache_dirs() {
        if dir.exists() {
            if let Ok(size) = calculate_dir_size(&dir) {
                if size > 0 {
                    results.items.push(ScannedItem {
                        path: dir,
                        size,
                        category: CleanupCategory::AppCaches,
                        last_accessed: None,
                        last_modified: None,
                    });
                }
            }
        }
    }

    // 3. Logs
    for dir in platform.log_dirs() {
        if dir.exists() {
            if let Ok(size) = calculate_dir_size(&dir) {
                if size > 0 {
                    results.items.push(ScannedItem {
                        path: dir,
                        size,
                        category: CleanupCategory::Logs,
                        last_accessed: None,
                        last_modified: None,
                    });
                }
            }
        }
    }

    // 4. Temp Files
    for dir in platform.temp_dirs() {
        if dir.exists() {
            if let Ok(size) = calculate_dir_size(&dir) {
                if size > 0 {
                    results.items.push(ScannedItem {
                        path: dir,
                        size,
                        category: CleanupCategory::TempFiles,
                        last_accessed: None,
                        last_modified: None,
                    });
                }
            }
        }
    }

    // 5. iOS Backups
    for dir in platform.mobile_backup_dirs() {
        if dir.exists() {
            if let Ok(size) = calculate_dir_size(&dir) {
                if size > 0 {
                    results.items.push(ScannedItem {
                        path: dir,
                        size,
                        category: CleanupCategory::IOSBackups,
                        last_accessed: None,
                        last_modified: None,
                    });
                }
            }
        }
    }

    // 6. Xcode Data
    for dir in platform.xcode_dirs() {
        if dir.exists() {
            if let Ok(size) = calculate_dir_size(&dir) {
                if size > 0 {
                    results.items.push(ScannedItem {
                        path: dir,
                        size,
                        category: CleanupCategory::XcodeData,
                        last_accessed: None,
                        last_modified: None,
                    });
                }
            }
        }
    }

    // 7. Downloads - large files
    if let Some(downloads) = platform.downloads_dir() {
        if downloads.exists() {
            let large = large_files::find_large_files(&[downloads], min_bytes, max_bytes)?;
            for file in large {
                results.items.push(ScannedItem {
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
    results.total_size = results.items.iter().map(|i| i.size).sum();
    results.total_recoverable = results.items
        .iter()
        .filter(|i| matches!(i.category.safety_level(), SafetyLevel::Safe | SafetyLevel::MostlySafe))
        .map(|i| i.size)
        .sum();

    Ok(results)
}

/// Calculate the total size of a directory
fn calculate_dir_size(path: &Path) -> Result<u64> {
    use walkdir::WalkDir;

    let mut total = 0u64;
    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                total += metadata.len();
            }
        }
    }
    Ok(total)
}

/// A child entry within a scanned item (for drill-down)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildEntry {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
}

/// Scan immediate children of a directory for drill-down
pub fn scan_directory_children(path: &Path) -> Result<Vec<ChildEntry>> {
    let mut children = Vec::new();

    if !path.is_dir() {
        return Ok(children);
    }

    let entries = std::fs::read_dir(path)?;

    for entry in entries.filter_map(|e| e.ok()) {
        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files starting with . (except in specific directories)
        if name.starts_with('.') && !name.starts_with("..") {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_dir = metadata.is_dir();
        let size = if is_dir {
            calculate_dir_size(&entry_path).unwrap_or(0)
        } else {
            metadata.len()
        };

        // Skip empty entries
        if size == 0 {
            continue;
        }

        children.push(ChildEntry {
            path: entry_path,
            name,
            size,
            is_dir,
        });
    }

    // Sort by size descending
    children.sort_by(|a, b| b.size.cmp(&a.size));

    Ok(children)
}
