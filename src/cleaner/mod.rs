//! Cleaner module - Safe cleanup operations with restore points

pub mod backup;
pub mod delete;

use std::path::PathBuf;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Result of a cleanup operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub items_deleted: usize,
    pub bytes_freed: u64,
    pub restore_point: Option<RestorePoint>,
    pub errors: Vec<CleanupError>,
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

/// Perform cleanup on the specified paths
pub fn cleanup(paths: &[PathBuf], options: &CleanupOptions) -> Result<CleanupResult> {
    let mut result = CleanupResult {
        items_deleted: 0,
        bytes_freed: 0,
        restore_point: None,
        errors: Vec::new(),
    };

    // Create restore point if needed
    if options.execute && options.create_restore_point {
        result.restore_point = Some(backup::create_restore_point(paths)?);
    }

    // Perform deletion
    if options.execute {
        for path in paths {
            match delete::safe_delete(path) {
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
