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
#[derive(Debug, Default, Serialize, Deserialize)]
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
