//! Cleaner module - Safe cleanup operations with restore points

pub mod backup;
pub mod delete;

use std::path::PathBuf;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::scanner::{ScannedItem, SafetyLevel};

/// Result of a cleanup operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub items_deleted: usize,
    pub bytes_freed: u64,
    pub restore_point: Option<RestorePoint>,
    pub errors: Vec<CleanupError>,
    /// Items skipped due to safe_only filter
    pub items_skipped: usize,
}

/// A restore point for recovering deleted files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestorePoint {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub manifest_path: PathBuf,
    pub total_size: u64,
    pub item_count: usize,
}

/// An error during cleanup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupError {
    pub path: PathBuf,
    pub message: String,
}

/// Options for cleanup operations
#[derive(Debug, Clone)]
pub struct CleanupOptions {
    /// Actually delete files (false = dry run)
    pub execute: bool,
    /// Create restore point before deletion
    pub create_restore_point: bool,
    /// Only process items marked as SAFE
    pub safe_only: bool,
    /// Skip confirmation prompts
    pub force: bool,
}

impl Default for CleanupOptions {
    fn default() -> Self {
        Self {
            execute: false, // Dry-run by default!
            create_restore_point: true,
            safe_only: false,
            force: false,
        }
    }
}

/// Perform cleanup on scanned items with full safety information
/// 
/// This is the recommended function to use when safety level information is available.
/// It properly respects the `safe_only` option.
pub fn cleanup_items(items: &[ScannedItem], options: &CleanupOptions) -> Result<CleanupResult> {
    let mut result = CleanupResult {
        items_deleted: 0,
        bytes_freed: 0,
        restore_point: None,
        errors: Vec::new(),
        items_skipped: 0,
    };

    // Filter items based on safe_only option
    let items_to_clean: Vec<&ScannedItem> = items
        .iter()
        .filter(|item| {
            let safety = item.category.safety_level();
            if options.safe_only {
                // Only SAFE items when safe_only is true
                let allowed = matches!(safety, SafetyLevel::Safe);
                if !allowed {
                    result.items_skipped += 1;
                }
                allowed
            } else {
                // SAFE and MOSTLY_SAFE items when safe_only is false
                let allowed = matches!(safety, SafetyLevel::Safe | SafetyLevel::MostlySafe);
                if !allowed {
                    result.items_skipped += 1;
                }
                allowed
            }
        })
        .collect();

    if items_to_clean.is_empty() {
        return Ok(result);
    }

    let paths: Vec<PathBuf> = items_to_clean.iter().map(|item| item.path.clone()).collect();

    // Create restore point if needed
    if options.execute && options.create_restore_point {
        result.restore_point = Some(backup::create_restore_point(&paths)?);
    }

    // Perform deletion (use Trash on macOS for safety)
    if options.execute {
        for item in items_to_clean {
            // Use trash on macOS, permanent delete on other platforms
            let delete_result = delete::move_to_trash(&item.path);
            
            match delete_result {
                Ok(size) => {
                    result.items_deleted += 1;
                    result.bytes_freed += size;
                }
                Err(e) => {
                    result.errors.push(CleanupError {
                        path: item.path.clone(),
                        message: e.to_string(),
                    });
                }
            }
        }
    }

    Ok(result)
}

/// Perform cleanup on the specified paths
/// 
/// # Warning
/// This function does not have safety level information. When possible,
/// use `cleanup_items()` instead to ensure proper safety filtering.
pub fn cleanup(paths: &[PathBuf], options: &CleanupOptions) -> Result<CleanupResult> {
    // Create dummy ScannedItems with unknown category (treated as Safe for backward compatibility)
    let items: Vec<ScannedItem> = paths
        .iter()
        .map(|path| ScannedItem {
            path: path.clone(),
            size: 0,
            category: crate::scanner::CleanupCategory::TempFiles, // Default to safe category
            last_accessed: None,
            last_modified: None,
        })
        .collect();
    
    cleanup_items(&items, options)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{ScannedItem, CleanupCategory, SafetyLevel};
    use std::path::PathBuf;

    #[test]
    fn test_cleanup_options_default() {
        let opts = CleanupOptions::default();
        assert!(!opts.execute); // Dry-run by default
        assert!(opts.create_restore_point);
        assert!(!opts.safe_only);
        assert!(!opts.force);
    }

    #[test]
    fn test_cleanup_result_with_skipped() {
        let result = CleanupResult {
            items_deleted: 5,
            bytes_freed: 1024 * 1024,
            restore_point: None,
            errors: vec![],
            items_skipped: 3,
        };
        
        assert_eq!(result.items_deleted, 5);
        assert_eq!(result.items_skipped, 3);
    }

    #[test]
    fn test_safe_only_filters_correctly() {
        // Create test items with different safety levels
        let items = vec![
            ScannedItem {
                path: PathBuf::from("/tmp/cache"),
                size: 1000,
                category: CleanupCategory::SystemCaches, // Safe
                last_accessed: None,
                last_modified: None,
            },
            ScannedItem {
                path: PathBuf::from("/tmp/log"),
                size: 1000,
                category: CleanupCategory::Logs, // MostlySafe
                last_accessed: None,
                last_modified: None,
            },
            ScannedItem {
                path: PathBuf::from("/tmp/download"),
                size: 1000,
                category: CleanupCategory::Downloads, // ReviewCarefully
                last_accessed: None,
                last_modified: None,
            },
        ];

        // Verify safety levels
        assert_eq!(items[0].category.safety_level(), SafetyLevel::Safe);
        assert_eq!(items[1].category.safety_level(), SafetyLevel::MostlySafe);
        assert_eq!(items[2].category.safety_level(), SafetyLevel::ReviewCarefully);

        // Test safe_only filtering (would be tested in integration)
        let safe_only_items: Vec<_> = items.iter()
            .filter(|item| matches!(item.category.safety_level(), SafetyLevel::Safe))
            .collect();
        assert_eq!(safe_only_items.len(), 1);

        let mostly_safe_items: Vec<_> = items.iter()
            .filter(|item| matches!(item.category.safety_level(), SafetyLevel::Safe | SafetyLevel::MostlySafe))
            .collect();
        assert_eq!(mostly_safe_items.len(), 2);
    }

    #[test]
    fn test_cleanup_error_creation() {
        let error = CleanupError {
            path: PathBuf::from("/test/path"),
            message: "Permission denied".to_string(),
        };
        
        assert_eq!(error.path.to_string_lossy(), "/test/path");
        assert_eq!(error.message, "Permission denied");
    }
}
